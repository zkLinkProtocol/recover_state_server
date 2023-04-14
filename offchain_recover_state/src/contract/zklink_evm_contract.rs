use crate::contract::utils::{get_genesis_account, load_abi, new_provider_with_url, ZKLINK_JSON};
use crate::contract::{
    BlockChain, LogInfo, TransactionInfo, ZkLinkContract, ZkLinkContractVersion,
};
use anyhow::{ensure, format_err};
use async_trait::async_trait;
use ethers::abi::Address;
use ethers::contract::Contract;
use ethers::core::types::BlockNumber as EthBlockNumber;
use ethers::prelude::{Filter, Http, Log, Middleware, Provider, Transaction};
use recover_state_config::Layer1Config;
use zklink_types::{Account, BlockNumber, ChainId, H256};

const FUNC_NAME_HASH_LENGTH: usize = 4;

#[derive(Debug)]
pub struct ZkLinkEvmContract {
    pub chain_id: ChainId,
    pub config: Layer1Config,
    pub contract: Contract<Provider<Http>>,
    pub version: ZkLinkContractVersion,
}

impl ZkLinkEvmContract {
    pub fn new(config: Layer1Config) -> ZkLinkEvmContract {
        let abi = load_abi(ZKLINK_JSON);
        let client = new_provider_with_url(&config.client.web3_url());
        let contract_address = Address::from_slice(config.contract.address.as_bytes());
        ZkLinkEvmContract {
            chain_id: config.chain.chain_id,
            contract: Contract::new(contract_address, abi, client.into()),
            config,
            version: ZkLinkContractVersion::V0,
        }
    }
}

#[async_trait]
impl BlockChain for ZkLinkEvmContract {
    type Log = Log;
    type Transaction = Transaction;

    fn layer1_chain_id(&self) -> u32 {
        self.config.client.chain_id
    }

    fn layer2_chain_id(&self) -> ChainId {
        self.chain_id
    }

    async fn block_number(&self) -> anyhow::Result<u64> {
        let block_number = self.contract.client().get_block_number().await?.as_u64();
        Ok(block_number)
    }
}

#[async_trait]
impl ZkLinkContract for ZkLinkEvmContract {
    fn get_event_signature(&self, name: &str) -> H256 {
        self.contract
            .abi()
            .event(name)
            .expect("Main contract abi error")
            .signature()
    }

    fn get_genesis_account(&self, genesis_tx: Self::Transaction) -> anyhow::Result<Account> {
        get_genesis_account(&genesis_tx)
    }

    async fn get_transaction(&self, hash: H256) -> anyhow::Result<Option<Self::Transaction>> {
        let tx = self.contract.client().get_transaction(hash).await?;
        Ok(tx)
    }

    async fn get_total_verified_blocks(&self) -> anyhow::Result<u32> {
        Ok(self
            .contract
            .method("totalBlocksExecuted", ())?
            .call()
            .await?)
    }

    async fn get_block_logs(
        &self,
        from: BlockNumber,
        to: BlockNumber,
    ) -> anyhow::Result<Vec<Self::Log>> {
        let topics: Vec<H256> = vec![
            self.get_event_signature("BlockCommit"),
            self.get_event_signature("BlockExecuted"),
            self.get_event_signature("BlocksRevert"),
        ];
        let filter = Filter::default()
            .address(vec![self.contract.address()])
            .from_block(EthBlockNumber::Number((*from).into()))
            .to_block(EthBlockNumber::Number((*to).into()))
            .topic0(topics);
        let result = self
            .contract
            .client()
            .get_logs(&filter)
            .await
            .map_err(|e| format_err!("Get logs: {}", e))?;

        Ok(result)
    }

    async fn get_gatekeeper_logs(&self) -> anyhow::Result<Vec<Log>> {
        let upgrade_contract_event = self
            .contract
            .abi()
            .event("UpgradeComplete")
            .expect("Upgrade Gatekeeper contract abi error")
            .signature();

        let filter = Filter::default()
            .address(vec![self.contract.address()])
            .from_block(EthBlockNumber::Earliest)
            .to_block(EthBlockNumber::Latest)
            .events(vec![upgrade_contract_event]);

        let result = self
            .contract
            .client()
            .get_logs(&filter)
            .await
            .map_err(|e| format_err!("Get logs: {}", e))?;
        Ok(result)
    }
}

impl TransactionInfo for Transaction {
    fn input_data(&self) -> anyhow::Result<Vec<u8>> {
        let input_data = self.input.0.clone();
        ensure!(
            input_data.len() > FUNC_NAME_HASH_LENGTH,
            format_err!("No commitment data in tx")
        );
        Ok(input_data[FUNC_NAME_HASH_LENGTH..].to_vec())
    }

    fn transaction_hash(&self) -> H256 {
        self.hash
    }

    fn block_number(&self) -> Option<u64> {
        self.block_number.map(|num| num.as_u64())
    }
}

impl LogInfo for Log {
    fn topics(&self) -> Vec<H256> {
        self.topics.clone()
    }

    fn data(&self) -> Vec<u8> {
        self.data.to_vec()
    }

    fn transaction_hash(&self) -> H256 {
        self.transaction_hash.unwrap()
    }

    fn block_number(&self) -> Option<u64> {
        self.block_number.map(|num| num.as_u64())
    }
}
