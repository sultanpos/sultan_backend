use async_trait::async_trait;
use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::{
    domain::{DomainResult, Error},
    storage::transaction::TransactionManager,
};

pub struct SqliteTransactionManager {
    pool: SqlitePool,
}

impl SqliteTransactionManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

#[async_trait]
impl TransactionManager for SqliteTransactionManager {
    type Transaction<'a>
        = Transaction<'a, Sqlite>
    where
        Self: 'a;

    async fn begin(&self) -> DomainResult<Self::Transaction<'_>> {
        self.pool
            .begin()
            .await
            .map_err(|e| Error::Database(format!("Failed to begin transaction: {}", e)))
    }

    async fn commit<'a>(&self, tx: Self::Transaction<'a>) -> DomainResult<()> {
        tx.commit()
            .await
            .map_err(|e| Error::Database(format!("Failed to commit transaction: {}", e)))
    }

    async fn rollback<'a>(&self, tx: Self::Transaction<'a>) -> DomainResult<()> {
        tx.rollback()
            .await
            .map_err(|e| Error::Database(format!("Failed to rollback transaction: {}", e)))
    }
}
