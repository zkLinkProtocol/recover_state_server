// Workspace deps
// Local deps
use crate::{circuit::*, witness::*};

pub fn reverse_bytes<T: Clone>(bits: &[T]) -> Vec<T> {
    assert_eq!(bits.len() % 8, 0);
    bits.chunks(8)
        .rev()
        .map(|x| x.to_vec())
        .fold(Vec::new(), |mut acc, mut byte| {
            acc.append(&mut byte);
            acc
        })
}

pub fn multi_or<E: JubjubEngine, CS: ConstraintSystem<E>>(
    mut cs: CS,
    x: &[Boolean],
) -> Result<Boolean, SynthesisError> {
    let mut lc = Num::zero();

    for bool_x in x.iter() {
        lc = lc.add_bool_with_coeff(CS::one(), bool_x, E::Fr::one());
    }

    Ok(Boolean::from(Expression::equals(
        cs.namespace(||"asserts whether boolean sum is equal to 0"),
        &lc,
        Expression::u64::<CS>(0 as u64)
    )?).not())
}

pub fn multi_and<E: Engine, CS: ConstraintSystem<E>>(
    mut cs: CS,
    x: &[Boolean],
) -> Result<Boolean, SynthesisError> {
    let mut lc = Num::zero();

    for bool_x in x.iter() {
        lc = lc.add_bool_with_coeff(CS::one(), bool_x, E::Fr::one());
    }

    Ok(Boolean::from(Expression::equals(
        cs.namespace(||"asserts whether boolean sum is equal to the Vector constant size"),
        &lc,
        Expression::u64::<CS>(x.len() as u64)
    )?))
}

pub fn allocate_sum<E: Engine, CS: ConstraintSystem<E>>(
    mut cs: CS,
    a: &AllocatedNum<E>,
    b: &AllocatedNum<E>,
) -> Result<AllocatedNum<E>, SynthesisError> {
    let sum = AllocatedNum::alloc(cs.namespace(|| "sum"), || {
        let mut sum = a.get_value().grab()?;
        sum.add_assign(&b.get_value().grab()?);
        Ok(sum)
    })?;
    cs.enforce(
        || "enforce sum",
        |lc| lc + a.get_variable() + b.get_variable(),
        |lc| lc + CS::one(),
        |lc| lc + sum.get_variable(),
    );

    Ok(sum)
}

pub fn pack_bits_to_element<E: Engine, CS: ConstraintSystem<E>>(
    mut cs: CS,
    bits: &[Boolean],
) -> Result<AllocatedNum<E>, SynthesisError> {
    assert!(
        bits.len() <= E::Fr::NUM_BITS as usize,
        "can not pack bits into field element: number of bits is larger than number of bits in a field"
    );
    let mut data_from_lc = Num::<E>::zero();
    let mut coeff = E::Fr::one();
    for bit in bits {
        data_from_lc = data_from_lc.add_bool_with_coeff(CS::one(), &bit, coeff);
        coeff.double();
    }

    let data_packed = AllocatedNum::alloc(
        cs.namespace(|| "allocate account data packed"),
        || Ok(*data_from_lc.get_value().get()?)
    )?;

    cs.enforce(
        || "pack account data",
        |lc| lc + data_packed.get_variable(),
        |lc| lc + CS::one(),
        |_| data_from_lc.lc(E::Fr::one()),
    );

    Ok(data_packed)
}

pub fn pack_bits_to_element_strict<E: Engine, CS: ConstraintSystem<E>>(
    cs: CS,
    bits: &[Boolean],
) -> Result<AllocatedNum<E>, SynthesisError> {
    assert!(
        bits.len() <= E::Fr::CAPACITY as usize,
        "can not pack bits into field element over the precision"
    );

    pack_bits_to_element(cs, bits)
}

pub fn multiplication_fr_with_arbitrary_precision<E:Engine>(
    a:E::Fr,
    b:E::Fr,
    precision:u64
) -> Option<E::Fr> where E::Fr:FeConvert{
    assert!(precision <= 18);
    let a_to_big_uint = a.into_big_uint();
    let b_to_big_uint = b.into_big_uint();
    let reduction_factor = BigUint::from(10u8).pow(precision as u32);

    let product = a_to_big_uint * b_to_big_uint / reduction_factor;
    E::Fr::from_big_uint(product)
}

