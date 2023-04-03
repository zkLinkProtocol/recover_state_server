use std::ops::Sub;
use num::BigUint;
use crate::{exit_circuit::*, utils::*};

#[derive(Clone)]
pub struct CircuitElement<E: Engine> {
    number: AllocatedNum<E>,
    bits_le: Vec<Boolean>,
    length: usize,
}

impl<E: Engine> CircuitElement<E> {
    pub fn unsafe_empty_of_some_length(zero_num: AllocatedNum<E>, length: usize) -> Self {
        assert!(length <= E::Fr::NUM_BITS as usize);
        let bits = vec![Boolean::constant(false); length];
        CircuitElement {
            number: zero_num,
            bits_le: bits,
            length,
        }
    }

    pub fn into_truncated_be_bits(&self, to_length: usize) -> Vec<Boolean> {
        assert!(to_length < self.bits_le.len());
        let mut bits = self.bits_le[0..to_length].to_vec();
        bits.reverse();
        bits
    }

    pub fn into_padded_le_bits(self, to_length: usize) -> Vec<Boolean> {
        let mut bits = self.bits_le;
        assert!(to_length >= bits.len());
        bits.resize(to_length, Boolean::constant(false));

        bits
    }

    pub fn into_padded_be_bits(self, to_length: usize) -> Vec<Boolean> {
        let mut bits = self.into_padded_le_bits(to_length);
        bits.reverse();

        bits
    }

    pub fn pad(self, n: usize) -> Self {
        assert!(n <= E::Fr::NUM_BITS as usize);
        let mut padded_bits = self.get_bits_le().to_vec();
        assert!(n >= padded_bits.len());
        padded_bits.resize(n, Boolean::constant(false));
        CircuitElement {
            number: self.number,
            bits_le: padded_bits,
            length: n,
        }
    }

    pub fn from_constant_with_known_length<
        CS: ConstraintSystem<E>,
        F: FnOnce() -> Result<E::Fr, SynthesisError>,
    >(
        mut cs: CS,
        field_element: F,
        max_length: usize,
    ) -> Result<Self, SynthesisError> {
        assert!(max_length <= E::Fr::NUM_BITS as usize);
        let number =
            AllocatedNum::alloc_constant(cs.namespace(|| "number from field element"), field_element)?;
        CircuitElement::from_number_with_known_length(
            cs.namespace(|| "circuit_element"),
            number,
            max_length,
        )
    }

    pub fn from_fe_with_known_length<
        CS: ConstraintSystem<E>,
        F: FnOnce() -> Result<E::Fr, SynthesisError>,
    >(
        mut cs: CS,
        field_element: F,
        max_length: usize,
    ) -> Result<Self, SynthesisError> {
        assert!(max_length <= E::Fr::NUM_BITS as usize);
        let number =
            AllocatedNum::alloc(cs.namespace(|| "number from field element"), field_element)?;
        CircuitElement::from_number_with_known_length(
            cs.namespace(|| "circuit_element"),
            number,
            max_length,
        )
    }

    /// Does not check for congruency
    pub fn from_fe<CS: ConstraintSystem<E>, F: FnOnce() -> Result<E::Fr, SynthesisError>>(
        mut cs: CS,
        field_element: F,
    ) -> Result<Self, SynthesisError> {
        let number =
            AllocatedNum::alloc(cs.namespace(|| "number from field element"), field_element)?;
        CircuitElement::from_number(cs.namespace(|| "circuit_element"), number)
    }

    pub fn from_witness_be_bits<CS: ConstraintSystem<E>>(
        mut cs: CS,
        witness_bits: &[Option<bool>],
    ) -> Result<Self, SynthesisError> {
        assert!(witness_bits.len() <= E::Fr::NUM_BITS as usize);
        let mut allocated_bits =
            allocate_bits_vector(cs.namespace(|| "allocate bits"), witness_bits)?;
        allocated_bits.reverse();
        let length = allocated_bits.len();
        let number = pack_bits_to_element(cs.namespace(|| "ce from bits"), &allocated_bits)?;
        Ok(Self {
            number,
            bits_le: allocated_bits,
            length,
        })
    }

    pub fn from_number_with_known_length<CS: ConstraintSystem<E>>(
        mut cs: CS,
        number: AllocatedNum<E>,
        max_length: usize,
    ) -> Result<Self, SynthesisError> {
        assert!(max_length <= E::Fr::NUM_BITS as usize);
        // decode into the fixed number of bits
        let bits = if max_length <= E::Fr::CAPACITY as usize {
            number.into_bits_le_fixed(cs.namespace(|| "into_bits_le_fixed"), max_length)?
        } else {
            number.into_bits_le_strict(cs.namespace(|| "into_bits_le_strict"))?
        };

        assert_eq!(bits.len(), max_length);

        let ce = CircuitElement {
            number,
            bits_le: bits,
            length: max_length,
        };

        Ok(ce)
    }

