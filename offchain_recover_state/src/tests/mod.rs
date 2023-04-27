pub(crate) mod utils;

use crate::{
    driver::RecoverStateDriver,
    storage_interactor::database::DatabaseStorageInteractor,
    storage_interactor::inmemory::InMemoryStorageInteractor,
    tests::utils::{create_log, u32_to_32bytes},
};
use aggegate::agg_op::commit_op::BlocksCommitOperation;
use chrono::Utc;
use ethers::abi::{ethabi, Tokenize};
use ethers::prelude::{Log, Transaction};
use futures::future;
use jsonrpc_core::Params;
use num::BigUint;
use serde_json::{json, Value};
use std::cmp::max;
use std::{collections::HashMap, future::Future};
use zklink_blockchain::eth::contract::{load_abi, ZKLINK_JSON};
use zklink_storage::{
    chain::account::AccountSchema, recover_state::RecoverSchema, StorageProcessor,
};
use zklink_types::{
    block::Block, AccountId, BlockNumber, ChainId, Deposit, DepositOp, ExecutedTx, Nonce,
    SubAccountId, TokenId, Withdraw, WithdrawOp, ZkLinkAddress, ZkLinkOp, H160, H256,
};

fn create_withdraw_operations(account_id: AccountId, to: ZkLinkAddress, amount: u32) -> ExecutedTx {
    let withdraw_op = ZkLinkOp::Withdraw(Box::new(WithdrawOp {
        tx: Withdraw::new(
            account_id,
            SubAccountId(0),
            ChainId(1),
            to,
            TokenId(0),
            TokenId(0),
            amount.into(),
            0u32.into(),
            Nonce(0),
            false,
            0,
            None,
            Default::default(),
        ),
        account_id,
        l1_target_token_after_mapping: Default::default(),
    }));
    ExecutedTx {
        tx: withdraw_op.try_get_tx().unwrap(),
        success: false,
        op: withdraw_op,
        fail_reason: None,
        block_index: None,
        created_at: Utc::now(),
    }
}

fn create_deposit(from: ZkLinkAddress, to: ZkLinkAddress, amount: u32) -> ExecutedTx {
    let deposit_op = ZkLinkOp::Deposit(Box::new(DepositOp {
        tx: Deposit {
            from_chain_id: Default::default(),
            from,
            sub_account_id: Default::default(),
            l1_source_token: Default::default(),
            amount: amount.into(),
            to,
            serial_id: 0,
            l2_target_token: Default::default(),
            eth_hash: Default::default(),
        },
        account_id: AccountId(0),
        l1_source_token_after_mapping: Default::default(),
    }));
    ExecutedTx {
        tx: deposit_op.try_get_tx().unwrap(),
        op: deposit_op,
        fail_reason: None,
        block_index: Some(0),
        created_at: Utc::now(),
        success: false,
    }
}

fn create_block(block_number: BlockNumber, transactions: Vec<ExecutedTx>) -> Block {
    Block::new(
        block_number,
        Default::default(),
        AccountId(0),
        transactions,
        20,
        100,
        1_000_000.into(),
        1_500_000.into(),
        H256::default(),
        H256::default(),
        0,
    )
}

// fn create_transaction_v4(number: u32, stored_block: Block, blocks: Vec<Block>) -> Transaction {
//     let hash: H256 = u32_to_32bytes(number).into();
//     let block_number = blocks
//         .last()
//         .expect("at least one should exist")
//         .block_number
//         .0;
//     let fake_data = [0u8; 4];
//     let mut input_data = vec![];
//     let op = BlocksCommitOperation {
//         last_committed_block: stored_block,
//         blocks,
//     };
//     input_data.extend_from_slice(&fake_data);
//     input_data.extend_from_slice(&ethabi::encode(op.get_eth_tx_args().as_ref()));
//
//     Transaction {
//         hash,
//         nonce: u32_to_32bytes(1).into(),
//         block_hash: Some(u32_to_32bytes(100).into()),
//         block_number: Some(block_number.into()),
//         transaction_index: Some(block_number.into()),
//         from: [5u8; 20].into(),
//         to: Some([7u8; 20].into()),
//         value: u32_to_32bytes(10).into(),
//         gas_price: u32_to_32bytes(1).into(),
//         gas: u32_to_32bytes(1).into(),
//         input: input_data.into(),
//         ..Default::default()
//     }
// }

// fn create_transaction(number: u32, block: Block) -> Transaction {
//     let hash: H256 = u32_to_32bytes(number).into();
//     let root = block.get_eth_encoded_root();
//     let public_data = block.get_eth_public_data();
//     let witness_data = block.get_eth_witness_data();
//     let fake_data = [0u8; 4];
//     let params = (
//         u64::from(*block.block_number),
//         u64::from(*block.fee_account),
//         vec![root],
//         public_data,
//         witness_data.0,
//         witness_data.1,
//     );
//     let mut input_data = vec![];
//     input_data.extend_from_slice(&fake_data);
//     input_data.extend_from_slice(&ethabi::encode(params.into_tokens().as_ref()));
//
//     Transaction {
//         hash,
//         nonce: u32_to_32bytes(1).into(),
//         block_hash: Some(u32_to_32bytes(100).into()),
//         block_number: Some((*block.block_number).into()),
//         transaction_index: Some((*block.block_number).into()),
//         from: [5u8; 20].into(),
//         to: Some([7u8; 20].into()),
//         value: u32_to_32bytes(10).into(),
//         gas_price: u32_to_32bytes(1).into(),
//         gas: u32_to_32bytes(1).into(),
//         input: input_data.into(),
//         ..Default::default()
//     }
// }

