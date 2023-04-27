use zklink_crypto::franklin_crypto::jubjub::{edwards, Unknown};
use crate::circuit::*;

pub const ARGUMENT_WITH_BOOL_CAPACITY: usize = 2;
pub const ARGUMENT_WITH_1_BYTE_CAPACITY: usize = 5;
pub const ARGUMENT_WITH_2_BYTES_CAPACITY: usize = 5;
pub const ARGUMENT_WITH_3_BYTES_CAPACITY: usize = 2;
pub const ARGUMENT_WITH_4_BYTES_CAPACITY: usize = 4;
pub const ARGUMENT_WITH_8_BYTES_CAPACITY: usize = 4;
pub const ARGUMENT_WITH_15_BYTES_CAPACITY: usize = 2;
pub const ARGUMENT_WITH_16_BYTES_CAPACITY: usize = 5;
pub const ARGUMENT_WITH_20_BYTES_CAPACITY: usize = 2;
pub const ARGUMENT_WITH_MAX_BYTES_CAPACITY: usize = 1;
pub const SPECIAL_ARGUMENT_FEE_CAPACITY: usize = 1;
pub const SPECIAL_ARGUMENT_AMMOUNT_CAPACITY: usize = 2;

#[derive(Clone, Debug)]
pub struct OperationBranchWitness<E: RescueEngine> {
    pub account_witness: AccountWitness<E>,
    pub account_path: Vec<Option<E::Fr>>,

    pub balance_value: Option<E::Fr>,
    pub balance_subtree_path: Vec<Option<E::Fr>>,

    pub order_nonce: Option<E::Fr>,
    pub order_residue: Option<E::Fr>,
    pub order_subtree_path: Vec<Option<E::Fr>>,
}

impl<E: RescueEngine> Default for OperationBranchWitness<E>{
    fn default() -> Self {
        Self{
            account_witness: Default::default(),
            account_path: vec![None; account_tree_depth()],
            balance_value: None,
            balance_subtree_path: vec![None; balance_tree_depth()],
            order_nonce: None,
            order_residue: None,
            order_subtree_path: vec![None; order_tree_depth()]
        }
    }
}

impl<E: RescueEngine> OperationBranchWitness<E>{
    fn circuit_init() -> Self {
        Self{
            account_witness: AccountWitness::circuit_init(),
            account_path: vec![Some(E::Fr::zero()); account_tree_depth()],
            balance_value: None,
            balance_subtree_path: vec![Some(E::Fr::zero()); balance_tree_depth()],
            order_nonce: None,
            order_residue: None,
            order_subtree_path: vec![Some(E::Fr::zero()); order_tree_depth()]
        }
    }
}

#[derive(Clone, Debug)]
pub struct OperationBranch<E: RescueEngine> {
    pub account_id: Option<E::Fr>,
    pub sub_account_id: Option<E::Fr>,
    pub token: Option<E::Fr>,
    pub slot_number: Option<E::Fr>,

    pub witness: OperationBranchWitness<E>,
}

impl<E: RescueEngine> Default for OperationBranch<E> {
    fn default() -> Self {
        Self {
            account_id: None,
            sub_account_id: None,
            token: None,
            slot_number: None,
            witness: Default::default()
        }
    }
}

impl<E: RescueEngine> OperationBranch<E> {
    pub fn circuit_init() -> Self {
        Self {
            account_id: Some(E::Fr::zero()),
            sub_account_id: Some(E::Fr::zero()),
            token: Some(E::Fr::zero()),
            slot_number: Some(E::Fr::zero()),
            witness: OperationBranchWitness::circuit_init()
        }
    }
}

