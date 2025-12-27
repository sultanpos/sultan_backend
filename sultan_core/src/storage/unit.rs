use async_trait::async_trait;

use crate::domain::{
    Context, DomainResult,
    model::product::{UnitOfMeasure, UnitOfMeasureCreate, UnitOfMeasureUpdate},
};

#[async_trait]
pub trait UnitOfMeasureRepository: Send + Sync {
    async fn create(&self, ctx: &Context, id: i64, uom: &UnitOfMeasureCreate) -> DomainResult<()>;
    async fn update(&self, ctx: &Context, id: i64, uom: &UnitOfMeasureUpdate) -> DomainResult<()>;
    async fn delete(&self, ctx: &Context, id: i64) -> DomainResult<()>;
    async fn get_all(&self, ctx: &Context) -> DomainResult<Vec<UnitOfMeasure>>;
    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<UnitOfMeasure>>;
}
