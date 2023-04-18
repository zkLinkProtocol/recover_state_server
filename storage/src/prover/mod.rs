// Built-in deps
use chrono::Utc;
use std::time::Instant;
use tracing::info;
// External imports
// Workspace imports
// Local imports
use crate::{QueryResult, StorageProcessor};
use records::*;

pub mod records;

/// Prover schema is capable of handling the prover-related informations,
/// such as started prover jobs, registered provers and proofs for blocks.
#[derive(Debug)]
pub struct ProverSchema<'a, 'c>(pub &'a mut StorageProcessor<'c>);

impl<'a, 'c> ProverSchema<'a, 'c> {
    pub async fn get_stored_exit_proofs(&mut self, n: i64) -> QueryResult<Vec<StoredExitProof>> {
        let exit_proofs =
            sqlx::query_as!(StoredExitProof, r#"SELECT * FROM exit_proofs LIMIT $1"#, n)
                .fetch_all(self.0.conn())
                .await?;

        Ok(exit_proofs)
    }

    pub async fn get_latest_proofs_by_id(
        &mut self,
        id: Option<i64>,
        num: i64,
    ) -> QueryResult<Vec<StoredExitProof>> {
        let exit_proofs = match id {
            Some(id_value) => {
                sqlx::query_as!(
                    StoredExitProof,
                    r#"SELECT * FROM exit_proofs WHERE proof IS NOT NULL AND id < $1 ORDER BY id DESC LIMIT $2"#,
                    id_value,
                    num
                )
                .fetch_all(self.0.conn())
                .await?
            }
            None => sqlx::query_as!(
                StoredExitProof,
                r#"SELECT * FROM exit_proofs WHERE proof IS NOT NULL ORDER BY id DESC LIMIT $1"#,
                num
            )
            .fetch_all(self.0.conn())
            .await?,
        };

        Ok(exit_proofs)
    }

    /// Loads the specified proof task by account_id and sub_account_id and token_id.
    pub async fn get_proof_by_exit_info(
        &mut self,
        exit_info: StoredExitInfo,
    ) -> QueryResult<Option<StoredExitProof>> {
        let start = Instant::now();

        let stored_exit_proof = sqlx::query_as!(
            StoredExitProof,
            "SELECT * FROM exit_proofs WHERE chain_id=$1 AND account_id=$2 \
            AND sub_account_id=$3 AND l1_target_token=$4 AND l2_source_token=$5",
            exit_info.chain_id,
            exit_info.account_id,
            exit_info.sub_account_id,
            exit_info.l1_target_token,
            exit_info.l2_source_token
        )
        .fetch_optional(self.0.conn())
        .await?;

        metrics::histogram!("sql.recover_state.get_proof_by_exit_info", start.elapsed());
        Ok(stored_exit_proof)
    }

    /// Loads the specified proof task by account_id and sub_account_id and token_id.
    pub async fn get_proofs(
        &mut self,
        account_id: i64,
        sub_account_id: i16,
        l2_source_token: i32,
    ) -> QueryResult<Vec<StoredExitProof>> {
        let start = Instant::now();

        let stored_exit_proofs = sqlx::query_as!(
            StoredExitProof,
            "SELECT * FROM exit_proofs WHERE account_id=$1 AND l2_source_token=$2 AND sub_account_id=$3",
            account_id, l2_source_token, sub_account_id
        )
            .fetch_all(self.0.conn())
            .await?;

        metrics::histogram!("sql.recover_state.get_proofs", start.elapsed());
        Ok(stored_exit_proofs)
    }

    /// Start the task of the specified generating exit proof.
    async fn start_this_exit_proof_task(&mut self, exit_info: StoredExitInfo) -> QueryResult<()> {
        let start = Instant::now();

        let created_at: chrono::DateTime<chrono::Local> = chrono::Local::now();
        sqlx::query!(
            "UPDATE exit_proofs SET created_at=$6 WHERE chain_id=$1 AND account_id=$2 \
            AND sub_account_id=$3 AND l1_target_token=$4 AND l2_source_token=$5",
            exit_info.chain_id,
            exit_info.account_id,
            exit_info.sub_account_id,
            exit_info.l1_target_token,
            exit_info.l2_source_token,
            created_at
        )
        .execute(self.0.conn())
        .await?;

        metrics::histogram!(
            "sql.recover_state.start_this_exit_proof_task",
            start.elapsed()
        );
        Ok(())
    }

    /// Cancel the task of the specified running exit proof.
    pub async fn cancel_this_exit_proof_task(
        &mut self,
        exit_info: StoredExitInfo,
    ) -> QueryResult<()> {
        let start = Instant::now();

        sqlx::query!(
            "UPDATE exit_proofs SET created_at=NULL WHERE chain_id=$1 AND account_id=$2 \
            AND sub_account_id=$3 AND l1_target_token=$4 AND l2_source_token=$5",
            exit_info.chain_id,
            exit_info.account_id,
            exit_info.sub_account_id,
            exit_info.l1_target_token,
            exit_info.l2_source_token,
        )
        .execute(self.0.conn())
        .await?;

        metrics::histogram!(
            "sql.recover_state.cancel_this_exit_proof_task",
            start.elapsed()
        );
        Ok(())
    }

    /// Loads the tasks that have never been started.
    pub async fn load_exit_proof_task(&mut self) -> QueryResult<Option<StoredExitProof>> {
        let start = Instant::now();
        let mut transaction = self.0.start_transaction().await?;

        let stored_exit_proof = sqlx::query_as!(
            StoredExitProof,
            "SELECT * FROM exit_proofs WHERE created_at IS NULL LIMIT 1",
        )
        .fetch_optional(transaction.conn())
        .await?;
        if let Some(exit_proof) = &stored_exit_proof {
            ProverSchema(&mut transaction)
                .start_this_exit_proof_task(exit_proof.into())
                .await?;
        }

        transaction.commit().await?;
        metrics::histogram!("sql.recover_state.load_exit_proofs", start.elapsed());
        Ok(stored_exit_proof)
    }

    /// Query task id by exit info.
    pub async fn get_task_id(
        &mut self,
        task: StoredExitInfo,
    ) -> QueryResult<i64> {
        let start = Instant::now();

        // Gets the creation time of the target task
        let target_task_id = sqlx::query!(
            "SELECT id from exit_proofs WHERE chain_id=$1 AND account_id=$2 \
            AND sub_account_id=$3 AND l1_target_token=$4 AND l2_source_token=$5",
            task.chain_id,
            task.account_id,
            task.sub_account_id,
            task.l1_target_token,
            task.l2_source_token,
        )
        .fetch_one(self.0.conn())
        .await?
        .id;

        metrics::histogram!(
            "sql.recover_state.get_task_id",
            start.elapsed()
        );
        Ok(target_task_id)
    }

    /// Query which task the prover is currently working on.
    pub async fn get_running_max_task_id(
        &mut self,
    ) -> QueryResult<i64> {
        let start = Instant::now();

        let running_task_id = sqlx::query!(
            "SELECT MAX(id) FROM exit_proofs WHERE created_at IS NOT NULL",
        )
            .fetch_one(self.0.conn())
            .await?
            .max
            .unwrap_or_default();

        metrics::histogram!(
            "sql.recover_state.get_running_max_task_id",
            start.elapsed()
        );
        Ok(running_task_id)
    }

    /// Count the number of tasks running
    pub async fn count_running_tasks(&mut self) -> QueryResult<i64> {
        let start = Instant::now();

        // counts tasks that have been started but not completed.
        let count = sqlx::query!(
            "SELECT count(*) FROM exit_proofs WHERE created_at IS NOT NULL AND finished_at IS NULL",
        )
        .fetch_one(self.0.conn())
        .await?
        .count
        .unwrap_or_default();

        metrics::histogram!("sql.recover_state.count_running_tasks", start.elapsed());
        Ok(count)
    }

    /// Changes created_at to null for previously unfinished tasks
    pub async fn process_unfinished_tasks(&mut self) -> QueryResult<()> {
        let start = Instant::now();

        // Clean tasks that have been started but not completed.
        sqlx::query!(
            "UPDATE exit_proofs SET created_at=NULL WHERE created_at IS NOT NULL AND finished_at IS NULL",
        )
            .execute(self.0.conn())
            .await?;

        metrics::histogram!("sql.recover_state.count_running_tasks", start.elapsed());
        Ok(())
    }

    /// Stores exit proof that generated by exit task.
    pub async fn store_exit_proof(&mut self, proof: StoredExitProof) -> QueryResult<()> {
        let start = Instant::now();

        let finished_at = Utc::now();
        // counts tasks that have been started but not completed.
        sqlx::query!(
            "UPDATE exit_proofs SET proof=$6, amount=$7, finished_at=$8 WHERE chain_id=$1 AND account_id=$2 \
            AND sub_account_id=$3 AND l1_target_token=$4 AND l2_source_token=$5",
            proof.chain_id, proof.account_id,
            proof.sub_account_id, proof.l1_target_token, proof.l2_source_token,
            proof.proof, proof.amount, finished_at
        )
            .execute(self.0.conn())
            .await?;

        metrics::histogram!("sql.recover_state.store_exit_proof", start.elapsed());
        Ok(())
    }

    /// Inserts task that generated exit proof.
    pub async fn insert_exit_task(&mut self, task: StoredExitInfo) -> QueryResult<i64> {
        info!("Insert new exit task: {}", task);
        let start = Instant::now();

        // counts tasks that have been started but not completed.
        let id = sqlx::query!(
            "INSERT INTO exit_proofs (chain_id, account_id, sub_account_id, l1_target_token, l2_source_token) \
            VALUES ($1, $2, $3, $4, $5)\
            ON CONFLICT (chain_id, account_id, sub_account_id, l1_target_token, l2_source_token) \
            DO NOTHING RETURNING id",
            task.chain_id, task.account_id, task.sub_account_id, task.l1_target_token, task.l2_source_token,
        )
            .fetch_one(self.0.conn())
            .await?
            .id;

        metrics::histogram!("sql.recover_state.insert_exit_task", start.elapsed());
        Ok(id)
    }

    pub async fn insert_batch_exit_tasks(
        &mut self,
        batch_exit_tasks: Vec<StoredExitInfo>,
    ) -> QueryResult<Vec<i64>> {
        let mut tasks_ids = Vec::with_capacity(batch_exit_tasks.len());
        let mut transaction = self.0.start_transaction().await?;
        for exit_task in batch_exit_tasks {
            tasks_ids.push(transaction
                .prover_schema()
                .insert_exit_task(exit_task)
                .await?
            );
        }
        transaction.commit().await?;

        Ok(tasks_ids)
    }
}