    pub fn from_expression_padded<CS: ConstraintSystem<E>>(
        mut cs: CS,
        expr: Expression<E>,
    ) -> Result<Self, SynthesisError> {
        let bits = expr.into_bits_le(cs.namespace(|| "into_bits_le"))?;
        let number = pack_bits_to_element(cs.namespace(|| "pack back"), &bits)?;
        let ce = CircuitElement {
            number,
            bits_le: bits,
            length: E::Fr::NUM_BITS as usize,
        };

        Ok(ce)
    }

    pub fn from_le_bits<CS: ConstraintSystem<E>>(
        mut cs: CS,
        bits: Vec<Boolean>,
    ) -> Result<Self, SynthesisError> {
        assert!(bits.len() <= E::Fr::NUM_BITS as usize);
        let number = pack_bits_to_element(cs.namespace(|| "pack back"), &bits)?;
        let ce = CircuitElement {
            number,
            bits_le: bits,
            length: E::Fr::NUM_BITS as usize,
        };

        Ok(ce)
    }

    /// Does not check for congruency
    pub fn from_number<CS: ConstraintSystem<E>>(
        mut cs: CS,
        number: AllocatedNum<E>,
    ) -> Result<Self, SynthesisError> {
        let bits = number.into_bits_le(cs.namespace(|| "into_bits_le"))?;
        assert_eq!(bits.len(), E::Fr::NUM_BITS as usize);

        let bits_len = bits.len();

        let ce = CircuitElement {
            number,
            bits_le: bits,
            length: bits_len,
        };

        Ok(ce)
    }

    pub fn enforce_length<CS: ConstraintSystem<E>>(
        &self,
        mut cs: CS,
    ) -> Result<(), SynthesisError> {
        let number_repacked =
            pack_bits_to_element(cs.namespace(|| "pack truncated bits"), &self.bits_le)?;
        cs.enforce(
            || format!("number can be represented in {} bits", self.length),
            |lc| lc + self.number.get_variable(),
            |lc| lc + CS::one(),
            |lc| lc + number_repacked.get_variable(),
        );

        Ok(())
    }

    pub fn enforce_specified_length<CS: ConstraintSystem<E>>(
        &self,
        mut cs: CS,
        length: usize
    ) -> Result<(), SynthesisError> {
        if self.length <= length {
            Ok(())
        } else {
            let number_repacked =
                pack_bits_to_element(cs.namespace(|| "pack truncated bits"), &self.bits_le[0..length])?;
            cs.enforce(
                || format!("number can be represented in {} bits", length),
                |lc| lc + self.number.get_variable(),
                |lc| lc + CS::one(),
                |lc| lc + number_repacked.get_variable(),
            );

            Ok(())
        }
    }

    pub fn select_if_eq<CS: ConstraintSystem<E>>(
        mut cs: CS,
        a: &AllocatedNum<E>,
        b: &AllocatedNum<E>,
        x: &Self,
        y: &Self,
    ) -> Result<Self, SynthesisError> {
        assert!(x.length <= E::Fr::NUM_BITS as usize);
        assert_eq!(x.length, y.length);
        // select by value and repack into bits

        let selected_number = AllocatedNum::select_ifeq(
            cs.namespace(|| "select_ifeq"),
            a,
            b,
            x.get_number(),
            y.get_number(),
        )?;

        CircuitElement::from_number_with_known_length(
            cs.namespace(|| "chosen number as ce"),
            selected_number,
            x.length,
        )
    }

    // doesn't enforce length by design, though applied to both strict values will give strict result
    pub fn conditionally_select<CS: ConstraintSystem<E>>(
        mut cs: CS,
        x: &Self,
        y: &Self,
        condition: &Boolean,
    ) -> Result<Self, SynthesisError> {
        assert!(x.length <= E::Fr::NUM_BITS as usize);
        assert_eq!(x.length, y.length);

        let selected_number = AllocatedNum::conditionally_select(
            cs.namespace(|| "conditionally_select"),
            x.get_number(),
            y.get_number(),
            condition,
        )?;

        CircuitElement::from_number_with_known_length(
            cs.namespace(|| "chosen number as ce"),
            selected_number,
            x.length,
        )
    }