pub fn div_fr_with_arbitrary_precision<E:Engine>(
    a:E::Fr,
    b:E::Fr,
    precision:u64
)-> Option<E::Fr> where E::Fr: FeConvert {
    assert!(precision <= 18);
    let a_to_big_uint = a.into_big_uint();
    let b_to_big_uint = b.into_big_uint();
    let amplification_factor = BigUint::from(10u8).pow(precision as u32);

    let quotient = a_to_big_uint * amplification_factor / b_to_big_uint;
    E::Fr::from_big_uint(quotient)
}

pub fn multiplication_and_sqrt<E:Engine>(a:E::Fr, b:E::Fr)-> Option<E::Fr>
    where E::Fr: FeConvert
{
    let a_to_big_uint = a.into_big_uint();
    let b_to_big_uint = b.into_big_uint();
    let product_sqrt = (a_to_big_uint * b_to_big_uint).sqrt();
    E::Fr::from_big_uint(product_sqrt)
}

pub fn multiply_based_on_u126<E:Engine, CS: ConstraintSystem<E>>(
    mut cs: CS,
    a: &CircuitElement<E>,
    b: &CircuitElement<E>,
    precision:u64
) -> Result<AllocatedNum<E>, SynthesisError>{
    assert!(precision <= 18);
    // Let's check a and b so that we don't overmultiply
    pre_check_to_prevent_overflow(
        cs.namespace(||"Pre-check to prevent overflow"),
        a, b
    )?;

    let product = a.get_number().mul(cs.namespace(||"product"),b.get_number())?;
    let product = CircuitElement::from_number_with_known_length(
        cs.namespace(||"product convert as ce"),
        product,
        E::Fr::CAPACITY as usize
    )?;
    let reduction_factor = AllocatedNum::alloc_constant(
        cs.namespace(|| "reduction_factor"),
        ||Ok(E::Fr::from_u64(10u64.pow(precision as u32))),
    )?;

    // allocated calculation result
    let quotient = CircuitElement::from_fe_with_known_length(
        cs.namespace(||"product div reduction_factor"),
        || if let (Some(a), Some(b)) = (product.get_number().get_value(), reduction_factor.get_value()){
            Ok(div_fr_with_arbitrary_precision::<E>(a, b, 0).unwrap())
        } else { Ok(E::Fr::one()) },
        BALANCE_BIT_WIDTH
    )?;
    // quotient <= 128 bits, reduction_factor <= 10^18, so no overflow(253bits).
    let quotient_mul_b = quotient.get_number().mul(cs.namespace(||"quotient * b"), &reduction_factor)?;
    let upper_bound = quotient_mul_b.add(cs.namespace(||"(quotient + 1) * b"), &reduction_factor)?;
    // compute lower bound: reduction_factor*q
    let lower_bound = CircuitElement::from_number_with_known_length(
        cs.namespace(|| "lower_bound"),
        quotient_mul_b,
        E::Fr::CAPACITY as usize
    )?;
    // compute upper bound: reduction_factor*(q+1)
    let upper_bound = CircuitElement::from_number_with_known_length(
        cs.namespace(|| "upper_bound"),
        upper_bound,
        E::Fr::CAPACITY as usize
    )?;
    // reduction_factor*q <= product < reduction_factor*(q+1)
    let range_check = CircuitElement::less_equal_fixed(
        cs.namespace(||"check lower bound"),
        &lower_bound, &product
    )?;
    let range_check1 = CircuitElement::less_than_fixed(
        cs.namespace(||"check upper bound"),
        &product, &upper_bound
    )?;
    let is_correct_division = Boolean::and(
        cs.namespace(|| "is_correct_division"),
        &range_check, &range_check1
    )?;
    Boolean::enforce_equal(
        cs.namespace(|| "range_check"),
        &is_correct_division, &Boolean::constant(true)
    )?;
    Ok(quotient.get_number().clone())
}

