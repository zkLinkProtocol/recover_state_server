use std::sync::Arc;
use std::time::Duration;
use moka::future::Cache;
use zklink_crypto::proof::EncodedSingleProof;
use zklink_types::ZkLinkAddress;
use zklink_utils::BigUintSerdeWrapper;
use zklink_prover::{ExitInfo, ExitProofData};
use zklink_storage::chain::account::records::StorageAccount;
use zklink_storage::ConnectionPool;
use crate::response::ExodusStatus;

const PROOFS_CACHE_SIZE: u64 = 1000;
const PROVING_TIME: u64 = 120; // parallel single proof generated time

#[derive(Clone)]
pub struct ProofsCache{
    conn_pool: ConnectionPool,
    pub cache: Arc<Cache<ExitInfo, Option<(BigUintSerdeWrapper, EncodedSingleProof)>>>
}

impl ProofsCache {
    pub async fn new(conn_pool: ConnectionPool) -> Self {
        let mut storage = conn_pool.access_storage()
            .await
            .expect("Failed to acquire access for ProofsCache initialization");
        let stored_exit_proofs = storage.prover_schema()
            .get_stored_exit_proofs(PROOFS_CACHE_SIZE as i64)
            .await
            .expect("Failed to get stored exit proof for ProofsCache initialization");
        let proofs_cache = Cache::builder()
            .max_capacity(PROOFS_CACHE_SIZE)
            .time_to_live(Duration::from_secs(PROVING_TIME))// for updating proof
            .time_to_idle(Duration::from_secs(60))
            .build();
        for stored_proof in stored_exit_proofs {
            let StorageAccount{address, ..} = storage.chain()
                .account_schema()
                .account_by_id(stored_proof.account_id)
                .await
                .expect("Failed to get account by id")
                .expect("Account must be existing");
            let ExitProofData{
                mut exit_info,
                amount,
                proof
            } = stored_proof.into();
            exit_info.account_address = ZkLinkAddress::from_slice(address.as_slice()).unwrap();
            proofs_cache.insert(
                exit_info,
                amount.and_then(|a|proof.map(|p|(a, p)))
            ).await;
        }
        drop(storage);

        ProofsCache { conn_pool, cache: Arc::new(proofs_cache) }
    }

    pub async fn get_proof(
        &self,
        exit_info: ExitInfo,
    ) -> Result<ExitProofData, ExodusStatus> {
        if let Some(proof) = self.cache.get(&exit_info) {
            return Ok(ExitProofData{
                exit_info,
                amount: proof.as_ref().map(|s|s.0.clone()),
                proof: proof.as_ref().map(|s|s.1.clone()),
            })
        }

        let mut storage = self.conn_pool.access_storage_with_retry().await?;
        if let Some(stored_proof) = storage.prover_schema()
            .get_proof_by_exit_info((&exit_info).into())
            .await?
        {
            let mut exit_data:ExitProofData = stored_proof.into();
            exit_data.exit_info.account_address = exit_info.account_address;

            let ExitProofData{
                exit_info,
                amount,
                proof
            } = exit_data.clone();
            self.cache.insert(
                exit_info,
                amount.and_then(|a|proof.map(|p|(a, p)))
            ).await;

            Ok(exit_data)
        } else {
            Err(ExodusStatus::ExitProofTaskNotExist)
        }
    }
}
