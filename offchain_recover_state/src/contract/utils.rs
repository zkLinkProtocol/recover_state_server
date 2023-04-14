use crate::contract::TransactionInfo;
use ethers::abi::ethabi;
use ethers::abi::Abi;
use ethers::prelude::{Address, Bytes, Transaction, U256};
use ethers::prelude::{Http, Provider};
use reqwest::{Client, Url};
use std::str::FromStr;
use zklink_crypto::params::{
    INPUT_DATA_ETH_ADDRESS_BYTES_WIDTH, INPUT_DATA_ETH_UINT_BYTES_WIDTH,
    INPUT_DATA_ROOT_HASH_BYTES_WIDTH,
};
use zklink_types::{Account, ZkLinkAddress, ZkLinkOp};

pub const ZKLINK_JSON: &str = include_str!("ZkLink.json");

#[derive(Debug, PartialEq, Eq, ethers::prelude::EthEvent)]
#[ethevent(name = "NewToken")]
pub struct NewToken {
    #[ethevent(indexed, name = "tokenId")]
    pub id: u16,
    #[ethevent(indexed, name = "token")]
    pub address: Address,
}

/// NewPriorityRequest defined in Events.sol
#[derive(Debug, PartialEq, Eq, ethers::prelude::EthEvent)]
#[ethevent(name = "NewPriorityRequest")]
pub struct NewPriorityRequest {
    pub sender: Address,
    pub serial_id: u64,
    pub op_type: u8,
    pub pub_data: Bytes,
    pub expiration_block: U256,
}

pub fn load_abi(json_file_content: &str) -> Abi {
    let abi_string = serde_json::Value::from_str(json_file_content)
        .unwrap()
        .get("abi")
        .unwrap()
        .to_string();
    serde_json::from_str(&abi_string).unwrap()
}

pub fn new_provider_with_url(url: &str) -> Provider<Http> {
    let http_client = Client::builder()
        .connect_timeout(std::time::Duration::from_secs(30))
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build http client!");
    let url = Url::parse(url).unwrap();
    Provider::new(Http::new_with_client(url, http_client))
}

/// Returns Rollup genesis (fees) account from the input of the Rollup contract creation transaction
///
/// Constructor parameters:
/// ```solidity
/// constructor(
///     Verifier _verifierTarget, ZkLink _zkLinkTarget, address _periphery, uint32 _blockNumber,
///     uint256 _timestamp, bytes32 _stateHash, bytes32 _commitment, bytes32 _syncHash,
///     address _firstValidator, address _governor, address _feeAccountAddress
/// )
/// ```
pub fn get_genesis_account(genesis_transaction: &Transaction) -> anyhow::Result<Account> {
    const ENCODED_INIT_PARAMETERS_WIDTH: usize = 6 * INPUT_DATA_ETH_ADDRESS_BYTES_WIDTH
        + INPUT_DATA_ROOT_HASH_BYTES_WIDTH * 3
        + 2 * INPUT_DATA_ETH_UINT_BYTES_WIDTH;

    let input_data = genesis_transaction.input_data()?;

    // Input for contract constructor contains the bytecode of the contract and
    // encoded arguments after it.
    // We are not interested in the bytecode and we know the size of arguments,
    // so we can simply cut the parameters bytes from the end of input array,
    // and then decode them to access required data.
    let encoded_init_parameters =
        input_data[input_data.len() - ENCODED_INIT_PARAMETERS_WIDTH..].to_vec();

    let init_parameters_types = vec![
        ethabi::ParamType::Address,   // Verifier contract address
        ethabi::ParamType::Address,   // zkLink contract address
        ethabi::ParamType::Address,   // periphery address
        ethabi::ParamType::Uint(32),  // block number
        ethabi::ParamType::Uint(256), // timestamp
        ethabi::ParamType::FixedBytes(INPUT_DATA_ROOT_HASH_BYTES_WIDTH), // state hash
        ethabi::ParamType::FixedBytes(INPUT_DATA_ROOT_HASH_BYTES_WIDTH), // commitment
        ethabi::ParamType::FixedBytes(INPUT_DATA_ROOT_HASH_BYTES_WIDTH), // syncHash
        ethabi::ParamType::Address,   // First validator (committer) address
        ethabi::ParamType::Address,   // Governor address
        ethabi::ParamType::Address,   // Fee account address
    ];
    let fee_account_address_argument_id = 10;

    let mut decoded_init_parameters = ethabi::decode(
        init_parameters_types.as_slice(),
        encoded_init_parameters.as_slice(),
    )
    .map_err(|_| {
        anyhow::Error::from(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "can't get decoded init parameters from contract creation transaction",
        )))
    })?;
    let fee_account_address = decoded_init_parameters.remove(fee_account_address_argument_id);
    let account = fee_account_address.into_address().map(|address| {
        Account::default_with_address(&ZkLinkAddress::from_slice(&address.0).unwrap())
    });
    account
        .ok_or(Err("Invalid token in parameters"))
        .map_err(|_: Result<Account, _>| {
            anyhow::Error::from(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "can't get decoded init parameter from contract creation transaction",
            )))
        })
}

