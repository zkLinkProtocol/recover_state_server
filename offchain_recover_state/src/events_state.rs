use std::cmp::min;
// External deps
use anyhow::format_err;
use std::convert::TryFrom;
// Workspace deps
use zklink_types::{BlockNumber, U256};
// Local deps
use crate::contract::{
    BlockChain, LogInfo, TransactionInfo, ZkLinkContract, ZkLinkContractVersion,
};
use crate::events::{BlockEvent, EventType};

/// Rollup contract events states description
#[derive(Debug, Clone)]
pub struct RollUpEvents {
    /// Committed operations blocks events
    pub committed_events: Vec<BlockEvent>,
    /// Verified operations blocks events
    pub verified_events: Vec<BlockEvent>,
    /// Last watched layer1 block number
    pub last_watched_block_number: u64,
}

impl Default for RollUpEvents {
    /// Create default Rollup contract events state
    fn default() -> Self {
        Self {
            committed_events: Vec::new(),
            verified_events: Vec::new(),
            last_watched_block_number: 0,
        }
    }
}

impl RollUpEvents {
    /// Saves the genesis block number as the last watched number
    /// Returns the genesis block number
    ///
    /// # Arguments
    ///
    /// * `genesis_transaction` - Genesis transaction description
    ///
    pub fn set_last_watched_block_number<T: ZkLinkContract>(
        &mut self,
        genesis_transaction: &T::Transaction,
    ) -> Result<u64, anyhow::Error> {
        let genesis_block_number = genesis_transaction
            .block_number()
            .ok_or_else(|| format_err!("No block number info in tx"))?;
        self.last_watched_block_number = genesis_block_number;
        Ok(genesis_block_number)
    }

    /// Update past events state from last watched layer1 block with delta between last layer1 block and last watched block.
    /// Returns new verified committed blocks evens, the last watched block number.
    ///
    /// # Arguments
    ///
    /// * `zklink_contract` - Rollup contract
    /// * `contract_upgraded_blocks` - layer1 blocks that include correct UpgradeComplete events
    /// * `view_blocks_step` - Blocks step for watching
    /// * `end_block_offset` - Delta between last block and last watched block
    /// * `init_contract_version` - The initial version of the deployed zkLink contract
    ///
    #[allow(clippy::too_many_arguments)]
    pub async fn update_block_events<T: ZkLinkContract>(
        &mut self,
        zklink_contract: &T,
        contract_upgraded_blocks: &[u64],
        view_blocks_step: u64,
        end_block_offset: u64,
        init_contract_version: u32,
    ) -> Result<(Vec<BlockEvent>, u64), anyhow::Error> {
        self.remove_verified_events();

        let (block_events, to_block_number) = Self::get_block_events_and_last_watched_block(
            zklink_contract,
            self.last_watched_block_number,
            view_blocks_step,
            end_block_offset,
        )
        .await?;
        // Parse the initial contract version.
        let init_contract_version = ZkLinkContractVersion::try_from(init_contract_version)
            .expect("invalid initial contract version provided");
        // Pass Layer1 block numbers that correspond to `UpgradeComplete`
        // events emitted by the Upgrade GateKeeper. Should be provided by the config.
        self.last_watched_block_number = to_block_number;
        self.update_blocks_state(
            zklink_contract,
            &block_events,
            contract_upgraded_blocks,
            init_contract_version,
        );

        let mut events_to_return = self.committed_events.clone();
        events_to_return.extend(self.verified_events.clone());

        Ok((events_to_return, self.last_watched_block_number))
    }

    /// Returns blocks logs, added token logs and the new last watched block number
    ///
    /// # Arguments
    ///
    /// * `zklink_contract` - Rollup contract
    /// * `governance_contract` - Governance contract
    /// * `last_watched_block_number` - the current last watched block
    /// * `view_blocks_step` - view layer1 blocks delta step
    /// * `end_block_offset` - the delta of the last block
    ///
    async fn get_block_events_and_last_watched_block<T: ZkLinkContract>(
        zklink_contract: &T,
        last_watched_block_number: u64,
        view_blocks_step: u64,
        end_block_offset: u64,
    ) -> anyhow::Result<(Vec<<T as BlockChain>::Log>, u64)> {
        let latest_block_minus_delta = zklink_contract.block_number().await? - end_block_offset;
        if latest_block_minus_delta == last_watched_block_number {
            return Ok((vec![], last_watched_block_number)); // No new layer1 blocks
        }

        let from_block_number = last_watched_block_number + 1;
        let to_block_number = min(
            from_block_number + view_blocks_step,
            latest_block_minus_delta,
        );

        let block_logs = zklink_contract
            .get_block_logs(from_block_number.into(), to_block_number.into())
            .await?;

        Ok((block_logs, to_block_number))
    }

