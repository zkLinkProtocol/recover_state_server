// External deps
use lazy_static::lazy_static;
// Workspace deps
use crate::franklin_crypto::alt_babyjubjub::AltJubjubBn256;
use crate::franklin_crypto::rescue::bn256::Bn256RescueParams;
use crate::merkle_tree::rescue_hasher::BabyRescueHasher;
use num::BigUint;
use zklink_basic_types::{AccountId, ChainId, Nonce, SlotId, SubAccountId, TokenId};

/// Maximum precision of token amount
pub const TOKEN_MAX_PRECISION: u64 = 18;
/// Order price decimals will be improved accuracy by 10^18
pub fn precision_magnified() -> BigUint {
    BigUint::from(10u8).pow(TOKEN_MAX_PRECISION as u32)
}

/// Maximum number of orders allowed => The width of every sub_account order partition.
pub const MAX_ORDER_NUMBER: usize = usize::pow(2, ORDER_SUB_TREE_DEPTH as u32);
/// Maximum number of chains allowed => The width of every token chain partition.(global asset tree)
pub const MAX_CHAIN_ID: ChainId = ChainId(u8::pow(2, CHAIN_SUB_TREE_DEPTH as u32) - 1);
/// Maximum number of tokens allowed => The width of every sub_account token partition.
pub const MAX_TOKEN_NUMBER: usize = usize::pow(2, BALANCE_SUB_TREE_DEPTH as u32);
pub const MAX_SUB_ACCOUNT_ID: SubAccountId =
    SubAccountId(u8::pow(2, SUB_ACCOUNT_TREE_DEPTH as u32) - 1);

/// Depth of the account tree.
pub const ACCOUNT_TREE_DEPTH: usize = 32;
/// Depth of sub-account tree allowed (be used for multiple different partition dex).
pub const SUB_ACCOUNT_TREE_DEPTH: usize = 5;
/// Depth of the balance subtree for each account.
pub const BALANCE_SUB_TREE_DEPTH: usize = 16;
/// Depth of the orders subtree for each account.
pub const ORDER_SUB_TREE_DEPTH: usize = 16;
/// Depth of the chains subtree for global asset tree(located GLOBAL_ASSET_ACCOUNT_ID's balance tree, sub_account_id => chain_id).
pub const CHAIN_SUB_TREE_DEPTH: usize = SUB_ACCOUNT_TREE_DEPTH;

pub const USED_ACCOUNT_SUBTREE_DEPTH: usize = 24;
pub const MAX_ACCOUNT_ID: AccountId = AccountId(u32::pow(2, USED_ACCOUNT_SUBTREE_DEPTH as u32) - 1);

pub const BALANCE_TREE_DEPTH: usize = SUB_ACCOUNT_TREE_DEPTH + BALANCE_SUB_TREE_DEPTH;
pub const ORDER_TREE_DEPTH: usize = SUB_ACCOUNT_TREE_DEPTH + ORDER_SUB_TREE_DEPTH;

/// max token id supported in circuit
pub const MAX_TOKEN_ID: TokenId = TokenId(u32::pow(2, BALANCE_TREE_DEPTH as u32) - 1);
/// uint16 is used as token id type in Contract, so the max token id can be used is 2^16-1=65535
pub const MAX_REAL_TOKEN_ID: TokenId = TokenId(u32::pow(2, BALANCE_SUB_TREE_DEPTH as u32) - 1);
/// max slot id supported in circuit
pub const MAX_SLOT_ID: SlotId = SlotId(u32::pow(2, ORDER_TREE_DEPTH as u32) - 1);
/// one slot is a leaf of order subtree, slot number = 2 ^ ORDER_SUB_TREE_DEPTH - 1
pub const MAX_REAL_SLOT_ID: SlotId = SlotId(u32::pow(2, ORDER_SUB_TREE_DEPTH as u32) - 1);

/// slot number bit width
pub const SLOT_BIT_WIDTH: usize = 16;
/// Order nonce bit width
pub const ORDER_NONCE_BIT_WIDTH: usize = 24;
pub const ORDER_NONCE_BYTES: usize = ORDER_NONCE_BIT_WIDTH / 8;