/// Attempts to restore block operations from the public data.
/// Should be used for contracts V1-V5.
pub fn get_rollup_ops_from_data(data: &[u8]) -> Result<Vec<ZkLinkOp>, anyhow::Error> {
    parse_pub_data(
        data,
        ZkLinkOp::from_public_data,
        ZkLinkOp::public_data_length,
    )
}

pub(super) fn parse_pub_data<Parse, GetSize>(
    data: &[u8],
    parse: Parse,
    get_data_size: GetSize,
) -> Result<Vec<ZkLinkOp>, anyhow::Error>
where
    Parse: Fn(&[u8]) -> anyhow::Result<ZkLinkOp>,
    GetSize: Fn(u8) -> anyhow::Result<usize>,
{
    let mut current_pointer = 0;
    let mut ops = Vec::new();
    while current_pointer < data.len() {
        let op_type: u8 = data[current_pointer];

        let pub_data_size = get_data_size(op_type)?;

        let pre = current_pointer;
        let post = pre + pub_data_size;

        let op = parse(&data[pre..post])?;

        ops.push(op);
        current_pointer += pub_data_size;
    }
    Ok(ops)
}

#[cfg(test)]
mod test {
    use super::get_rollup_ops_from_data;
    use zklink_types::{
        AccountId, ChainId, ChangePubKey, ChangePubKeyOp, Deposit, DepositOp, FullExit, FullExitOp,
        Nonce, PubKeyHash, SubAccountId, TokenId, Transfer, TransferOp, TransferToNewOp, Withdraw,
        WithdrawOp, ZkLinkOp,
    };

    #[test]
    fn test_deposit() {
        let priority_op = Deposit {
            from_chain_id: Default::default(),
            from: "1111111111111111111111111111111111111111".parse().unwrap(),
            sub_account_id: Default::default(),
            l1_source_token: Default::default(),
            amount: 10u32.into(),
            to: "7777777777777777777777777777777777777777".parse().unwrap(),
            serial_id: 0,
            l2_target_token: Default::default(),
            eth_hash: Default::default(),
        };
        let op1 = ZkLinkOp::Deposit(Box::new(DepositOp {
            tx: Deposit {
                from_chain_id: Default::default(),
                from: Default::default(),
                sub_account_id: Default::default(),
                l1_source_token: Default::default(),
                l2_target_token: Default::default(),
                amount: Default::default(),
                to: Default::default(),
                serial_id: 0,
                eth_hash: Default::default(),
            },
            account_id: AccountId(6),
            l1_source_token_after_mapping: Default::default(),
        }));
        let pub_data1 = op1.public_data();
        let op2 = get_rollup_ops_from_data(&pub_data1)
            .expect("cant get ops from data")
            .pop()
            .expect("empty ops array");
        let pub_data2 = op2.public_data();
        assert_eq!(pub_data1, pub_data2);
    }

    #[test]
    fn test_part_exit() {
        let tx = Withdraw::new(
            AccountId(3),
            SubAccountId(3),
            ChainId(1),
            vec![9u8; 20].into(),
            TokenId(1),
            TokenId(1),
            20u32.into(),
            10u32.into(),
            Nonce(2),
            false,
            10,
            None,
            Default::default(),
        );
        let op1 = ZkLinkOp::Withdraw(Box::new(WithdrawOp {
            tx,
            account_id: AccountId(3),
            l1_target_token_after_mapping: Default::default(),
        }));
        let pub_data1 = op1.public_data();
        let op2 = get_rollup_ops_from_data(&pub_data1)
            .expect("cant get ops from data")
            .pop()
            .expect("empty ops array");
        let pub_data2 = op2.public_data();
        assert_eq!(pub_data1, pub_data2);
    }

