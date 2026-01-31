use async_trait::async_trait;
use sea_orm::ConnectionTrait;

use crate::domain::{
    DomainResult,
    model::{
        customer::{Customer, CustomerCreate, CustomerFilter, CustomerUpdate},
        pagination::PaginationOptions,
    },
};

/// Repository trait for Customer operations.
///
/// This trait defines the contract for managing customers in the system.
/// All methods accept `RepoCtx<impl ConnectionTrait>` to support both direct database
/// access and transactional operations.
#[async_trait]
pub trait CustomerRepository: Send + Sync {
    /// Creates a new customer.
    async fn create(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        customer: &CustomerCreate,
    ) -> DomainResult<()>;

    /// Updates an existing customer.
    async fn update(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        customer: &CustomerUpdate,
    ) -> DomainResult<()>;

    /// Soft deletes a customer.
    async fn delete(&self, ctx: &super::RepoCtx<impl ConnectionTrait>, id: i64)
    -> DomainResult<()>;

    /// Retrieves a customer by their unique number.
    async fn get_by_number(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        number: &str,
    ) -> DomainResult<Option<Customer>>;

    /// Retrieves a customer by ID (excluding soft-deleted records).
    async fn get_by_id(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<Customer>>;

    /// Retrieves all customers with filtering and pagination.
    async fn get_all(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        filter: &CustomerFilter,
        pagination: &PaginationOptions,
    ) -> DomainResult<Vec<Customer>>;
}