/// order tree depth.
pub fn order_tree_depth() -> usize {
    ORDER_SUB_TREE_DEPTH + SUB_ACCOUNT_TREE_DEPTH
}
/// balance tree depth.
pub fn balance_tree_depth() -> usize {
    SUB_ACCOUNT_TREE_DEPTH + BALANCE_SUB_TREE_DEPTH
}
/// account tree depth.
pub fn account_tree_depth() -> usize {
    ACCOUNT_TREE_DEPTH
}

/// Number of supported tokens.
pub fn total_tokens() -> usize {
    2usize.pow(balance_tree_depth() as u32)
}

/// Number of supported slots.
pub fn total_slots() -> usize {
    2usize.pow(order_tree_depth() as u32)
}

/// Depth of the left subtree of the account tree that can be used in the current version of the circuit.
pub fn used_account_subtree_depth() -> usize {
    // total accounts = 2.pow(num) ~ 16mil
    assert!(USED_ACCOUNT_SUBTREE_DEPTH <= account_tree_depth());
    USED_ACCOUNT_SUBTREE_DEPTH
}

/// Max token id, based on the depth of the used left subtree
pub fn max_account_id() -> AccountId {
    let list_count = 2u32.saturating_pow(used_account_subtree_depth() as u32);
    if list_count == u32::MAX {
        AccountId(list_count)
    } else {
        AccountId(list_count - 1)
    }
}

/// Number of tokens that are processed by this release
pub fn number_of_processable_tokens() -> usize {
    //let num = 2048;
    let num = total_tokens();
    assert!(num.is_power_of_two());
    num
}

/// Max token id, based on the number of processable tokens
pub fn max_token_id() -> TokenId {
    TokenId(number_of_processable_tokens() as u32 - 1)
}

pub const CHAIN_ID_BIT_WIDTH: usize = 8;
pub const ACCOUNT_ID_BIT_WIDTH: usize = 32;
pub const SUB_ACCOUNT_ID_BIT_WIDTH: usize = 8;
pub const PRICE_BIT_WIDTH: usize = 120;
pub const MIN_PRICE: u128 = 1;
/// deciamls of price in order will be improved with TOKEN_MAX_PRECISION(18)
/// the bit width of price in pubdata is PRICE_BIT_WIDTH(120)
/// so the max price of price that order can submit is
/// 2 ** 120 - 1 / 10 ^18 = 1329227995784915872
pub const MAX_PRICE: u128 = 1329227995784915872000000000000000000;

pub const INPUT_DATA_ETH_ADDRESS_BYTES_WIDTH: usize = 32;
pub const INPUT_DATA_ETH_UINT_BYTES_WIDTH: usize = 32;
pub const INPUT_DATA_BLOCK_NUMBER_BYTES_WIDTH: usize = 32;
pub const INPUT_DATA_FEE_ACC_BYTES_WIDTH_WITH_EMPTY_OFFSET: usize = 32;
pub const INPUT_DATA_FEE_ACC_BYTES_WIDTH: usize = 3;
pub const INPUT_DATA_ROOT_BYTES_WIDTH: usize = 32;
pub const INPUT_DATA_EMPTY_BYTES_WIDTH: usize = 64;
pub const INPUT_DATA_ROOT_HASH_BYTES_WIDTH: usize = 32;

pub const TOKEN_BIT_WIDTH: usize = 16;
pub const TX_TYPE_BIT_WIDTH: usize = 8;

/// Account subtree hash width
pub const SUBTREE_HASH_WIDTH: usize = 254; //seems to be equal to Bn256::NUM_BITS could be replaced
pub const SUBTREE_HASH_WIDTH_PADDED: usize = 256;

/// balance bit width
pub const BALANCE_BIT_WIDTH: usize = 128;

/// The maximum bit width allowed by multiplication and division
pub const MAX_CALCULATION_BIT_WIDTH: usize = 126;

pub const NEW_PUBKEY_HASH_WIDTH: usize = FR_ADDRESS_LEN * 8;
pub const ADDRESS_WIDTH: usize = FR_ADDRESS_LEN * 8;

/// Nonce bit width
pub const NONCE_BIT_WIDTH: usize = 32;
pub const MAX_NONCE: Nonce = Nonce(u32::MAX);

