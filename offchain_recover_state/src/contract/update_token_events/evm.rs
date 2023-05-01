use super::UpdateTokenEvents;
use crate::contract::utils::{
    load_abi, new_provider_with_url, NewPriorityRequest, NewToken, ZKLINK_JSON,
};
use crate::contract::LogInfo;
use crate::storage_interactor::DatabaseStorageInteractor;
use crate::storage_interactor::StorageInteractor;
use anyhow::format_err;
use async_trait::async_trait;
use ethers::contract::Contract;
use ethers::core::types::BlockNumber as EthBlockNumber;
use ethers::prelude::{parse_log, Address, Filter, Http, Log, Middleware, Provider, H256};
use recover_state_config::Layer1Config;
use tracing::info;
use zklink_storage::chain::operations::records::StoredSubmitTransaction;
use zklink_storage::ConnectionPool;
use zklink_types::{ChainId, PriorityOp, ZkLinkAddress, ZkLinkPriorityOp};

pub const ERC20_JSON: &str = include_str!("ERC20.json");
const VIEW_STEP_BLOCK: u64 = 1000;

pub struct EvmTokenEvents {
    connection_pool: ConnectionPool,
    erc20_abi: ethers::abi::Abi,
    contract: Contract<Provider<Http>>,

    chain_id: ChainId,
    gas_token: String,
    view_block_step: u64,
    last_sync_block_number: u64,
    last_sync_serial_id: i64,
}

impl EvmTokenEvents {
    pub async fn new(
        config: &Layer1Config,
        connection_pool: ConnectionPool,
    ) -> Self {
        let (last_watched_block_number, last_sync_serial_id) = {
            let mut storage = connection_pool.access_storage().await.unwrap();
            storage
                .recover_schema()
                .last_watched_block_number(*config.chain.chain_id as i16, "token")
                .await
                .expect("Failed to get last watched block number")
                .unwrap_or((config.contract.deployment_block as i64, -1))
        };
        let address = Address::from_slice(config.contract.address.as_bytes());
        let zklink_abi = load_abi(ZKLINK_JSON);
        let erc20_abi = load_abi(ERC20_JSON);
        let client = new_provider_with_url(&config.client.web3_url());

        Self {
            erc20_abi,
            contract: Contract::new(address, zklink_abi, client.into()),
            chain_id: config.chain.chain_id,
            gas_token: config.chain.gas_token.clone(),
            view_block_step: VIEW_STEP_BLOCK,
            last_sync_block_number: last_watched_block_number as u64,
            last_sync_serial_id,
            connection_pool,
        }
    }

    fn get_event_signature(&self, name: &str) -> H256 {
        self.contract
            .abi()
            .event(name)
            .expect("Main contract abi error")
            .signature()
    }

    fn process_priority_ops(
        &self,
        last_serial_id: i64,
        ops: Vec<PriorityOp>,
    ) -> anyhow::Result<(i64, Vec<StoredSubmitTransaction>)> {
        // Check whether serial_id of the received events is correct and converted to StoredSubmitTransaction.
        if let Some(PriorityOp { serial_id, .. }) = ops.last() {
            let mut priority_txs = Vec::with_capacity(ops.len());

            assert!(last_serial_id + 1 >= 0, "Invalid last_serial_id");
            let mut cur_serial_id = (last_serial_id + 1) as u64;

            for PriorityOp {
                data, serial_id, ..
            } in ops.iter()
            {
                assert_eq!(cur_serial_id, *serial_id);

                // Parsed into StoredSubmitTransaction
                let tx = match data {
                    ZkLinkPriorityOp::Deposit(deposit) => deposit.into(),
                    ZkLinkPriorityOp::FullExit(full_exit) => full_exit.into(),
                };
                priority_txs.push(tx);

                // update progress
                cur_serial_id += 1;
            }

            Ok((*serial_id as i64, priority_txs))
        } else {
            Ok((last_serial_id, vec![]))
        }
    }

