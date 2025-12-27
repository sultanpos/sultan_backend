use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::{
    domain::{Context, DomainResult, Error, model::token::Token},
    storage::token_repo::TokenRepository,
};

// Database model for Token - SQLite
#[derive(sqlx::FromRow, Debug)]
pub struct TokenDbSqlite {
    pub id: i64,
    pub user_id: i64,
    pub expired_at: String,
    pub token: String,
}

impl TryFrom<TokenDbSqlite> for Token {
    type Error = Error;

    fn try_from(db: TokenDbSqlite) -> Result<Self, Self::Error> {
        let expired_at = chrono::DateTime::parse_from_rfc3339(&db.expired_at)
            .map_err(|e| Error::Internal(format!("Failed to parse expired_at: {}", e)))?
            .with_timezone(&chrono::Utc);

        Ok(Token {
            id: db.id,
            user_id: db.user_id,
            expired_at,
            token: db.token,
        })
    }
}

#[derive(Clone)]
pub struct SqliteTokenRepository {
    pool: SqlitePool,
}

impl SqliteTokenRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TokenRepository for SqliteTokenRepository {
    async fn save(&self, _: &Context, token: &Token) -> DomainResult<()> {
        let expired_at = token.expired_at.to_rfc3339();

        let query = sqlx::query(
            r#"
            INSERT INTO refresh_tokens (user_id, expired_at, token)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(token.user_id)
        .bind(&expired_at)
        .bind(&token.token);

        query.execute(&self.pool).await?;

        Ok(())
    }

    async fn delete(&self, _: &Context, id: i64) -> DomainResult<()> {
        let query = sqlx::query("DELETE FROM refresh_tokens WHERE id = ?").bind(id);

        let result = query.execute(&self.pool).await?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound(format!("Token with id {} not found", id)));
        }

        Ok(())
    }

    async fn get_by_token(&self, _: &Context, token: &str) -> DomainResult<Option<Token>> {
        let query = sqlx::query_as::<_, TokenDbSqlite>(
            "SELECT id, user_id, expired_at, token FROM refresh_tokens WHERE token = ?",
        )
        .bind(token);

        let result = query.fetch_optional(&self.pool).await?;

        match result {
            Some(db_token) => Ok(Some(db_token.try_into()?)),
            None => Ok(None),
        }
    }
}
