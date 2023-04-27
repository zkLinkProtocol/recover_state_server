use crate::{circuit::*, operation::*, utils::*, account::AccountContent};

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
            || operation_branch.account_id.grab(),
            account_tree_depth(),
        )?.pad(ACCOUNT_ID_BIT_WIDTH);
        let sub_account_id = CircuitElement::from_fe_with_known_length(
            cs.namespace(|| "sub_account_address"),
            || operation_branch.sub_account_id.grab(),
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
            || operation_branch.witness.balance_value.grab(),
            BALANCE_BIT_WIDTH,
        )?;
        let token = CircuitElement::from_fe_with_known_length(
            cs.namespace(|| "token"),
            || operation_branch.token.grab(),
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
            || operation_branch.witness.order_nonce.grab(),
            ORDER_NONCE_BIT_WIDTH,
        )?;
        let order_residue = CircuitElement::from_fe_with_known_length(
            cs.namespace(|| "order residue"),
            || operation_branch.witness.order_residue.grab(),
            BALANCE_BIT_WIDTH,
        )?;
        let slot_number = CircuitElement::from_fe_with_known_length(
            cs.namespace(|| "slot id"),
            || operation_branch.slot_number.grab(),
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
            slot
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
            token
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

pub struct AllocatedChunkData<E: Engine> {
    pub is_chunk_last: Boolean,
    pub is_chunk_first: Boolean,
    pub chunk_number: AllocatedNum<E>,
    pub tx_type: CircuitElement<E>,
}

#[derive(Clone)]
pub struct AllocatedOperationData<E: Engine> {
    pub ces_with_bool: Vec<CircuitElement<E>>,
    pub ces_with_1_byte: Vec<CircuitElement<E>>,
    pub ces_with_2_bytes: Vec<CircuitElement<E>>,
    pub ces_with_3_bytes: Vec<CircuitElement<E>>,
    pub ces_with_4_bytes: Vec<CircuitElement<E>>,
    pub ces_with_8_bytes: Vec<CircuitElement<E>>,
    pub ces_with_15_bytes: Vec<CircuitElement<E>>,
    pub ces_with_16_bytes: Vec<CircuitElement<E>>,
    pub ces_with_20_bytes: Vec<CircuitElement<E>>,
    pub ces_with_max_bytes: Vec<CircuitElement<E>>,

    pub fee_packed_ces: Vec<CircuitElement<E>>,
    pub fee_unpacked_ces: Vec<CircuitElement<E>>,
    pub amount_packed_ces: Vec<CircuitElement<E>>,
    pub amount_unpacked_ces: Vec<CircuitElement<E>>,

    pub first_sig_msg: CircuitElement<E>,
    pub second_sig_msg: CircuitElement<E>,
    pub third_sig_msg: CircuitElement<E>,

    pub a: CircuitElement<E>,
    pub b: CircuitElement<E>,
}

pub enum CommonArgs {
    L2Token,
    L1Token,

    ValidFrom,
    ValidUntil,
}

impl<E:Engine> std::ops::Index<CommonArgs> for AllocatedOperationData<E>{
    type Output = CircuitElement<E>;

    fn index(&self, index: CommonArgs) -> &Self::Output {
        match index{
            CommonArgs::L2Token => &self.ces_with_2_bytes[0],
            CommonArgs::L1Token => &self.ces_with_2_bytes[1],

            CommonArgs::ValidFrom => &self.ces_with_8_bytes[0],
            CommonArgs::ValidUntil => &self.ces_with_8_bytes[1],
        }
    }
}

impl<E: RescueEngine> AllocatedOperationData<E> {
    pub fn empty_from_zero(zero_element: AllocatedNum<E>) -> Result<Self, SynthesisError> {
        let ce_with_bool = CircuitElement::unsafe_empty_of_some_length(zero_element.clone(), 1);
        let ce_with_1_byte = CircuitElement::unsafe_empty_of_some_length(zero_element.clone(), 8);
        let ce_with_2_bytes = CircuitElement::unsafe_empty_of_some_length(zero_element.clone(), 2 * 8);
        let ce_with_3_bytes = CircuitElement::unsafe_empty_of_some_length(zero_element.clone(), 3 * 8);
        let ce_with_4_bytes = CircuitElement::unsafe_empty_of_some_length(zero_element.clone(), 4 * 8);
        let ce_with_5_bytes = CircuitElement::unsafe_empty_of_some_length(zero_element.clone(), 5 * 8);
        let ce_with_8_bytes = CircuitElement::unsafe_empty_of_some_length(zero_element.clone(), 8 * 8);
        let ce_with_15_bytes = CircuitElement::unsafe_empty_of_some_length(zero_element.clone(), 15 * 8);
        let ce_with_16_bytes = CircuitElement::unsafe_empty_of_some_length(zero_element.clone(), 16 * 8);
        let ce_with_20_bytes = CircuitElement::unsafe_empty_of_some_length(zero_element.clone(), 20 * 8);
        let ce_with_max_bytes = CircuitElement::unsafe_empty_of_some_length(zero_element.clone(), FR_BIT_WIDTH);

        let first_sig_msg = CircuitElement::unsafe_empty_of_some_length(zero_element.clone(), E::Fr::CAPACITY as usize);
        let second_sig_msg = CircuitElement::unsafe_empty_of_some_length(zero_element.clone(), E::Fr::CAPACITY as usize);
        let third_sig_msg = CircuitElement::unsafe_empty_of_some_length(
            zero_element,
            MAX_CIRCUIT_MSG_HASH_BITS - (2 * E::Fr::CAPACITY as usize), //TODO: think of more consistent constant flow (ZKS-54).
        );

        Ok(AllocatedOperationData {
            a: ce_with_16_bytes.clone(),
            b: ce_with_16_bytes.clone(),
            ces_with_bool: vec![ce_with_bool;ARGUMENT_WITH_BOOL_CAPACITY],
            ces_with_1_byte: vec![ce_with_1_byte; ARGUMENT_WITH_1_BYTE_CAPACITY],
            ces_with_2_bytes: vec![ce_with_2_bytes.clone(); ARGUMENT_WITH_2_BYTES_CAPACITY],
            ces_with_3_bytes: vec![ce_with_3_bytes; ARGUMENT_WITH_3_BYTES_CAPACITY],
            ces_with_4_bytes: vec![ce_with_4_bytes; ARGUMENT_WITH_4_BYTES_CAPACITY],
            ces_with_8_bytes: vec![ce_with_8_bytes; ARGUMENT_WITH_8_BYTES_CAPACITY],
            ces_with_15_bytes: vec![ce_with_15_bytes; ARGUMENT_WITH_15_BYTES_CAPACITY],
            ces_with_16_bytes: vec![ce_with_16_bytes.clone(); ARGUMENT_WITH_16_BYTES_CAPACITY],
            ces_with_20_bytes: vec![ce_with_20_bytes; ARGUMENT_WITH_20_BYTES_CAPACITY],
            ces_with_max_bytes: vec![ce_with_max_bytes; ARGUMENT_WITH_MAX_BYTES_CAPACITY],
            fee_packed_ces: vec![ce_with_2_bytes; SPECIAL_ARGUMENT_FEE_CAPACITY],
            fee_unpacked_ces: vec![ce_with_16_bytes.clone(); SPECIAL_ARGUMENT_FEE_CAPACITY],
            amount_packed_ces: vec![ce_with_5_bytes; SPECIAL_ARGUMENT_AMMOUNT_CAPACITY],
            amount_unpacked_ces: vec![ce_with_16_bytes; SPECIAL_ARGUMENT_AMMOUNT_CAPACITY],
            first_sig_msg,
            second_sig_msg,
            third_sig_msg,
        })
    }

    fn convert_amounts<CS: ConstraintSystem<E>>(
        mut cs: CS,
        amount: Option<E::Fr>,
        is_amount: bool
    ) -> Result<(CircuitElement<E>, CircuitElement<E>), SynthesisError> {
        let amount_packed = CircuitElement::from_fe_with_known_length(
            cs.namespace(|| "amount_packed"),
            || amount.grab(),
            if is_amount{ AMOUNT_EXPONENT_BIT_WIDTH + AMOUNT_MANTISSA_BIT_WIDTH }
            else { FEE_EXPONENT_BIT_WIDTH + FEE_MANTISSA_BIT_WIDTH },
        )?;
        let amount_parsed = parse_with_exponent_le(
            cs.namespace(|| "parse amount"),
            amount_packed.get_bits_le(),
            if is_amount {AMOUNT_EXPONENT_BIT_WIDTH} else { FEE_EXPONENT_BIT_WIDTH},
            if is_amount {AMOUNT_MANTISSA_BIT_WIDTH} else { FEE_MANTISSA_BIT_WIDTH},
            10,
        )?;
        let amount_unpacked = CircuitElement::from_number_with_known_length(
            cs.namespace(|| "amount_unpacked"),
            amount_parsed,
            BALANCE_BIT_WIDTH,
        )?;
        Ok((amount_packed, amount_unpacked))
    }

    fn frs_convert_ces<CS:ConstraintSystem<E>>(
        mut cs: CS,
        frs: &[Option<E::Fr>],
        bits_length: usize
    ) -> Result<Vec<CircuitElement<E>>, SynthesisError>{
        frs.iter()
            .enumerate()
            .map(|(idx, witness)| {
                CircuitElement::from_fe_with_known_length(
                    cs.namespace(|| format!("allocate {}th {}-bits CircuitElement", idx, bits_length)),
                    || witness.grab(),
                    bits_length,
                )
            })
            .collect::<Result<Vec<_>, _>>()
    }

    pub fn from_witness<CS: ConstraintSystem<E>>(
        mut cs: CS,
        op: &OperationUnit<E>,
    ) -> Result<AllocatedOperationData<E>, SynthesisError> {
        let ces_with_bool = Self::frs_convert_ces(cs.namespace(||"bool"), &op.args.frs_with_bool.0, 1)?;
        let ces_with_1_byte = Self::frs_convert_ces(cs.namespace(||"1_byte"), &op.args.frs_with_1_byte.0, 8)?;
        let ces_with_2_bytes = Self::frs_convert_ces(cs.namespace(||"2_bytes"), &op.args.frs_with_2_bytes.0, 2*8)?;
        let ces_with_3_bytes = Self::frs_convert_ces(cs.namespace(||"3_bytes"), &op.args.frs_with_3_bytes.0, 3*8)?;
        let ces_with_4_bytes = Self::frs_convert_ces(cs.namespace(||"4_bytes"), &op.args.frs_with_4_bytes.0, 4*8)?;
        let ces_with_8_bytes = Self::frs_convert_ces(cs.namespace(||"8_bytes"), &op.args.frs_with_8_bytes.0, 8*8)?;
        let ces_with_15_bytes = Self::frs_convert_ces(cs.namespace(||"15_bytes"), &op.args.frs_with_15_bytes.0, 15*8)?;
        let ces_with_16_bytes = Self::frs_convert_ces(cs.namespace(||"16_bytes"), &op.args.frs_with_16_bytes.0, 16*8)?;
        let ces_with_20_bytes = Self::frs_convert_ces(cs.namespace(||"20_bytes"), &op.args.frs_with_20_bytes.0, 20*8)?;
        let ces_with_max_bytes = Self::frs_convert_ces(cs.namespace(||"max_bytes"), &op.args.frs_with_max_bytes.0, FR_BIT_WIDTH)?;
        let (amounts_packed, amounts_unpacked) = op
            .args
            .amounts_packed
            .0
            .iter()
            .enumerate()
            .map(|(idx, &amount)| {
                Self::convert_amounts(
                    cs.namespace(|| format!("amount with index {}", idx)),
                    amount,
                    true
                )
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .unzip();
        let (fees_packed, fees_unpacked) = op
            .args
            .fees_packed
            .0
            .iter()
            .enumerate()
            .map(|(idx, &fee)| {
                Self::convert_amounts(
                    cs.namespace(|| format!("fee with index {}", idx)),
                    fee,
                    false
                )
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .unzip();
        let first_sig_msg = CircuitElement::from_fe_with_known_length(
            cs.namespace(|| "first_part_signature_message"),
            || op.first_sig_msg.grab(),
            E::Fr::CAPACITY as usize,
        )?;

        let second_sig_msg = CircuitElement::from_fe_with_known_length(
            cs.namespace(|| "second_part_signature_message"),
            || op.second_sig_msg.grab(),
            E::Fr::CAPACITY as usize, //TODO: think of more consistent constant flow (ZKS-54).
        )?;

        let third_sig_msg = CircuitElement::from_fe_with_known_length(
            cs.namespace(|| "third_part_signature_message"),
            || op.third_sig_msg.grab(),
            MAX_CIRCUIT_MSG_HASH_BITS - (2 * E::Fr::CAPACITY as usize), //TODO: think of more consistent constant flow (ZKS-54).
        )?;
        let a = CircuitElement::from_fe_with_known_length(
            cs.namespace(|| "a"),
            || op.args.a.grab(),
            BALANCE_BIT_WIDTH,
        )?;
        let b = CircuitElement::from_fe_with_known_length(
            cs.namespace(|| "b"),
            || op.args.b.grab(),
            BALANCE_BIT_WIDTH,
        )?;

        Ok(AllocatedOperationData {
            ces_with_bool,
            ces_with_1_byte,
            ces_with_2_bytes,
            ces_with_3_bytes,
            ces_with_4_bytes,
            ces_with_8_bytes,
            ces_with_15_bytes,
            ces_with_16_bytes,
            ces_with_20_bytes,
            ces_with_max_bytes,
            fee_packed_ces: fees_packed,
            fee_unpacked_ces: fees_unpacked,
            amount_packed_ces: amounts_packed,
            amount_unpacked_ces: amounts_unpacked,
            first_sig_msg,
            second_sig_msg,
            third_sig_msg,
            a,
            b
        })
    }
}
