use async_trait::async_trait;

use crate::domain::{
    Context, DomainResult,
    model::{
        customer::{Customer, CustomerCreate, CustomerFilter, CustomerUpdate},
        pagination::PaginationOptions,
    },
};

#[async_trait]
pub trait CustomerRepository: Send + Sync {
    async fn create(&self, ctx: &Context, id: i64, customer: &CustomerCreate) -> DomainResult<()>;
    async fn update(&self, ctx: &Context, id: i64, customer: &CustomerUpdate) -> DomainResult<()>;
    async fn delete(&self, ctx: &Context, id: i64) -> DomainResult<()>;
    async fn get_by_number(&self, ctx: &Context, number: &str) -> DomainResult<Option<Customer>>;
    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<Customer>>;
    async fn get_all(
        &self,
        ctx: &Context,
        filter: &CustomerFilter,
        pagination: &PaginationOptions,
    ) -> DomainResult<Vec<Customer>>;
}
