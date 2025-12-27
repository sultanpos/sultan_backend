use async_trait::async_trait;

use crate::domain::{
    Context, DomainResult,
    model::category::{Category, CategoryCreate, CategoryUpdate},
};

#[async_trait]
pub trait CategoryRepository: Send + Sync {
    async fn create(&self, ctx: &Context, id: i64, category: &CategoryCreate) -> DomainResult<()>;
    async fn update(&self, ctx: &Context, id: i64, category: &CategoryUpdate) -> DomainResult<()>;
    async fn delete(&self, ctx: &Context, id: i64) -> DomainResult<()>;
    async fn get_all(&self, ctx: &Context) -> DomainResult<Vec<Category>>;
    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<Category>>;
}
