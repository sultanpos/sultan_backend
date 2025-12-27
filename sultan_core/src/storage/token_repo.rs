use async_trait::async_trait;

use crate::domain::{Context, DomainResult, model::token::Token};

#[async_trait]
pub trait TokenRepository: Send + Sync {
    async fn save(&self, ctx: &Context, token: &Token) -> DomainResult<()>;
    async fn delete(&self, ctx: &Context, id: i64) -> DomainResult<()>;
    async fn get_by_token(&self, ctx: &Context, token: &str) -> DomainResult<Option<Token>>;
}
