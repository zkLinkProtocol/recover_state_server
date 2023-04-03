use crate::{exit_circuit::*, utils::*, account::AccountContent};
use crate::exit_circuit::allocate_merkle_root;
use crate::witness::OperationBranch;

pub struct AllocatedOperationBranch<E: RescueEngine> {
    pub account: AccountContent<E>,
    pub account_id: CircuitElement<E>,
    pub sub_account_id: CircuitElement<E>,
    pub account_audit_path: Vec<AllocatedNum<E>>,

    pub token: CircuitElement<E>,
    pub actual_token: CircuitElement<E>,
    pub balance: CircuitElement<E>,
    pub balance_audit_path: Vec<AllocatedNum<E>>,

    pub slot_number: CircuitElement<E>,
    pub actual_slot: CircuitElement<E>,
    pub order: AllocatedOrder<E>,
    pub order_audit_path: Vec<AllocatedNum<E>>
}

pub struct AllocatedOrder<E: RescueEngine>{
    pub nonce: CircuitElement<E>,
    pub residue: CircuitElement<E>,
}

impl<E:RescueEngine> std::fmt::Debug for AllocatedOrder<E>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AllocatedOrder")
            .field("nonce", &self.nonce.get_number().get_value())
            .field("residue", &self.residue.get_number().get_value())
            .finish()
    }
}

impl<E: RescueEngine> AllocatedOperationBranch<E> {
    pub fn from_witness<CS: ConstraintSystem<E>>(
        mut cs: CS,
        operation_branch: &OperationBranch<E>,
    ) -> Result<AllocatedOperationBranch<E>, SynthesisError> {
        // account leaf and merkle path
        let account = AccountContent::from_witness(
            cs.namespace(|| "allocate account_content"),
            &operation_branch.witness.account_witness,
        )?;
        let account_id = CircuitElement::from_fe_with_known_length(
            cs.namespace(|| "account_address"),
            || Ok(operation_branch.account_id.grab()?),
            account_tree_depth(),
        )?.pad(ACCOUNT_ID_BIT_WIDTH);
        let sub_account_id = CircuitElement::from_fe_with_known_length(
            cs.namespace(|| "sub_account_address"),
            || Ok(operation_branch.sub_account_id.grab()?),
            SUB_ACCOUNT_ID_BIT_WIDTH,
        )?;
        let account_audit_path = allocate_numbers_vec(
            cs.namespace(|| "account_audit_path"),
            &operation_branch.witness.account_path,
        )?;
        assert_eq!(account_audit_path.len(), account_tree_depth());

        // account balance leaf and merkle path
        let balance = CircuitElement::from_fe_with_known_length(
            cs.namespace(|| "balance"),
            || Ok(operation_branch.witness.balance_value.grab()?),
            BALANCE_BIT_WIDTH,
        )?;
        let token = CircuitElement::from_fe_with_known_length(
            cs.namespace(|| "token"),
            || Ok(operation_branch.token.grab()?),
            TOKEN_BIT_WIDTH,
        )?;
        let balance_audit_path = allocate_numbers_vec(
            cs.namespace(|| "balance_audit_path"),
            &operation_branch.witness.balance_subtree_path,
        )?;
        assert_eq!(balance_audit_path.len(), balance_tree_depth());

        // account order slots leaf and merkle path
        let order_nonce = CircuitElement::from_fe_with_known_length(
            cs.namespace(|| "order nonce"),
            || Ok(operation_branch.witness.order_nonce.grab()?),
            ORDER_NONCE_BIT_WIDTH,
        )?;
        let order_residue = CircuitElement::from_fe_with_known_length(
            cs.namespace(|| "order residue"),
            || Ok(operation_branch.witness.order_residue.grab()?),
            BALANCE_BIT_WIDTH,
        )?;
        let slot_number = CircuitElement::from_fe_with_known_length(
            cs.namespace(|| "slot id"),
            || Ok(operation_branch.slot_number.grab()?),
            SLOT_BIT_WIDTH,
        )?;
        let order_audit_path = allocate_numbers_vec(
            cs.namespace(|| "order_audit_path"),
            &operation_branch.witness.order_subtree_path,
        )?;
        let actual_token = Self::calculate_actual_token(
            cs.namespace(||"calculate actual token"),
            &token,
            &sub_account_id
        )?;
        let actual_slot = Self::calculate_actual_slot(
            cs.namespace(||"calculate actual slot"),
            &slot_number,
            &sub_account_id
        )?;
        assert_eq!(order_audit_path.len(), order_tree_depth());

        Ok(AllocatedOperationBranch {
            account,
            account_audit_path,
            account_id,
            balance,
            balance_audit_path,
            token,
            actual_token,
            slot_number,
            actual_slot,
            order: AllocatedOrder{
                nonce: order_nonce,
                residue: order_residue,
            },
            order_audit_path,
            sub_account_id
        })
    }

    pub fn calculate_actual_slot_id<CS: ConstraintSystem<E>>(
        mut cs: CS,
        slot: &AllocatedNum<E>,
        sub_account_id: &CircuitElement<E>,
    ) -> Result<AllocatedNum<E>, SynthesisError>{
        let max_num = AllocatedNum::alloc_constant(
            cs.namespace(||"max_token_number"),
            || Ok(E::Fr::from_u64(MAX_ORDER_NUMBER as u64))
        )?;
        sub_account_id.get_number().mul(
            cs.namespace(|| "calculate actual slot id: sub_account_id * MAX_ORDER_NUMBER"),
            &max_num
        )?.add(
            cs.namespace(||"calculate actual slot id: sub_account_id * MAX_ORDER_NUMBER + slot"),
            &slot
        )
    }