    #[test]
    fn test_successfull_full_exit() {
        let priority_op = FullExit {
            to_chain_id: Default::default(),
            account_id: AccountId(11),
            sub_account_id: Default::default(),
            exit_address: vec![9u8; 20].into(),
            l2_source_token: Default::default(),
            l1_target_token: Default::default(),
            serial_id: 0,
            eth_hash: Default::default(),
        };
        let op1 = ZkLinkOp::FullExit(Box::new(FullExitOp {
            tx: priority_op,
            exit_amount: Default::default(),
            l1_target_token_after_mapping: Default::default(),
        }));
        let pub_data1 = op1.public_data();
        let op2 = get_rollup_ops_from_data(&pub_data1)
            .expect("cant get ops from data")
            .pop()
            .expect("empty ops array");
        let pub_data2 = op2.public_data();
        assert_eq!(pub_data1, pub_data2);
    }

    #[test]
    fn test_failed_full_exit() {
        let priority_op = FullExit {
            to_chain_id: Default::default(),
            account_id: AccountId(11),
            sub_account_id: Default::default(),
            l2_source_token: TokenId(1),
            l1_target_token: TokenId(1),
            serial_id: 0,
            exit_address: vec![9u8; 20].into(),
            eth_hash: Default::default(),
        };
        let op1 = ZkLinkOp::FullExit(Box::new(FullExitOp {
            tx: priority_op,
            exit_amount: Default::default(),
            l1_target_token_after_mapping: Default::default(),
        }));
        let pub_data1 = op1.public_data();
        let op2 = get_rollup_ops_from_data(&pub_data1)
            .expect("cant get ops from data")
            .pop()
            .expect("empty ops array");
        let pub_data2 = op2.public_data();
        assert_eq!(pub_data1, pub_data2);
    }

    #[test]
    fn test_transfer_to_new() {
        let tx = Transfer::new(
            AccountId(11),
            "7777777777777777777777777777777777777777".parse().unwrap(),
            SubAccountId(1),
            SubAccountId(2),
            TokenId(1),
            20u32.into(),
            20u32.into(),
            Nonce(3),
            None,
            Default::default(),
        );
        let op1 = ZkLinkOp::TransferToNew(Box::new(TransferToNewOp {
            tx,
            from: AccountId(11),
            to: AccountId(12),
        }));
        let pub_data1 = op1.public_data();
        let op2 = get_rollup_ops_from_data(&pub_data1)
            .expect("cant get ops from data")
            .pop()
            .expect("empty ops array");
        let pub_data2 = op2.public_data();
        assert_eq!(pub_data1, pub_data2);
    }

    #[test]
    fn test_transfer() {
        let tx = Transfer::new(
            AccountId(11),
            "7777777777777777777777777777777777777777".parse().unwrap(),
            SubAccountId(1),
            SubAccountId(2),
            TokenId(1),
            20u32.into(),
            20u32.into(),
            Nonce(3),
            None,
            Default::default(),
        );
        let op1 = ZkLinkOp::Transfer(Box::new(TransferOp {
            tx,
            from: AccountId(11),
            to: AccountId(12),
        }));
        let pub_data1 = op1.public_data();
        let op2 = get_rollup_ops_from_data(&pub_data1)
            .expect("cant get ops from data")
            .pop()
            .expect("empty ops array");
        let pub_data2 = op2.public_data();
        assert_eq!(pub_data1, pub_data2);
    }

    #[test]
    fn test_change_pubkey_offchain() {
        let tx = ChangePubKey::new(
            ChainId(1),
            AccountId(11),
            SubAccountId(1),
            PubKeyHash::from_hex("sync:0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f").unwrap(),
            TokenId(0),
            Default::default(),
            Nonce(3),
            None,
            None,
            Default::default(),
        );
        let op1 = ZkLinkOp::ChangePubKeyOffchain(Box::new(ChangePubKeyOp {
            tx,
            account_id: AccountId(11),
            address: Default::default(),
        }));
        let pub_data1 = op1.public_data();
        let op2 = get_rollup_ops_from_data(&pub_data1)
            .expect("cant get ops from data")
            .pop()
            .expect("empty ops array");
        let pub_data2 = op2.public_data();
        assert_eq!(pub_data1, pub_data2);
    }
}