    pub fn conditionally_reverse<CS: ConstraintSystem<E>>(
        mut cs: CS,
        x: &Self,
        y: &Self,
        condition: &Boolean,
    ) -> Result<(Self, Self), SynthesisError> {
        assert!(x.length <= E::Fr::NUM_BITS as usize);
        assert_eq!(x.length, y.length);

        let (selected_number1, selected_number2) = AllocatedNum::conditionally_reverse(
            cs.namespace(|| "conditionally_select"),
            x.get_number(),
            y.get_number(),
            condition,
        )?;

        let selected_ce1 = CircuitElement::from_number_with_known_length(
            cs.namespace(|| "chosen number 1as ce"),
            selected_number1,
            x.length,
        )?;
        let selected_ce2 = CircuitElement::from_number_with_known_length(
            cs.namespace(|| "chosen number2 as ce"),
            selected_number2,
            x.length,
        )?;
        Ok((selected_ce1, selected_ce2))
    }

    // doesn't enforce length by design, though applied to both strict values will give strict result
    pub fn conditionally_select_with_number_strict<
        CS: ConstraintSystem<E>,
        EX: Into<Expression<E>>,
    >(
        mut cs: CS,
        x: EX,
        y: &Self,
        condition: &Boolean,
    ) -> Result<Self, SynthesisError> {
        let selected_number = Expression::conditionally_select(
            cs.namespace(|| "conditionally_select"),
            x,
            y.get_number(),
            condition,
        )?;

        CircuitElement::from_number_with_known_length(
            cs.namespace(|| "chosen number as ce"),
            selected_number,
            y.length,
        )
    }

    pub fn equals<CS: ConstraintSystem<E>>(
        mut cs: CS,
        x: &Self,
        y: &Self,
    ) -> Result<Boolean, SynthesisError> {
        let is_equal =
            AllocatedNum::equals(cs.namespace(|| "equals"), x.get_number(), y.get_number())?;
        Ok(Boolean::from(is_equal))
    }

    pub fn less_than_fixed<CS: ConstraintSystem<E>>(
        mut cs: CS,
        x: &Self,
        y: &Self,
    ) -> Result<Boolean, SynthesisError> {
        let length = std::cmp::max(x.length, y.length);
        assert!(
            length <= E::Fr::CAPACITY as usize,
            "comparison is only supported for fixed-length elements"
        );

        let base = E::Fr::from_big_uint(
            BigUint::from(2u8)
                .pow(length as u32)
                .sub(BigUint::from(1u8))
        ).unwrap();


        let expr = Expression::constant::<CS>(base) - x.get_number() + y.get_number();
        let bits = expr.into_bits_le_fixed(cs.namespace(|| "diff bits"), length + 1)?;

        Ok(bits
            .last()
            .expect("expr bit representation should always contain at least one bit")
            .clone())
    }

    pub fn less_equal_fixed<CS: ConstraintSystem<E>>(
        mut cs: CS,
        x: &Self,
        y: &Self,
    ) -> Result<Boolean, SynthesisError> {
        let length = std::cmp::max(x.length, y.length);
        assert!(
            length <= E::Fr::CAPACITY as usize,
            "comparison is only supported for fixed-length elements"
        );

        let base = E::Fr::from_big_uint(
            BigUint::from(2u8)
                .pow(length as u32)
                .sub(BigUint::from(1u8))
        ).unwrap();

        let expr = Expression::constant::<CS>(base) - x.get_number() + y.get_number();
        let bits = expr.into_bits_le_fixed(cs.namespace(|| "diff bits"), length + 1)?;

        let diff = Expression::equals(
            cs.namespace(|| "pack bits to element"),
            expr, Expression::constant::<CS>(base)
        )?;

        let less_equal = Boolean::and(
            cs.namespace(|| "less equal"),
            &Boolean::from(diff).not(),
            &bits.last()
                .expect("expr bit representation should always contain at least one bit")
                .not()
        )?.not();

        Ok(less_equal)
    }

    pub fn get_number(&self) -> &AllocatedNum<E> {
        &self.number
    }

    pub fn into_number(self) -> AllocatedNum<E> {
        self.number
    }

    pub fn get_bits_le(&self) -> &[Boolean] {
        &self.bits_le
    }

    pub fn bits_length(&self) -> usize{ self.length }

    pub fn get_bits_be(&self) -> Vec<Boolean> {
        let mut bits_be = self.bits_le.clone();
        bits_be.reverse();
        bits_be
    }

    pub fn grab(&self) -> Result<E::Fr, SynthesisError> {
        self.number
            .get_value()
            .ok_or(SynthesisError::AssignmentMissing)
    }
}