    /// Updates committed and verified blocks state by extending their arrays
    /// Returns flag that indicates if there are any logs
    ///
    /// # Arguments
    ///
    /// * `contract` - Specified contract
    /// * `logs` - Block events with their info
    /// * `contract_upgrade_eth_blocks` - Layer1 blocks that correspond to emitted `UpgradeComplete` events
    /// * `init_contract_version` - The initial version of the deployed zkLink contract
    fn update_blocks_state<T: ZkLinkContract>(
        &mut self,
        contract: &T,
        logs: &[<T as BlockChain>::Log],
        contract_upgraded_blocks: &[u64],
        init_contract_version: ZkLinkContractVersion,
    ) {
        if logs.is_empty() {
            return;
        }

        let block_verified_topic = contract.get_event_signature("BlockExecuted");
        let block_committed_topic = contract.get_event_signature("BlockCommit");
        let reverted_topic = contract.get_event_signature("BlocksRevert");

        for log in logs {
            let topic = log.topics()[0];

            // Because the layer1 contract design(The layer2 block number recorded by the block log is not sequential) is currently useless
            // Remove reverted committed blocks first
            if topic == reverted_topic {
                const U256_SIZE: usize = 32;
                // Fields in `BlocksRevert` are not `indexed`, thus they're located in `data`.
                let data = log.data();
                assert_eq!(data.len(), U256_SIZE * 2);

                let total_executed = U256::from_big_endian(&data[..U256_SIZE]).as_u32();
                let total_committed = U256::from_big_endian(&data[U256_SIZE..]).as_u32();

                self.committed_events
                    .retain(|bl| bl.block_num <= total_committed.into());
                self.verified_events
                    .retain(|bl| bl.block_num <= total_executed.into());

                continue;
            }

            // Go into new blocks
            let transaction_hash = log.transaction_hash();
            // Restore the number of contract upgrades using layer1 block numbers.
            let log_block_number = log
                .block_number()
                .expect("no Layer1 block number for block log");

            let num = contract_upgraded_blocks
                .iter()
                .filter(|&block| log_block_number >= *block)
                .count();
            let contract_version = init_contract_version.upgrade(num as u32);
            let layer2_block_number = log.topics()[1];

            let mut block = BlockEvent {
                block_num: BlockNumber(U256::from(layer2_block_number.as_bytes()).as_u32()),
                transaction_hash,
                block_type: EventType::Committed,
                contract_version,
            };
            if topic == block_verified_topic {
                block.block_type = EventType::Verified;
                self.verified_events.push(block);
            } else if topic == block_committed_topic {
                self.committed_events.push(block);
            }
        }
    }

    /// Removes verified committed blocks events and all verified
    fn remove_verified_events(&mut self) {
        let count_to_remove = self.verified_events.len();
        self.verified_events.clear();
        self.committed_events.drain(0..count_to_remove);
    }

    /// Returns only verified committed blocks from verified
    pub fn get_only_verified_committed_events(&self) -> Vec<BlockEvent> {
        let count_to_get = self.verified_events.len();
        self.committed_events[0..count_to_get].to_vec()
    }
}

#[cfg(test)]
mod test {
    use super::RollUpEvents;
    use ethers::prelude::Bytes;
    use zklink_types::H160;

    use crate::contract::{ZkLinkContractVersion, ZkLinkEvmContract};
    use crate::tests::utils::{create_log, u32_to_32bytes};

    #[test]
    fn event_state() {
        let mut events_state = RollUpEvents::default();

        let contract = ZkLinkEvmContract::new(Default::default());
        let contract_addr = H160::from([1u8; 20]);

        let block_verified_topic = contract
            .contract
            .abi()
            .event("BlockVerification")
            .expect("Main contract abi error")
            .signature();
        let block_committed_topic = contract
            .contract
            .abi()
            .event("BlockCommit")
            .expect("Main contract abi error")
            .signature();
        let reverted_topic = contract
            .contract
            .abi()
            .event("BlocksRevert")
            .expect("Main contract abi error")
            .signature();

        let mut logs = vec![];
        for i in 0..32 {
            logs.push(create_log(
                contract_addr,
                block_committed_topic,
                vec![u32_to_32bytes(i).into()],
                Bytes(vec![].into()),
                i,
                u32_to_32bytes(i).into(),
            ));
            logs.push(create_log(
                contract_addr,
                block_verified_topic,
                vec![u32_to_32bytes(i).into()],
                Bytes(vec![].into()),
                i,
                u32_to_32bytes(i).into(),
            ));
        }

        let v0 = ZkLinkContractVersion::V0;
        let upgrade_blocks = Vec::new();
        events_state.update_blocks_state(&contract, &logs, &upgrade_blocks, v0);
        assert_eq!(events_state.committed_events.len(), 32);
        assert_eq!(events_state.verified_events.len(), 32);

        let last_block_ver = u32_to_32bytes(15);
        let last_block_com = u32_to_32bytes(10);
        let mut data = vec![];
        data.extend(&last_block_com);
        data.extend(&last_block_ver);
        let log = create_log(
            contract_addr,
            reverted_topic,
            vec![u32_to_32bytes(3).into()],
            Bytes(data.into()),
            3,
            u32_to_32bytes(1).into(),
        );
        events_state.update_blocks_state(&contract, &[log], &upgrade_blocks, v0);
        assert_eq!(events_state.committed_events.len(), 16);
        assert_eq!(events_state.verified_events.len(), 11);
    }
}