#[derive(Debug, Clone)]
pub(crate) struct Web3Transport {
    transactions: HashMap<String, Transaction>,
    logs: HashMap<String, Vec<Log>>,
    last_block: u32,
}

impl Web3Transport {
    fn new() -> Self {
        Self {
            transactions: HashMap::default(),
            logs: HashMap::default(),
            last_block: 0,
        }
    }
    fn push_transactions(&mut self, transactions: Vec<Transaction>) {
        for transaction in transactions {
            self.last_block = max(transaction.block_number.unwrap().as_u32(), self.last_block);
            self.transactions
                .insert(format!("{:?}", &transaction.hash), transaction);
        }
    }

    fn insert_logs(&mut self, topic: String, logs: Vec<Log>) {
        self.logs.insert(topic, logs);
    }

    fn get_logs(&self, filter: Value) -> Vec<Log> {
        let topics = if let Ok(topics) =
            serde_json::from_value::<Vec<Vec<String>>>(filter.get("topics").unwrap().clone())
        {
            topics.first().unwrap().clone()
        } else {
            serde_json::from_value::<Vec<String>>(filter.get("topics").unwrap().clone()).unwrap()
        };
        let mut logs = vec![];

        for topic in &topics {
            if let Some(topic_logs) = self.logs.get(topic) {
                logs.extend_from_slice(topic_logs)
            }
        }

        logs
    }
}
//
// impl Transport for Web3Transport {
//     type Out = dyn Future<Output = Result<Value, web3::Error>> + Send + Unpin;
//
//     fn prepare(
//         &self,
//         method: &str,
//         params: Vec<Value>,
//     ) -> (RequestId, jsonrpc_core::Call) {
//         (
//             1,
//             jsonrpc_core::Call::MethodCall(jsonrpc_core::MethodCall {
//                 jsonrpc: Some(jsonrpc_core::Version::V2),
//                 method: method.to_string(),
//                 params: Params::Array(params),
//                 id: jsonrpc_core::Id::Num(1),
//             }),
//         )
//     }
//
//     fn send(&self, _id: RequestId, request: jsonrpc_core::Call) -> Box<Self::Out> {
//         Box::new(future::ready({
//             if let jsonrpc_core::Call::MethodCall(req) = request {
//                 let mut params = if let Params::Array(params) = req.params {
//                     params
//                 } else {
//                     unreachable!()
//                 };
//                 match req.method.as_str() {
//                     "eth_blockNumber" => Ok(json!("0x80")),
//                     "eth_getLogs" => {
//                         let filter = params.pop().unwrap();
//                         Ok(json!(self.get_logs(filter)))
//                     }
//                     "eth_getTransactionByHash" => {
//                         // TODO Cut `"` from start and end of the string
//                         let hash = &format!("{}", params.pop().unwrap())[1..67];
//                         if let Some(transaction) = self.transactions.get(hash) {
//                             Ok(json!(transaction))
//                         } else {
//                             unreachable!()
//                         }
//                     }
//                     "eth_call" => {
//                         // Now it's call only for one function totalVerifiedBlocks later,
//                         // if it's necessary, add more complex logic for routing
//                         Ok(json!(format!("{:#066x}", self.last_block)))
//                     }
//                     _ => Err(web3::Error::Unreachable),
//                 }
//             } else {
//                 Err(web3::Error::Unreachable)
//             }
//         }))
//     }
// }