pub const CHUNK_BIT_WIDTH: usize = CHUNK_BYTES * 8;
pub const CHUNK_BYTES: usize = 19;

pub const MAX_CIRCUIT_MSG_HASH_BITS: usize = 736;

pub const ETH_ADDRESS_BIT_WIDTH: usize = 160;
/// Block number bit width
pub const BLOCK_NUMBER_BIT_WIDTH: usize = 32;

/// Amount bit widths
pub const AMOUNT_BIT_WIDTH: usize = AMOUNT_EXPONENT_BIT_WIDTH + AMOUNT_MANTISSA_BIT_WIDTH;
pub const AMOUNT_EXPONENT_BIT_WIDTH: usize = 5;
pub const AMOUNT_MANTISSA_BIT_WIDTH: usize = 35;

/// Fee bit widths
pub const FEE_BIT_WIDTH: usize = FEE_EXPONENT_BIT_WIDTH + FEE_MANTISSA_BIT_WIDTH;
pub const FEE_EXPONENT_BIT_WIDTH: usize = 5;
pub const FEE_MANTISSA_BIT_WIDTH: usize = 11;

/// Timestamp bit width
pub const TIMESTAMP_BIT_WIDTH: usize = 8 * 8;
pub const SIMP_TIMESTAMP_BIT_WIDTH: usize = 4 * 8;

// Signature data
pub const SIGNATURE_S_BIT_WIDTH: usize = 254;
pub const SIGNATURE_S_BIT_WIDTH_PADDED: usize = 256;
pub const SIGNATURE_R_X_BIT_WIDTH: usize = 254;
pub const SIGNATURE_R_Y_BIT_WIDTH: usize = 254;
pub const SIGNATURE_R_BIT_WIDTH_PADDED: usize = 256;

// Fr element encoding
pub const FR_BIT_WIDTH: usize = 254;
pub const FR_BIT_WIDTH_PADDED: usize = 256;
pub const BN256_MASK: u8 = 0x1f;

pub const LEAF_DATA_BIT_WIDTH: usize =
    NONCE_BIT_WIDTH + NEW_PUBKEY_HASH_WIDTH + ETH_ADDRESS_BIT_WIDTH;

/// Priority op should be executed for this number of eth blocks.
// pub const PRIORITY_EXPIRATION: u64 = 35000;
pub const FR_ADDRESS_LEN: usize = 20;

pub const PAD_MSG_BEFORE_HASH_BITS_LEN: usize = 736;

/// Size of the data that is signed for withdraw tx
pub const SIGNED_WITHDRAW_BIT_WIDTH: usize = TX_TYPE_BIT_WIDTH
    + CHAIN_ID_BIT_WIDTH
    + ACCOUNT_ID_BIT_WIDTH
    + SUB_ACCOUNT_ID_BIT_WIDTH
    + ETH_ADDRESS_BIT_WIDTH
    + 2 * TOKEN_BIT_WIDTH
    + BALANCE_BIT_WIDTH
    + 2 * FEE_EXPONENT_BIT_WIDTH
    + 2 * FEE_MANTISSA_BIT_WIDTH
    + NONCE_BIT_WIDTH
    + 8 // fast withdraw
    + SIMP_TIMESTAMP_BIT_WIDTH;

/// Size of the data that is signed for transfer tx
pub const SIGNED_TRANSFER_BIT_WIDTH: usize = TX_TYPE_BIT_WIDTH
    + ACCOUNT_ID_BIT_WIDTH
    + 2 * SUB_ACCOUNT_ID_BIT_WIDTH
    + ETH_ADDRESS_BIT_WIDTH
    + TOKEN_BIT_WIDTH
    + AMOUNT_EXPONENT_BIT_WIDTH
    + AMOUNT_MANTISSA_BIT_WIDTH
    + FEE_EXPONENT_BIT_WIDTH
    + FEE_MANTISSA_BIT_WIDTH
    + NONCE_BIT_WIDTH
    + SIMP_TIMESTAMP_BIT_WIDTH;

