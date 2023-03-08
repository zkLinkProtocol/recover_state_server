// Built-in deps
use std::time::Instant;
// External imports
use anyhow::format_err;
// Workspace imports
use zklink_types::BlockNumber;
// Local imports
use crate::{QueryResult, StorageProcessor};
use zklink_crypto::proof::{AggregatedProof, SingleProof};
use zklink_types::prover::{ProverJob, ProverJobStatus, ProverJobType};
use chrono::{DateTime, Utc};
use crate::chain::operations::records::AggType;
use crate::chain::operations::OperationsSchema;

pub mod records;

/// Prover schema is capable of handling the prover-related informations,
/// such as started prover jobs, registered provers and proofs for blocks.
#[derive(Debug)]
pub struct ProverSchema<'a, 'c>(pub &'a mut StorageProcessor<'c>);

impl<'a, 'c> ProverSchema<'a, 'c> {
    // /// Stores the proof for a block.
    // pub async fn store_proof(
    //     &mut self,
    //     job_id: i32,
    //     block_number: BlockNumber,
    //     proof: &SingleProof,
    // ) -> QueryResult<()> {
    //     let start = Instant::now();
    //     let mut transaction = self.0.start_transaction().await?;
    //     let updated_rows = sqlx::query!(
    //         "UPDATE prover_job_queue
    //         SET (updated_at, job_status, updated_by) = (now(), $1, 'server_finish_job')
    //         WHERE id = $2 AND job_type = $3",
    //         ProverJobStatus::Done.to_number(),
    //         job_id as i64,
    //         ProverJobType::SingleProof.to_string()
    //     )
    //         .execute(transaction.conn())
    //         .await?
    //         .rows_affected();
    //
    //     if updated_rows != 1 {
    //         return Err(format_err!("Missing job for stored proof"));
    //     }
    //
    //     sqlx::query!(
    //         "UPDATE proofs set status = $1, proof = $2 WHERE block_number = $3",
    //         BlockStatus::ProofCreated.to_number(),
    //         serde_json::to_value(proof).unwrap(),
    //         i64::from(*block_number),
    //     )
    //         .execute(transaction.conn())
    //         .await?;
    //     transaction.commit().await?;
    //
    //     metrics::histogram!("sql", start.elapsed(), "prover" => "store_proof");
    //     Ok(())
    // }
    //
    // /// Check the stored proof if exist for a block.
    // pub async fn check_proof_exist(
    //     &mut self,
    //     block_number: i64,
    // ) -> QueryResult<bool> {
    //     let count = sqlx::query!(
    //         "SELECT count(block_number) FROM proofs WHERE block_number = $1 AND status = $2",
    //         block_number,
    //         BlockStatus::ProofCreated.to_number()
    //     )
    //         .fetch_one(self.0.conn())
    //         .await?
    //         .count
    //         .unwrap_or(0);
    //     Ok(count > 0)
    // }
    //
    // /// Gets the stored proof for a block.
    // pub async fn load_proof(
    //     &mut self,
    //     block_number: i64,
    // ) -> QueryResult<Option<SingleProof>> {
    //     let stored: Option<NewProof> = sqlx::query_as!(
    //         NewProof,
    //         "SELECT block_number,proof FROM proofs WHERE block_number = $1",
    //         block_number,
    //     )
    //         .fetch_optional(self.0.conn())
    //         .await?;
    //     if let Some(new_proof) = stored {
    //         if let Some(proof) = new_proof.proof {
    //             return Ok(Some(serde_json::from_value(proof).unwrap()));
    //         }
    //     }
    //     Ok(None)
    // }
}