    fn process_priority_op_events(&self, logs: Vec<Log>) -> anyhow::Result<Vec<PriorityOp>> {
        logs.into_iter()
            .map(|log| {
                let block_number = log.block_number.unwrap().as_u64();
                let transaction_hash = log.transaction_hash.unwrap();
                let event: NewPriorityRequest =
                    parse_log(log).map_err(|e| format_err!("Parse priority log error: {:?}", e))?;
                let sender = ZkLinkAddress::from_slice(event.sender.as_bytes()).unwrap();
                Ok(PriorityOp {
                    serial_id: event.serial_id,
                    data: ZkLinkPriorityOp::parse_from_priority_queue_logs(
                        event.pub_data.as_ref(),
                        event.op_type,
                        sender,
                        event.serial_id,
                        transaction_hash,
                    )
                    .unwrap(),
                    deadline_block: event.expiration_block.as_u64(),
                    eth_block: block_number,
                })
            })
            .collect::<anyhow::Result<Vec<_>>>()
    }

    async fn process_token_logs(
        &self,
        logs: Vec<Log>,
    ) -> anyhow::Result<(Vec<NewToken>, Vec<String>)> {
        let token_events = logs
            .into_iter()
            .map(|log| {
                parse_log::<NewToken>(log)
                    .map_err(|e| format_err!("Parse token log error: {:?}", e))
            })
            .collect::<anyhow::Result<Vec<NewToken>>>()?;
        let mut token_symbols = Vec::with_capacity(token_events.len());
        for token in &token_events {
            let address = Address::from_slice(token.address.as_bytes());
            let symbol = if address
                == "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
                    .parse()
                    .unwrap()
            {
                self.gas_token.clone()
            } else {
                Contract::new(address, self.erc20_abi.clone(), self.contract.client())
                    .method("symbol", ())?
                    .call()
                    .await?
            };
            info!(
                "Loading token from [{:?}] layer1: {symbol}, address: {address}",
                self.chain_id
            );
            token_symbols.push(symbol);
        }
        Ok((token_events, token_symbols))
    }
}

#[async_trait]
impl UpdateTokenEvents for EvmTokenEvents {
    fn reached_latest_block(&self, latest_block: u64) -> bool {
        self.last_sync_block_number + self.view_block_step > latest_block
    }

    async fn block_number(&self) -> anyhow::Result<u64> {
        let block_number = self.contract.client().get_block_number().await?.as_u64();
        Ok(block_number)
    }

    async fn update_token_events(&mut self) -> anyhow::Result<u64> {
        let from = self.last_sync_block_number + 1;
        let to = self.last_sync_block_number + self.view_block_step;
        let topics: Vec<H256> = vec![
            self.get_event_signature("NewToken"),
            self.get_event_signature("NewPriorityRequest"),
        ];
        let filter = Filter::default()
            .address(vec![self.contract.address()])
            .from_block(EthBlockNumber::Number(from.into()))
            .to_block(EthBlockNumber::Number(to.into()))
            .topic0(topics.clone());
        let logs = self
            .contract
            .client()
            .get_logs(&filter)
            .await
            .map_err(|e| format_err!("Get logs: {}", e))?;

        let mut token_logs = Vec::new();
        let mut priority_logs = Vec::new();
        for log in logs {
            if topics[0] == log.topics()[0] {
                token_logs.push(log);
            } else if topics[1] == log.topics()[0] {
                priority_logs.push(log);
            } else {
                panic!("Not exist topic");
            }
        }

        // priority txs
        let ops = self.process_priority_op_events(priority_logs)?;
        let (last_serial_id, submit_ops) =
            self.process_priority_ops(self.last_sync_serial_id, ops)?;
        // registered tokens
        let (token_events, symbols) = self.process_token_logs(token_logs).await?;

        // updated storage
        let storage = self.connection_pool.access_storage_with_retry().await;
        let mut interactor = DatabaseStorageInteractor::new(storage);
        interactor
            .update_priority_ops_and_tokens(
                self.chain_id,
                to,
                last_serial_id,
                submit_ops,
                token_events,
                symbols,
            )
            .await;

        // update cache
        self.last_sync_block_number = to;
        self.last_sync_serial_id = last_serial_id;
        Ok(to)
    }
}