/// Size of the data that is signed for forced exit tx
pub const SIGNED_FORCED_EXIT_BIT_WIDTH: usize = TX_TYPE_BIT_WIDTH
    + CHAIN_ID_BIT_WIDTH
    + 2 * SUB_ACCOUNT_ID_BIT_WIDTH
    + ACCOUNT_ID_BIT_WIDTH
    + ETH_ADDRESS_BIT_WIDTH
    + 3 * TOKEN_BIT_WIDTH
    + FEE_EXPONENT_BIT_WIDTH
    + FEE_MANTISSA_BIT_WIDTH
    + NONCE_BIT_WIDTH
    + SIMP_TIMESTAMP_BIT_WIDTH;

/// Size of the data that is signed for change pubkey tx
pub const SIGNED_CHANGE_PUBKEY_BIT_WIDTH: usize = TX_TYPE_BIT_WIDTH
    + CHAIN_ID_BIT_WIDTH
    + ACCOUNT_ID_BIT_WIDTH
    + SUB_ACCOUNT_ID_BIT_WIDTH
    + NEW_PUBKEY_HASH_WIDTH
    + TOKEN_BIT_WIDTH
    + FEE_EXPONENT_BIT_WIDTH
    + FEE_MANTISSA_BIT_WIDTH
    + NONCE_BIT_WIDTH
    + SIMP_TIMESTAMP_BIT_WIDTH;

/// Size of the data that is signed for order_matching tx
pub const SIGNED_ORDER_MATCHING_BIT_WIDTH: usize = TX_TYPE_BIT_WIDTH
    + ACCOUNT_ID_BIT_WIDTH
    + SUB_ACCOUNT_ID_BIT_WIDTH
    + FR_BIT_WIDTH / 8 * 8
    + TOKEN_BIT_WIDTH
    + FEE_EXPONENT_BIT_WIDTH
    + FEE_MANTISSA_BIT_WIDTH
    + 2 * BALANCE_BIT_WIDTH;

/// Size of the data that is signed for order
pub const SIGNED_ORDER_BIT_WIDTH: usize = TX_TYPE_BIT_WIDTH
    + ACCOUNT_ID_BIT_WIDTH
    + SUB_ACCOUNT_ID_BIT_WIDTH
    + SLOT_BIT_WIDTH
    + ORDER_NONCE_BIT_WIDTH
    + 2 * TOKEN_BIT_WIDTH
    + PRICE_BIT_WIDTH
    + 2 * FEE_RATIO_BIT_WIDTH
    + AMOUNT_BIT_WIDTH
    + 8; // order -> is_sell

pub const ORDERS_BIT_WIDTH: usize = 1424;
pub const ORDERS_BYTES: usize = ORDERS_BIT_WIDTH / 8;

/// Number of inputs in the basic circuit that is aggregated by recursive circuit
pub const RECURSIVE_CIRCUIT_NUM_INPUTS: usize = 1;
/// Depth of the tree which contains different verification keys for basic circuit
pub const RECURSIVE_CIRCUIT_VK_TREE_DEPTH: usize = 4;

/// The number of all ops: NoopOp(0x00)-OrderMatchingOp(0x08)
pub const ALL_DIFFERENT_TRANSACTIONS_TYPE_NUMBER: usize = 9;
/// The number of ops compositions of circuits containing all op's.
pub const EXEC_ALL_OPS_COMPOSITION_NUMBER: usize =
    2usize.pow(ALL_DIFFERENT_TRANSACTIONS_TYPE_NUMBER as u32) - 1;

/// The gas token contract address of multi chains that interacts with zklink protocol.
pub const GAS_TOKEN_CONTRACT_ADDRESS: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";

/// 0 can not be used as token id
pub const TOKEN_ID_ZERO: u32 = 0;
pub const USD_TOKEN_ID: u32 = 1;
pub const USD_TOKEN_BIT: usize = 5;
pub const USD_SYMBOL: &str = "USD";
pub const USDX_TOKEN_ID_LOWER_BOUND: u32 = USD_TOKEN_ID + 1;
pub const USDX_TOKEN_ID_UPPER_BOUND: u32 = 16;
pub const USDX_TOKEN_ID_RANGE: u32 = USDX_TOKEN_ID_UPPER_BOUND - USDX_TOKEN_ID_LOWER_BOUND + 1;
pub const MAX_USD_TOKEN_ID: u32 = USDX_TOKEN_ID_UPPER_BOUND + USDX_TOKEN_ID_RANGE;

