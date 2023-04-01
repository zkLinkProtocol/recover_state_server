//! Storage crate provides the interfaces to interact with the database.
//! The backend database used in this crate is `Postgres`, and interaction
//! with it is based on the `sqlx` crate.
//!
//! The essential structure of this crate is the `StorageProcessor`, which
//! holds down the connection to the database and provides abstract interfaces
//! to modify it (called `Schema`s).
//!
//! # Crate Architecture Overview
//!
//! This crate can be divided into three logical parts:
//! - Connection utilities. Tools to establish connections to the database,
//!   stored in the `connection` module.
//! - `Schema`s. Schema is a logically extracted access to the part of
//!   the database, e.g. `ethereum` (which contains methods to store the
//!   information about interaction with the Ethereum blockchain).
//! - `StorageProcessor`. A structure that connects the two points above
//!   into one user-friendly interface.
//!
//! Most of schema modules contain at least two files:
//! - `scheduler`, which contains the schema itself.
//! - `records.rs`, which contains the representation of the associated database
//!   tables as structures.
//!
//! The latter ones usually don't contain any logic other than the structures
//! declarations, and all the logic is contained in either schema (for most
//! modules), or in an additional helper module (e.g. in the `chain/block` module).
//!
//! # Schema Hierarchy
//!
//! There are the following sets of schemas:
//!
//! - recover_state, for the recover_state crate.
//! - ethereum, for the data associated with the Ethereum blockchain.
//! - prover, for the data on prover jobs, proofs, etc.
//! - tokens, for storing and loading known tokens.
//! - chain - the biggest one, which includes several schemas for the ZkLink sidechain itself.
//!
//! The chain module includes the following schemas:
//!
//! - account, for storing and loading account data.
//! - block, the main one, which implements the logic of the block creation.
//! - operations, the transactions storage.
//! - operations_ext, a set of getters for the operations, more specific and convenient to use than operations has.
//! - state, basically the sidechain state manager (which includes the applying of the state changes).
//! - stats, other auxiliary schema which provides additional getters for the database stats.
//!
//! If you have to add a method, and can't decide which schema it belongs to, use the following logic:
//!
//! 1. Will your method be used by different modules? If no (e.g. it'll be only used by `sender` or `recover_state`),
//!    then mind adding method to high-level schema (you may even create a new one, if it makes sense).
//!    If yes, probably it affects the sidechain state, and you should choose one of the `chain` schemas.
//! 2. Will your method be used by other schemas? If yes, choose one of the "low-level" schemas, like `operations,
//!    or `account`.
//! 3. Is your method is some form of convenient getter? If so, `operations_ext` may be suitable.
//! 4. Otherwise, it probably should be in `block` (for high-level interaction), `state` (for ZkLink tables update that
//!    are not low-level enough for other modules), or a new schema (if none of existing ones fit your needs).
//!
//! # Testing Approach
//!
//! Tests for the storage use the actual empty Postgres database.
//! Because of that, these tests are disabled by default, to run them you must use
//! `zk test db` (or `zk test db --no-reset`, if this is not a first run)
//! command, which will setup the database and enable the tests by passing a feature flag.
//!
//! Tests are implemented in a form of "test transactions", which are database transactions
//! that will never be committed. Thus it is not required to clear the database after running
//! tests. Also the database used for tests is different than the database used for `server`,
//! thus one should not fear to overwrite any important data by running the tests.

// `sqlx` macros result in these warning being triggered.
#![allow(clippy::toplevel_ref_arg, clippy::suspicious_else_formatting)]

// Built-in deps
// use std::env;
// External imports
use sqlx::{postgres::Postgres, Connection, PgConnection, Transaction};
// Workspace imports
use zklink_basic_types::BlockNumber;
use zklink_types::ActionType;
// Local imports
use crate::connection::{holder::ConnectionHolder, PooledConnection};

pub mod chain;
pub mod connection;
pub mod diff;
pub mod tokens;
pub mod recover_state;
pub mod prover;

pub use crate::connection::ConnectionPool;
pub type QueryResult<T, E = anyhow::Error> = Result<T, E>;

