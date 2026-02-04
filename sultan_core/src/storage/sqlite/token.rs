use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set,
};

use super::entity::{TokenColumn, TokenEntity};
use crate::{
    domain::{DomainResult, Error, model::token::Token},
    storage::{RepoCtx, TokenRepository},
};

/// SQLite implementation of [`TokenRepository`] using SeaORM.
///
/// This repository handles all refresh token-related database operations
/// including save, delete, and retrieval by token value.
///
/// Unlike most repositories, this uses:
/// - Auto-increment primary key (not Snowflake ID)
/// - Physical deletion (not soft-delete)
#[derive(Clone, Default)]
pub struct SqliteTokenRepository {}

impl SqliteTokenRepository {
    pub fn new() -> Self {
        SqliteTokenRepository {}
    }
}

#[async_trait]
impl TokenRepository for SqliteTokenRepository {
    async fn save(&self, ctx: &RepoCtx<impl ConnectionTrait>, token: &Token) -> DomainResult<()> {
        let expired_at = token.expired_at.to_rfc3339();

        let token_model = super::entity::TokenActiveModel {
            id: ActiveValue::NotSet, // Let database auto-generate the ID
            user_id: Set(token.user_id),
            expired_at: Set(expired_at),
            token: Set(token.token.clone()),
        };

        // Use insert to let auto-increment handle ID generation
        token_model.insert(&ctx.db).await?;
        Ok(())
    }

    async fn delete(&self, ctx: &RepoCtx<impl ConnectionTrait>, id: i64) -> DomainResult<()> {
        let result = TokenEntity::delete_by_id(id).exec(&ctx.db).await?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!("Token with id {} not found", id)));
        }

        Ok(())
    }

    async fn get_by_token(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        token: &str,
    ) -> DomainResult<Option<Token>> {
        let result = TokenEntity::find()
            .filter(TokenColumn::Token.eq(token))
            .one(&ctx.db)
            .await?;

        match result {
            Some(model) => Ok(Some(model.to_domain()?)),
            None => Ok(None),
        }
    }
}