/// jump tokens related USD(1-31) and zkl(32)
pub fn calc_gas_token_by_chain_id(chain_id: ChainId) -> TokenId {
    TokenId(MAX_USD_TOKEN_ID + 1 + *chain_id as u32)
}

/// Test usd token[17-31]
pub fn is_usd_token(token_id: &TokenId) -> bool {
    token_id.0 > USDX_TOKEN_ID_UPPER_BOUND && token_id.0 <= MAX_USD_TOKEN_ID
}

/// USD_X = X - 15
pub fn get_usd_mapping_token(token_id: &TokenId) -> TokenId {
    TokenId(token_id.0 - USDX_TOKEN_ID_RANGE)
}

/// Special account id
/// The account that is used to charge fees
pub const FEE_ACCOUNT_ID: AccountId = AccountId(0);
/// The account used to store the remaining assets of the tokens for contracts of layer1.
/// The token balances of this account are used in withdraw to layer one or create exit proof.
///
/// There are two kind of accounts:
/// * Normal account(id = \[0, 2-MAX_ACCOUNT_ID\])
/// * Global asset account(id = 1)
///
/// Tokens stored in normal account are: USD(1), USD stable coins(\[17,31\]), other coins(\[32,..\]).
/// Tokens stored in global account are: USD_X(\[2,16\]), USD stable coins(\[17,31\]), other coins(\[32,..\]).
///
/// **NOTE** the differenc of tokens stored in these two kind of accounts
///
/// The sub account id is used to represent chain id of zklink in GLOBAL_ASSET_ACCOUNT.
///
/// **NOTE** MAX_CHAIN_ID <= MAX_SUB_ACCOUNT_ID
///
/// For example
///
/// Alice deposit 100 USDC in Ethereum(which chain id is 1) with no mapping, account tree is:
/// account_id, sub_account_id, token_id, balance
/// Alice,0,USDC,100
/// Global,1,USDC,100
///
/// And then bob deposit 50 USDT in BSC(which chain id is 2) with mapping, account tree update to:
/// account_id, sub_account_id, token_id, balance
/// Alice,0,USDC,100
/// Global,1,USDC,100
/// Bob,0,USD,50
/// Global,2,USD_USDT,50
///
/// **NOTE** if bob select to receive USD(use token mapping), and Global's USD_X token will be update
///
/// When withdraw USD to a special mapping token to a layer one chain, we can not exceed the amount stored in Global account
/// of the mapping token of that chain.
///
/// Continue to use the above example:
/// * Can bob withdraw 50 USD to USDC in ETH? no, because Global account's USD_USDC balance of ETH is 0.
/// * Can bob withdraw 60 USD to USDT in BSC? no, because Global account's USD_USDT balance of BSC is 50.
/// * Can bob withdraw 30 USD to USDT in BSC? yes
/// * Can alice withdraw 100 USDC in ETH? yes, because Global account's USDC balance of ETH is 100.
pub const GLOBAL_ASSET_ACCOUNT_ID: AccountId = AccountId(1);
/// As the black hole address of the global asset account, no one can control.
pub const GLOBAL_ASSET_ACCOUNT_ADDR: &str = "0xffffffffffffffffffffffffffffffffffffffff";

/// Special sub_account id
/// The subaccount is used to collect the fees to which subaccount of the fee_account.
pub const MAIN_SUB_ACCOUNT_ID: SubAccountId = SubAccountId(0);

/// All fee related values
pub const FEE_RATIO_BIT_WIDTH: usize = 8;
pub const FEE_DENOMINATOR: usize = 10usize.pow(FEE_PRECISION as u32);
pub const FEE_PRECISION: u64 = 4;

lazy_static! {
    pub static ref JUBJUB_PARAMS: AltJubjubBn256 = AltJubjubBn256::new();
    pub static ref RESCUE_PARAMS: Bn256RescueParams = Bn256RescueParams::new_checked_2_into_1();
    pub static ref RESCUE_HASHER: BabyRescueHasher = BabyRescueHasher::default();
}