#[derive(Clone, Debug)]
pub struct OperationUnit<E: RescueEngine> {
    pub tx_type: Option<E::Fr>,
    pub chunk: Option<E::Fr>,
    pub pubdata_chunk: Option<E::Fr>,
    pub signer_pub_key_packed: Vec<Option<bool>>,
    pub first_sig_msg: Option<E::Fr>,
    pub second_sig_msg: Option<E::Fr>,
    pub third_sig_msg: Option<E::Fr>,
    pub signature_data: SignatureData,
    pub args: OperationArguments<E>,
    pub prev_branch: OperationBranch<E>,
    pub post_branch: OperationBranch<E>
}

impl<E: RescueEngine> OperationUnit<E> {
    pub fn circuit_init() -> Self{
        Self {
            tx_type: Some(E::Fr::zero()),
            chunk: Some(E::Fr::zero()),
            pubdata_chunk: Some(E::Fr::zero()),
            signer_pub_key_packed: vec![Some(false); FR_BIT_WIDTH_PADDED],
            first_sig_msg: Some(E::Fr::zero()),
            second_sig_msg: Some(E::Fr::zero()),
            third_sig_msg: Some(E::Fr::zero()),
            signature_data: SignatureData::init_empty(),
            args: OperationArguments::circuit_init(),
            prev_branch: OperationBranch::circuit_init(),
            post_branch: OperationBranch::circuit_init()
        }
    }
}

impl<E: RescueEngine> Default for OperationUnit<E>{
    fn default() -> Self {
        Self{
            tx_type: None,
            chunk: None,
            pubdata_chunk: None,
            signer_pub_key_packed: vec![None; FR_BIT_WIDTH_PADDED],
            first_sig_msg: None,
            second_sig_msg: None,
            third_sig_msg: None,
            signature_data: Default::default(),
            args: Default::default(),
            prev_branch: Default::default(),
            post_branch: Default::default()
        }
    }
}

#[derive(Clone, Debug)]
pub struct OperationArguments<E: RescueEngine> {
    pub frs_with_bool: ArgumentsWithSameLength<E, ARGUMENT_WITH_BOOL_CAPACITY>,
    pub frs_with_1_byte: ArgumentsWithSameLength<E, ARGUMENT_WITH_1_BYTE_CAPACITY>,
    pub frs_with_2_bytes: ArgumentsWithSameLength<E, ARGUMENT_WITH_2_BYTES_CAPACITY>,
    pub frs_with_3_bytes: ArgumentsWithSameLength<E, ARGUMENT_WITH_3_BYTES_CAPACITY>,
    pub frs_with_4_bytes: ArgumentsWithSameLength<E, ARGUMENT_WITH_4_BYTES_CAPACITY>,
    pub frs_with_8_bytes: ArgumentsWithSameLength<E, ARGUMENT_WITH_8_BYTES_CAPACITY>,
    pub frs_with_15_bytes: ArgumentsWithSameLength<E, ARGUMENT_WITH_15_BYTES_CAPACITY>,
    pub frs_with_16_bytes: ArgumentsWithSameLength<E, ARGUMENT_WITH_16_BYTES_CAPACITY>,
    pub frs_with_20_bytes: ArgumentsWithSameLength<E, ARGUMENT_WITH_20_BYTES_CAPACITY>,
    pub frs_with_max_bytes: ArgumentsWithSameLength<E, ARGUMENT_WITH_MAX_BYTES_CAPACITY>,

    pub fees_packed: ArgumentsWithSameLength<E, SPECIAL_ARGUMENT_FEE_CAPACITY>,
    pub amounts_packed: ArgumentsWithSameLength<E, SPECIAL_ARGUMENT_AMMOUNT_CAPACITY>,

    pub a: Option<E::Fr>,
    pub b: Option<E::Fr>,
}