/// The maximum possible block number in the storage.
pub const MAX_BLOCK_NUMBER: BlockNumber = BlockNumber(u32::MAX);
/// The maximum possible index value in block in the storage.
pub const MAX_BLOCK_INDEX: u32 = i32::MAX as u32;

/// Storage processor is the main storage interaction point.
/// It holds down the connection (either direct or pooled) to the database
/// and provide methods to obtain different storage schemas.
#[derive(Debug)]
pub struct StorageProcessor<'a> {
    conn: ConnectionHolder<'a>,
    in_transaction: bool,
}

#[derive(sqlx::Type, Debug, Clone, PartialEq, Eq)]
#[sqlx(type_name = "action_type")]
pub enum StorageActionType {
    COMMIT,
    VERIFY,
}

impl From<ActionType> for StorageActionType {
    fn from(action_type: ActionType) -> Self {
        match action_type {
            ActionType::COMMIT => StorageActionType::COMMIT,
            ActionType::VERIFY => StorageActionType::VERIFY,
        }
    }
}

impl<'a> StorageProcessor<'a> {
    /// Creates a `StorageProcessor` using an unique sole connection to the database.
    pub async fn mock() -> QueryResult<StorageProcessor<'a>> {
        let connection = PgConnection::connect("postgres://postgres:postgres@localhost/plasma").await?;
        Ok(StorageProcessor {
            conn: ConnectionHolder::Direct(connection),
            in_transaction: false,
        })
    }

    /// Creates a `StorageProcessor` using an unique sole connection to the database.
    pub async fn establish_connection(database_url: String) -> QueryResult<StorageProcessor<'a>> {
        let connection = PgConnection::connect(&database_url).await?;
        Ok(StorageProcessor {
            conn: ConnectionHolder::Direct(connection),
            in_transaction: false,
        })
    }

    pub async fn start_transaction(
        &mut self,
    ) -> Result<StorageProcessor<'_>, anyhow::Error> {
        let transaction = self.conn().begin().await?;

        let mut processor = StorageProcessor::from_transaction(transaction);
        processor.in_transaction = true;

        Ok(processor)
    }

    /// Checks if the `StorageProcessor` is currently within database transaction.
    pub fn in_transaction(&self) -> bool {
        self.in_transaction
    }

    pub fn from_transaction(conn: Transaction<'_, Postgres>) -> StorageProcessor<'_> {
        StorageProcessor {
            conn: ConnectionHolder::Transaction(conn),
            in_transaction: true,
        }
    }

    pub async fn commit(self) -> QueryResult<()> {
        if let ConnectionHolder::Transaction(transaction) = self.conn {
            transaction.commit().await?;
            Ok(())
        } else {
            panic!("StorageProcessor::commit can only be invoked after calling StorageProcessor::begin_transaction");
        }
    }

    /// Creates a `StorageProcessor` using a pool of connections.
    /// This method borrows one of the connections from the pool, and releases it
    /// after `drop`.
    pub fn from_pool(conn: PooledConnection) -> Self {
        Self {
            conn: ConnectionHolder::Pooled(conn),
            in_transaction: false,
        }
    }

    pub fn conn(&mut self) -> &mut PgConnection {
        match &mut self.conn {
            ConnectionHolder::Pooled(conn) => conn,
            ConnectionHolder::Direct(conn) => conn,
            ConnectionHolder::Transaction(conn) => conn,
        }
    }

    /// Gains access to the `Chain` schemas.
    pub fn chain(&mut self) -> chain::ChainIntermediator<'_, 'a> {
        chain::ChainIntermediator(self)
    }

    /// Gains access to the `Prover` schema.
    pub fn prover_schema(&mut self) -> prover::ProverSchema<'_, 'a> {
        prover::ProverSchema(self)
    }

    /// Gains access to the `Tokens` schema.
    pub fn tokens_schema(&mut self) -> tokens::TokensSchema<'_, 'a> {
        tokens::TokensSchema(self)
    }

    pub fn recover_schema(&mut self) -> recover_state::RecoverSchema<'_, 'a> {
        recover_state::RecoverSchema(self)
    }
}