    pub fn calculate_actual_token_id<CS: ConstraintSystem<E>>(
        mut cs: CS,
        token: &AllocatedNum<E>,
        sub_account_id: &CircuitElement<E>,
    ) -> Result<AllocatedNum<E>, SynthesisError>{
        // token_id + sub_account_id * MAX_TOKEN_NUMBER
        let max_num = AllocatedNum::alloc_constant(
            cs.namespace(||"max_token_number"),
            || Ok(E::Fr::from_u64(MAX_TOKEN_NUMBER as u64))
        )?;
        sub_account_id.get_number().mul(
            cs.namespace(|| "sub_account_id * MAX_TOKEN_NUMBER"),
            &max_num
        )?.add(
            cs.namespace(||"token_id + sub_account_id * MAX_TOKEN_NUMBER"),
            &token
        )
    }

    pub fn calculate_actual_token<CS: ConstraintSystem<E>>(
        mut cs: CS,
        token: &CircuitElement<E>,
        sub_account_id: &CircuitElement<E>,
    ) -> Result<CircuitElement<E>, SynthesisError> {
        let actual_token = Self::calculate_actual_token_id(
            cs.namespace(||"calculate_actual_token"),
            token.get_number(),
            sub_account_id,
        )?;
        CircuitElement::from_number_with_known_length(
            cs.namespace(||"token_number into bits"),
            actual_token,
            balance_tree_depth()
        )
    }

    pub fn calculate_actual_slot<CS: ConstraintSystem<E>>(
        mut cs: CS,
        slot_number: &CircuitElement<E>,
        sub_account_id: &CircuitElement<E>,
    ) -> Result<CircuitElement<E>, SynthesisError> {
        let actual_slot = Self::calculate_actual_slot_id(
            cs.namespace(||"calculate_actual_slot"),
            slot_number.get_number(),
            sub_account_id,
        )?;
        CircuitElement::from_number_with_known_length(
            cs.namespace(||"order_number into bits"),
            actual_slot,
            order_tree_depth()
        )
    }

    pub fn calculate_balance_tree_root<CS: ConstraintSystem<E>>(&self, mut cs: CS, params: &E::Params) -> Result<CircuitElement<E>, SynthesisError> {
        let balance_root = allocate_merkle_root(
            cs.namespace(|| "balance subtree root"),
            self.balance.get_bits_le(),
            self.actual_token.get_bits_le(),
            &self.balance_audit_path,
            balance_tree_depth(),
            params,
        )?;
        let balance_subtree_root = CircuitElement::from_number(cs.namespace(|| "balance_subtree_root_ce"), balance_root)?;
        let state_tree_root = calc_account_state_tree_root(
            cs.namespace(|| "state tree root"),
            &balance_subtree_root,
            params,
        )?;
        Ok(state_tree_root)
    }

    pub fn calculate_order_tree_root<CS: ConstraintSystem<E>>(&self, mut cs: CS, params: &E::Params) -> Result<Vec<Boolean>, SynthesisError> {
        let mut order_data = Vec::with_capacity(NONCE_BIT_WIDTH + BALANCE_BIT_WIDTH + FR_BIT_WIDTH);
        order_data.extend_from_slice(self.order.nonce.get_bits_le());
        order_data.extend_from_slice(self.order.residue.get_bits_le());
        let order_root = allocate_merkle_root(
            cs.namespace(|| "order root"),
            &order_data,
            self.actual_slot.get_bits_le(),
            &self.order_audit_path,
            order_tree_depth(),
            params,
        )?;
        let order_subtree_root = CircuitElement::from_number(cs.namespace(|| "order_subtree_root_ce"), order_root)?;
        let order_tree_root = calc_account_state_tree_root(
            cs.namespace(|| "order tree root"),
            &order_subtree_root,
            params,
        )?;
        Ok(order_tree_root.into_padded_le_bits(FR_BIT_WIDTH_PADDED))
    }
}

impl<E:RescueEngine> std::fmt::Debug for AllocatedOperationBranch<E>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AllocatedOperationBranch")
            .field("account", &self.account)
            .field("balance", &self.balance.get_number().get_value())
            .field("account_id", &self.account_id.get_number().get_value())
            .field("sub_account_id", &self.sub_account_id.get_number().get_value())
            .field("token", &self.token.get_number().get_value())
            .field("actual_token", &self.actual_token.get_number().get_value())
            .field("slot_number", &self.slot_number.get_number().get_value())
            .field("actual_slot", &self.actual_slot.get_number().get_value())
            .field("order", &self.order)
            .finish()
    }
}


/// Account tree state will be extended in the future, so for current balance tree we
/// append emtpy hash to reserve place for the future tree before hashing.
pub fn calc_account_state_tree_root<E: RescueEngine, CS: ConstraintSystem<E>>(
    mut cs: CS,
    balance_root: &CircuitElement<E>,
    params: &E::Params,
) -> Result<CircuitElement<E>, SynthesisError> {
    let state_tree_root_input = balance_root.get_number().clone();
    let empty_root_padding =
        AllocatedNum::zero(cs.namespace(|| "allocate zero element for padding"))?;

    let mut sponge_output = rescue::rescue_hash(
        cs.namespace(|| "hash state root and balance root"),
        &[state_tree_root_input, empty_root_padding],
        params,
    )?;

    assert_eq!(sponge_output.len(), 1);
    let state_tree_root = sponge_output.pop().expect("must get a single element");

    CircuitElement::from_number(cs.namespace(|| "total_subtree_root_ce"), state_tree_root)
}