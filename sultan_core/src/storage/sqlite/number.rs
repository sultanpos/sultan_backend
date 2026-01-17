use async_trait::async_trait;
use serde::Serialize;
use sqlx::{Row, SqlitePool};

use crate::{
    domain::{
        Context, DomainResult,
        model::number::{NumberGenerateParams, NumberSequence},
    },
    storage::NumberRepository,
};

#[derive(Clone)]
pub struct SqliteNumberRepository {
    pool: SqlitePool,
}

impl SqliteNumberRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

// Database model for NumberSequence - SQLite
#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct NumberSequenceDbSqlite {
    pub id: i64,
    pub created_at: String,
    pub updated_at: String,
    pub prefix: String,
    pub branch_id: Option<i64>,
    pub year: i32,
    pub month: Option<i32>,
    pub last_number: i32,
}

impl From<NumberSequenceDbSqlite> for NumberSequence {
    fn from(db: NumberSequenceDbSqlite) -> Self {
        NumberSequence {
            id: db.id,
            created_at: super::parse_sqlite_date(&db.created_at),
            updated_at: super::parse_sqlite_date(&db.updated_at),
            prefix: db.prefix,
            branch_id: db.branch_id,
            year: db.year,
            month: db.month,
            last_number: db.last_number,
        }
    }
}

#[async_trait]
impl NumberRepository for SqliteNumberRepository {
    async fn generate_next(&self, _: &Context, params: &NumberGenerateParams) -> DomainResult<i32> {
        // Use a transaction to ensure atomicity
        let mut tx = self.pool.begin().await?;

        // Try to increment existing sequence
        let result = sqlx::query(
            r#"
            UPDATE number_sequences
            SET last_number = last_number + 1,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE prefix = ?
              AND (branch_id IS ? OR (branch_id IS NULL AND ? IS NULL))
              AND year = ?
              AND (month IS ? OR (month IS NULL AND ? IS NULL))
            RETURNING last_number
            "#,
        )
        .bind(&params.prefix)
        .bind(params.branch_id)
        .bind(params.branch_id)
        .bind(params.year)
        .bind(params.month)
        .bind(params.month)
        .fetch_optional(&mut *tx)
        .await?;

        let next_number = if let Some(row) = result {
            // Sequence exists, return incremented value
            row.try_get::<i32, _>("last_number")?
        } else {
            // Sequence doesn't exist, create it with initial value
            let initial_number = 1;

            sqlx::query(
                r#"
                INSERT INTO number_sequences ( prefix, branch_id, year, month, last_number)
                VALUES ( ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&params.prefix)
            .bind(params.branch_id)
            .bind(params.year)
            .bind(params.month)
            .bind(initial_number)
            .execute(&mut *tx)
            .await?;

            initial_number
        };

        // Commit the transaction
        tx.commit().await?;

        Ok(next_number)
    }

    async fn get_sequence(
        &self,
        _: &Context,
        params: &NumberGenerateParams,
    ) -> DomainResult<Option<NumberSequence>> {
        let result = sqlx::query_as::<_, NumberSequenceDbSqlite>(
            r#"
            SELECT id, created_at, updated_at, prefix, branch_id, year, month, last_number
            FROM number_sequences
            WHERE prefix = ?
              AND (branch_id IS ? OR (branch_id IS NULL AND ? IS NULL))
              AND year = ?
              AND (month IS ? OR (month IS NULL AND ? IS NULL))
            "#,
        )
        .bind(&params.prefix)
        .bind(params.branch_id)
        .bind(params.branch_id)
        .bind(params.year)
        .bind(params.month)
        .bind(params.month)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(NumberSequence::from))
    }
}