// #[ignore]
// #[tokio::test]
// async fn test_with_inmemory_storage() {
//     let contract_addr = H160::from([1u8; 20]);
//     // Start with V3, upgrade it after a couple of blocks to V4.
//     let init_contract_version: u32 = 3;
//     let contract_upgrade_eth_blocks = vec![3];
//
//     let mut transport = Web3Transport::new();
//
//     let mut interactor = InMemoryStorageInteractor::new();
//     let contract = load_abi(ZKLINK_JSON);
//
//     let block_verified_topic = contract
//         .event("BlockVerification")
//         .expect("Main contract abi error")
//         .signature();
//     let block_verified_topic_string = format!("{:?}", block_verified_topic);
//     // Starting from Eth block number 3 the version is upgraded.
//     transport.insert_logs(
//         block_verified_topic_string,
//         vec![
//             create_log(
//                 contract_addr,
//                 block_verified_topic,
//                 vec![u32_to_32bytes(1).into()],
//                 vec![].into(),
//                 1,
//                 u32_to_32bytes(1).into(),
//             ),
//             create_log(
//                 contract_addr,
//                 block_verified_topic,
//                 vec![u32_to_32bytes(2).into()],
//                 vec![].into(),
//                 2,
//                 u32_to_32bytes(2).into(),
//             ),
//             create_log(
//                 contract_addr,
//                 block_verified_topic,
//                 vec![u32_to_32bytes(3).into()],
//                 vec![].into(),
//                 3,
//                 u32_to_32bytes(3).into(),
//             ),
//             create_log(
//                 contract_addr,
//                 block_verified_topic,
//                 vec![u32_to_32bytes(4).into()],
//                 vec![].into(),
//                 4,
//                 u32_to_32bytes(3).into(),
//             ),
//         ],
//     );
//
//     let block_committed_topic = contract
//         .event("BlockCommit")
//         .expect("Main contract abi error")
//         .signature();
//     let block_commit_topic_string = format!("{:?}", block_committed_topic);
//     transport.insert_logs(
//         block_commit_topic_string,
//         vec![
//             create_log(
//                 contract_addr,
//                 block_committed_topic,
//                 vec![u32_to_32bytes(1).into()],
//                 vec![].into(),
//                 1,
//                 u32_to_32bytes(1).into(),
//             ),
//             create_log(
//                 contract_addr,
//                 block_committed_topic,
//                 vec![u32_to_32bytes(2).into()],
//                 vec![].into(),
//                 2,
//                 u32_to_32bytes(2).into(),
//             ),
//             create_log(
//                 contract_addr,
//                 block_committed_topic,
//                 vec![u32_to_32bytes(3).into()],
//                 vec![].into(),
//                 3,
//                 u32_to_32bytes(3).into(),
//             ),
//             create_log(
//                 contract_addr,
//                 block_committed_topic,
//                 vec![u32_to_32bytes(4).into()],
//                 vec![].into(),
//                 4,
//                 u32_to_32bytes(3).into(),
//             ),
//         ],
//     );
//
//     let reverted_topic = contract
//         .event("BlocksRevert")
//         .expect("Main contract abi error")
//         .signature();
//     let _reverted_topic_string = format!("{:?}", reverted_topic);
//
//     let new_token_topic = contract
//         .event("NewToken")
//         .expect("Main contract abi error")
//         .signature();
//     let new_token_topic_string = format!("{:?}", new_token_topic);
//     transport.insert_logs(
//         new_token_topic_string,
//         vec![create_log(
//             contract_addr,
//             new_token_topic,
//             vec![[0; 32].into(), u32_to_32bytes(3).into()],
//             vec![].into(),
//             3,
//             u32_to_32bytes(1).into(),
//         )],
//     );
//
//     transport.push_transactions(vec![
//         create_transaction(
//             1,
//             create_block(
//                 BlockNumber(1),
//                 vec![create_deposit(Default::default(), Default::default(), 50)],
//             ),
//         ),
//         create_transaction(
//             2,
//             create_block(
//                 BlockNumber(2),
//                 vec![create_withdraw_operations(
//                     AccountId(0),
//                     Default::default(),
//                     10,
//                 )],
//             ),
//         ),
//         create_transaction_v4(
//             3,
//             create_block(
//                 BlockNumber(2),
//                 vec![create_deposit(Default::default(), Default::default(), 50)],
//             ),
//             vec![
//                 create_block(
//                     BlockNumber(3),
//                     vec![create_deposit(Default::default(), Default::default(), 50)],
//                 ),
//                 create_block(
//                     BlockNumber(4),
//                     vec![create_withdraw_operations(
//                         AccountId(0),
//                         Default::default(),
//                         10,
//                     )],
//                 ),
//             ],
//         ),
//     ]);
//
//
//     let mut driver = RecoverStateDriver::new(
//         contract_addr,
//         contract_upgrade_eth_blocks.clone(),
//         init_contract_version,
//         init_contract_version,
//         VIEW_BLOCKS_STEP,
//         END_BLOCK_OFFSET,
//         true,
//         None,
//     );
//
//     driver.recover_state(&mut interactor).await;
//
//     // Check that it's stores some account, created by deposit
//     let (_, account) = interactor
//         .get_account_by_address(&Default::default())
//         .unwrap();
//     let balance = account.get_balance(TokenId(0));
//
//     assert_eq!(BigUint::from(80u32), balance);
//     assert_eq!(driver.rollup_events.committed_events.len(), 4);
//     let events = interactor.load_committed_events_state();
//
//     assert_eq!(driver.rollup_events.committed_events.len(), events.len());
//
//     // Nullify the state of driver
//     let mut driver = RecoverStateDriver::new(
//         contract_addr,
//         contract_upgrade_eth_blocks,
//         init_contract_version,
//         VIEW_BLOCKS_STEP,
//         END_BLOCK_OFFSET,
//         true,
//         None,
//         // ZkLinkEvmContract::version4(eth, [1u8; 20].into()),
//     );
//
//     // Load state from db and check it
//     assert!(driver.load_state_from_storage(&mut interactor).await);
//     assert_eq!(driver.rollup_events.committed_events.len(), events.len());
//     assert_eq!(*driver.tree_state.state.block_number, 4)
// }