impl<E: RescueEngine> OperationArguments<E> {
    pub fn circuit_init() -> Self {
        OperationArguments {
            frs_with_bool: ArgumentsWithSameLength::circuit_init(),
            frs_with_1_byte: ArgumentsWithSameLength::circuit_init(),
            frs_with_2_bytes: ArgumentsWithSameLength::circuit_init(),
            frs_with_3_bytes: ArgumentsWithSameLength::circuit_init(),
            frs_with_4_bytes: ArgumentsWithSameLength::circuit_init(),
            frs_with_8_bytes: ArgumentsWithSameLength::circuit_init(),
            frs_with_15_bytes: ArgumentsWithSameLength::circuit_init(),
            frs_with_16_bytes: ArgumentsWithSameLength::circuit_init(),
            frs_with_20_bytes: ArgumentsWithSameLength::circuit_init(),
            frs_with_max_bytes: ArgumentsWithSameLength::circuit_init(),

            fees_packed: ArgumentsWithSameLength::circuit_init(),
            amounts_packed: ArgumentsWithSameLength::circuit_init(),
            a: Some(E::Fr::zero()),
            b: Some(E::Fr::zero()),
        }
    }
}
impl<E: RescueEngine> Default for OperationArguments<E> {
    fn default() -> Self {
        OperationArguments {
            frs_with_bool: Default::default(),
            frs_with_1_byte: Default::default(),
            frs_with_2_bytes: Default::default(),
            frs_with_3_bytes: Default::default(),
            frs_with_4_bytes: Default::default(),
            frs_with_8_bytes: Default::default(),
            frs_with_15_bytes: Default::default(),
            frs_with_16_bytes: Default::default(),
            frs_with_20_bytes: Default::default(),
            frs_with_max_bytes: Default::default(),

            fees_packed: Default::default(),
            amounts_packed: Default::default(),
            a: Default::default(),
            b: Default::default(),
        }
    }
}

// The maximum length of all elements in the arguments vector is the same.
#[derive(Clone, Debug)]
pub struct ArgumentsWithSameLength<E: RescueEngine, const T: usize>(pub Vec<Option<E::Fr>>);

impl<E: RescueEngine, const T: usize> std::ops::Index<usize> for ArgumentsWithSameLength<E, T>{
    type Output = Option<E::Fr>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<E: RescueEngine, const T: usize> Iterator for ArgumentsWithSameLength<E, T> {
    type Item = Option<E::Fr>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.first().copied()
    }
}

impl<E: RescueEngine, const T: usize> From<Vec<Option<E::Fr>>> for ArgumentsWithSameLength<E, T>{
    fn from(mut args: Vec<Option<E::Fr>>) -> Self {
        assert!(args.len() <= T);
        args.resize(T, Some(E::Fr::zero()));
        Self ( args )
    }
}

impl<E: RescueEngine, const T: usize> Default for ArgumentsWithSameLength<E, T>{
    fn default() -> Self {
        Self(vec![None;T])
    }
}

impl<E: RescueEngine, const T: usize> ArgumentsWithSameLength<E, T>{
    pub fn circuit_init() -> Self {
        Self(vec![Some(E::Fr::zero());T])
    }
}

#[derive(Clone)]
pub struct TransactionSignature<E: JubjubEngine> {
    pub r: edwards::Point<E, Unknown>,
    pub s: E::Fr,
}

impl<E: JubjubEngine> TransactionSignature<E> {
    pub fn empty() -> Self {
        let empty_point: edwards::Point<E, Unknown> = edwards::Point::zero();

        Self {
            r: empty_point,
            s: E::Fr::zero(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SignatureData {
    pub r_packed: Vec<Option<bool>>,
    pub s: Vec<Option<bool>>,
}

impl Default for SignatureData {
    fn default() -> Self {
        Self{
            r_packed: vec![None; FR_BIT_WIDTH_PADDED],
            s: vec![None; FR_BIT_WIDTH_PADDED]
        }
    }
}

impl SignatureData {
    pub fn init_empty() -> Self {
        Self {
            r_packed: vec![Some(false); FR_BIT_WIDTH_PADDED],
            s: vec![Some(false); FR_BIT_WIDTH_PADDED],
        }
    }
}
