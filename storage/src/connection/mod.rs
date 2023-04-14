// Built-in deps
use std::time::Duration;
use std::{fmt, time::Instant};
// External imports
use async_trait::async_trait;
use deadpool::managed::{Manager, PoolConfig, RecycleResult, Timeouts};
use deadpool::Runtime;
use sqlx::{Connection, Error as SqlxError, PgConnection};
use tokio::time::sleep;
use tracing::log::warn;
// Local imports
use crate::StorageProcessor;

pub mod holder;

type Pool = deadpool::managed::Pool<DbPool>;

pub type PooledConnection = deadpool::managed::Object<DbPool>;

#[derive(Clone)]
pub struct DbPool {
    url: String,
}

const DB_CONNECTION_RETRIES: usize = 30000;

impl DbPool {
    fn create(url: impl Into<String>, max_size: usize) -> Pool {
        let pool_config = PoolConfig {
            max_size,
            timeouts: Timeouts::wait_millis(20_000), // wait 20 seconds before returning error
        };
        Pool::builder(DbPool { url: url.into() })
            .config(pool_config)
            .runtime(Runtime::Tokio1)
            .build()
            .unwrap()
    }
}

#[async_trait]
impl Manager for DbPool {
    type Type = PgConnection;
    type Error = SqlxError;

    async fn create(&self) -> Result<PgConnection, SqlxError> {
        PgConnection::connect(&self.url).await
    }
    async fn recycle(&self, obj: &mut PgConnection) -> RecycleResult<SqlxError> {
        Ok(obj.ping().await?)
    }
}

/// `ConnectionPool` is a wrapper over a `diesel`s `Pool`, encapsulating
/// the fixed size pool of connection to the database.
///
/// The size of the pool and the database URL are configured via environment
/// variables `DATABASE_POOL_SIZE` and `DATABASE_URL` respectively.
#[derive(Clone)]
pub struct ConnectionPool {
    pool: Pool,
}

impl fmt::Debug for ConnectionPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Recoverable connection")
    }
}

impl ConnectionPool {
    /// Establishes a pool of the connections to the database and
    /// creates a new `ConnectionPool` object.
    /// max_size - number of connections in pool
    pub fn new(database_url: String, max_size: usize) -> Self {
        let pool = DbPool::create(database_url, max_size);

        Self { pool }
    }

    /// Creates a `StorageProcessor` entity over a recoverable connection.
    /// Upon a database outage connection will block the thread until
    /// it will be able to recover the connection (or, if connection cannot
    /// be restored after several retries, this will be considered as
    /// irrecoverable database error and result in panic).
    ///
    /// This method is intended to be used in crucial contexts, where the
    /// database access is must-have (e.g. block worker).
    pub async fn access_storage_with_retry(&self) -> anyhow::Result<StorageProcessor<'_>> {
        let start = Instant::now();
        let connection = self.get_pooled_connection().await;
        metrics::histogram!("sql.connection_acquire", start.elapsed());

        Ok(StorageProcessor::from_pool(connection))
    }

    pub async fn access_storage(&self) -> anyhow::Result<StorageProcessor<'_>> {
        let start = Instant::now();
        let connection = self
            .pool
            .get()
            .await
            .map_err(|e| anyhow::format_err!("Failed to get connection to db: {}", e))?;
        metrics::histogram!("sql.connection_acquire", start.elapsed());

        Ok(StorageProcessor::from_pool(connection))
    }

    async fn get_pooled_connection(&self) -> PooledConnection {
        let mut retry_count = 0;

        let one_second = Duration::from_secs(1);

        while retry_count < DB_CONNECTION_RETRIES {
            let connection = self.pool.get().await;

            match connection {
                Ok(connection) => return connection,
                Err(e) => {
                    warn!(
                        "Failed to get connection to db: {}. \
                        Backing off for 1 second, retry_count: {:?}",
                        e, retry_count
                    );
                    retry_count += 1
                }
            }

            // Backing off for one second if facing an error
            sleep(one_second).await;
        }

        // Attempting to get the pooled connection for the last time
        self.pool.get().await.unwrap()
    }
}