// This division requires that neither a nor b exceed 2^126
pub fn div_based_on_u126<E:Engine, CS: ConstraintSystem<E>>(
    mut cs: CS,
    a: &CircuitElement<E>,
    b: &CircuitElement<E>,
    precision:u64
) -> Result<CircuitElement<E>, SynthesisError> where E::Fr: FeConvert {
    assert!(precision <= 18);
    // Let's check a and b so that we don't overmultiply
    pre_check_to_prevent_overflow(
        cs.namespace(||"Pre-check to prevent overflow"),
        a, b
    )?;

    let is_a_zero = Boolean::from(Expression::equals(
        cs.namespace(|| "is a zero"),
        a.get_number(),
        Expression::constant::<CS>(E::Fr::zero()),
    )?);
    let is_b_zero = Boolean::from(Expression::equals(
        cs.namespace(|| "is b zero"),
        b.get_number(),
        Expression::constant::<CS>(E::Fr::zero()),
    )?);

    let disallow_divisor_is_zero = AllocatedNum::alloc_constant(
        cs.namespace(|| "disallow divisor is zero"),
        ||Ok(E::Fr::from_u64(10u64.pow(TOKEN_MAX_PRECISION as u32))),
    )?;
    let divisor = AllocatedNum::conditionally_select(
        cs.namespace(|| "ensure not div zero"),
        &disallow_divisor_is_zero,
         b.get_number(),
        &is_b_zero,
    )?;

    // Magnify 10^18 times, reduce the inaccuracy
    // compute magnify_a constraints
    let amplification_factor = AllocatedNum::alloc_constant(
        cs.namespace(|| "amplification_factor"),
        ||Ok(E::Fr::from_u64(10u64.pow(precision as u32))),
    )?;
    // a <= 126 bits , amplification_factor <= 10^18, so no overflow.
    let magnify_a = a.get_number().mul(cs.namespace(||"magnify_a"), &amplification_factor)?;
    let magnify_a = CircuitElement::from_number_with_known_length(
        cs.namespace(|| "magnify_a with bits"),
        magnify_a,
        E::Fr::CAPACITY as usize
    )?;

    // allocated calculation result
    let quotient = CircuitElement::from_fe_with_known_length(
        cs.namespace(||"product div reduction_factor"),
        || if let (Some(a), Some(b)) = (a.get_number().get_value(), divisor.get_value()){
            Ok(div_fr_with_arbitrary_precision::<E>(a, b, precision).unwrap())
        } else { Ok(E::Fr::one()) },
        MAX_CALCULATION_BIT_WIDTH
    )?;
    // 126 + 126 = 252 < E::Fr::CAPACITY, so no overflow.
    let quotient_mul_b = quotient.get_number().mul(cs.namespace(||"quotient * b"), &divisor)?;
    let upper_bound = quotient_mul_b.add(cs.namespace(||"(quotient + 1) * b"), &divisor)?;
    // compute upper bound: b*(q+1)
    let upper_bound = CircuitElement::from_number_with_known_length(
        cs.namespace(|| "upper_bound"),
        upper_bound,
        E::Fr::CAPACITY as usize
    )?;
    // compute lower bound: b*q
    let lower_bound = CircuitElement::from_number_with_known_length(
        cs.namespace(|| "lower_bound"),
        quotient_mul_b,
        E::Fr::CAPACITY as usize
    )?;

    // b*q <= a*magnify < b*(q+1)
    let range_check = CircuitElement::less_equal_fixed(
        cs.namespace(||"check lower bound"),
        &lower_bound, &magnify_a
    )?;
    let range_check1 = CircuitElement::less_than_fixed(
        cs.namespace(||"check upper bound"),
        &magnify_a, &upper_bound
    )?;
    let is_correct_division = Boolean::and(
        cs.namespace(|| "is_correct_division"),
        &range_check, &range_check1
    )?;
    Boolean::enforce_equal(
        cs.namespace(|| "range_check"),
        &is_correct_division, &Boolean::constant(true)
    )?;

    let is_not_a_zero_and_is_b_zero = Boolean::and(
        cs.namespace(||"is_not_a_zero and is_b_zero"),
        &is_a_zero.not(),
        &is_b_zero
    )?;
    let quotient = AllocatedNum::conditionally_select(
        cs.namespace(||"selected correct quotient"),
        &disallow_divisor_is_zero,
        &quotient.get_number(),
        &is_not_a_zero_and_is_b_zero
    )?;
    CircuitElement::from_number_with_known_length(
        cs.namespace(|| "three precision quotient"),
        quotient,
        BALANCE_BIT_WIDTH
    )
}

