use async_trait::async_trait;

use crate::domain::{
    Context, DomainResult,
    model::{
        pagination::PaginationOptions,
        supplier::{Supplier, SupplierCreate, SupplierFilter, SupplierUpdate},
    },
};

#[async_trait]
pub trait SupplierRepository: Send + Sync {
    async fn create(&self, ctx: &Context, id: i64, supplier: &SupplierCreate) -> DomainResult<()>;
    async fn update(&self, ctx: &Context, id: i64, supplier: &SupplierUpdate) -> DomainResult<()>;
    async fn delete(&self, ctx: &Context, id: i64) -> DomainResult<()>;
    async fn get_all(
        &self,
        ctx: &Context,
        filter: &SupplierFilter,
        pagination: &PaginationOptions,
    ) -> DomainResult<Vec<Supplier>>;
    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<Supplier>>;
}
