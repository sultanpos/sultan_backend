use async_trait::async_trait;
use sqlx::Executor;

use crate::domain::{
    Context, DomainResult,
    model::product::{UnitOfMeasure, UnitOfMeasureCreate, UnitOfMeasureUpdate},
};

#[async_trait]
pub trait UnitOfMeasureRepository<DB>: Send + Sync {
    async fn create<'e, E>(
        &self,
        ctx: &Context,
        e: E,
        id: i64,
        uom: &UnitOfMeasureCreate,
    ) -> DomainResult<()>
    where
        E: Executor<'e, Database = DB>;
    async fn update<'e, E>(
        &self,
        ctx: &Context,
        e: E,
        id: i64,
        uom: &UnitOfMeasureUpdate,
    ) -> DomainResult<()>
    where
        E: Executor<'e, Database = DB>;
    async fn delete<'e, E>(&self, ctx: &Context, e: E, id: i64) -> DomainResult<()>
    where
        E: Executor<'e, Database = DB>;
    async fn get_all<'e, E>(&self, ctx: &Context, e: E) -> DomainResult<Vec<UnitOfMeasure>>
    where
        E: Executor<'e, Database = DB>;
    async fn get_by_id<'e, E>(
        &self,
        ctx: &Context,
        e: E,
        id: i64,
    ) -> DomainResult<Option<UnitOfMeasure>>
    where
        E: Executor<'e, Database = DB>;
}
