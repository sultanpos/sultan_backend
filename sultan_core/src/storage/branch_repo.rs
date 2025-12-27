use async_trait::async_trait;

use crate::domain::Context;
use crate::domain::DomainResult;
use crate::domain::model::branch::{Branch, BranchCreate, BranchUpdate};

#[async_trait]
pub trait BranchRepository: Send + Sync {
    async fn create(&self, ctx: &Context, id: i64, branch: &BranchCreate) -> DomainResult<()>;
    async fn update(&self, ctx: &Context, id: i64, branch: &BranchUpdate) -> DomainResult<()>;
    async fn delete(&self, ctx: &Context, id: i64) -> DomainResult<()>;
    async fn get_all(&self, ctx: &Context) -> DomainResult<Vec<Branch>>;
    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<Branch>>;
}