// result = sqrt(x*y)
// This sqrt requires that neither x nor y exceed 2^126
pub fn sqrt_enforce<E: Engine, CS: ConstraintSystem<E>>(
    mut cs: CS,
    x: &CircuitElement<E>,
    y: &CircuitElement<E>,
) -> Result<CircuitElement<E>, SynthesisError> where E::Fr:FeConvert {
    // Let's check x and y so that we don't overmultiply
    pre_check_to_prevent_overflow(
        cs.namespace(||"Pre-check to prevent overflow"),
        x, y
    )?;

    let k = CircuitElement::from_fe_with_known_length(
        cs.namespace(|| "k sqrt"),
        || {
            let x_big_uint = x.get_number().get_value().map_or(BigUint::one(), FeConvert::into_big_uint);
            let y_big_uint = y.get_number().get_value().map_or(BigUint::one(), FeConvert::into_big_uint);
            Ok(E::Fr::from_str(&(x_big_uint * y_big_uint).sqrt().to_string()).unwrap())
        },
        MAX_CALCULATION_BIT_WIDTH
    )?;
    let product = x.get_number().mul(cs.namespace(||"x_temp mul y_temp"), y.get_number())?;
    let product = CircuitElement::from_number_with_known_length(
        cs.namespace(|| "x * y"),
        product,
        E::Fr::CAPACITY as usize
    )?;

    let lower_bound = k.get_number().square(cs.namespace(|| "k_square"))?;
    let lower_bound = CircuitElement::from_number_with_known_length(
        cs.namespace(|| "k_square add bits"),
        lower_bound,
        E::Fr::CAPACITY as usize
    )?;

    let k_add_one = k.get_number().add_constant(cs.namespace(|| "k + 1"), E::Fr::one())?;
    let upper_bound = k_add_one.square(cs.namespace(|| "(k + 1)^2"))?;
    let upper_bound = CircuitElement::from_number_with_known_length(
        cs.namespace(|| "k_square+1 add bits"),
        upper_bound,
        E::Fr::CAPACITY as usize
    )?;

    let range_check = CircuitElement::less_equal_fixed(
        cs.namespace(||"check lower bound"),
        &lower_bound, &product
    )?;
    let range_check1 = CircuitElement::less_equal_fixed(
        cs.namespace(||"check upper bound"),
        &product, &upper_bound
    )?;

    let is_correct_sqrt = Boolean::and(
        cs.namespace(|| "is_correct_sqrt"),
        &range_check, &range_check1
    )?;
    Boolean::enforce_equal(
        cs.namespace(|| "range_check"),
        &is_correct_sqrt, &Boolean::constant(true)
    )?;
    Ok(k)
}

pub fn pre_check_to_prevent_overflow<E: Engine, CS: ConstraintSystem<E>>(
    mut cs: CS,
    a: &CircuitElement<E>,
    b: &CircuitElement<E>
) -> Result<(), SynthesisError> {
    a.enforce_specified_length(
        cs.namespace(||"a must be less equal the maximum value that can be expressed in 126 bits"),
        MAX_CALCULATION_BIT_WIDTH
    )?;
    b.enforce_specified_length(
        cs.namespace(||"b must be less equal the maximum value that can be expressed in 126 bits"),
        MAX_CALCULATION_BIT_WIDTH
    )?;
    Ok(())
}

pub fn allocate_numbers_vec<E, CS>(
    mut cs: CS,
    witness_vec: &[Option<E::Fr>],
) -> Result<Vec<AllocatedNum<E>>, SynthesisError>
where
    E: Engine,
    CS: ConstraintSystem<E>,
{
    let mut allocated = Vec::new();
    for (i, e) in witness_vec.iter().enumerate() {
        let path_element =
            AllocatedNum::alloc(cs.namespace(|| format!("path element{}", i)), || {
                Ok(*e.get()?)
            })?;
        allocated.push(path_element);
    }

    Ok(allocated)
}

pub fn allocate_bits_vector<E, CS>(
    mut cs: CS,
    bits: &[Option<bool>],
) -> Result<Vec<Boolean>, SynthesisError>
where
    E: Engine,
    CS: ConstraintSystem<E>,
{
    let mut allocated = vec![];
    for (i, e) in bits.iter().enumerate() {
        let element = Boolean::from(AllocatedBit::alloc(
            cs.namespace(|| format!("path element{}", i)),
            *e,
        )?);
        allocated.push(element);
    }

    Ok(allocated)
}

pub fn print_boolean_vec(bits: &[Boolean]) {
    let mut bytes = vec![];
    for slice in bits.chunks(8) {
        let mut b = 0u8;
        for (i, bit) in slice.iter().enumerate() {
            if bit.get_value().unwrap() {
                b |= 1u8 << (7 - i);
            }
        }
        bytes.push(b);
    }
}

pub fn boolean_or<E: Engine, CS: ConstraintSystem<E>>(
    mut cs: CS,
    x: &Boolean,
    y: &Boolean,
) -> Result<Boolean, SynthesisError> {
    // A OR B = ( A NAND A ) NAND ( B NAND B ) = (NOT(A)) NAND (NOT (B))
    let result = Boolean::and(
        cs.namespace(|| "lhs_valid nand rhs_valid"),
        &x.not(),
        &y.not(),
    )?
    .not();

    Ok(result)
}

