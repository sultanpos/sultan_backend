use async_trait::async_trait;
use sea_orm::ConnectionTrait;

use crate::domain::{
    DomainResult,
    model::customer::{Customer, CustomerCreate, CustomerPage, CustomerQuery, CustomerUpdate},
};

/// Repository trait for Customer operations.
///
/// This trait defines the contract for managing customers in the system.
/// All methods accept `RepoCtx<impl ConnectionTrait>` to support both direct database
/// access and transactional operations.
///
/// # Implementations
///
/// - SQLite: [`SqliteCustomerRepository`](crate::storage::sqlite::customer::SqliteCustomerRepository)
///
/// # Example
///
/// ```rust,ignore
/// use sultan_core::storage::customer_repo::{CustomerRepository, RepoCtx};
/// use sultan_core::storage::sqlite::customer::SqliteCustomerRepository;
///
/// async fn example(db: &DatabaseConnection) -> DomainResult<()> {
///     let repo = SqliteCustomerRepository::new();
///     let ctx = RepoCtx {
///         ctx: Context::new(),
///         db,
///     };
///     
///     // Create a new customer
///     let customer = CustomerCreate {
///         number: "CUST001".to_string(),
///         name: "John Doe".to_string(),
///         address: Some("123 Main St".to_string()),
///         email: Some("john@example.com".to_string()),
///         phone: Some("+1234567890".to_string()),
///         level: 1,
///         metadata: None,
///     };
///     repo.create(&ctx, 12345, &customer).await?;
///     
///     // Get the customer by ID
///     let customer = repo.get_by_id(&ctx, 12345).await?;
///     
///     // Get customer by number
///     let customer = repo.get_by_number(&ctx, "CUST001").await?;
///     
///     // List all customers with filtering
///     let filter = CustomerFilter::default();
///     let pagination = PaginationOptions::new(1, 20, None);
///     let customers = repo.get_all(&ctx, &filter, &pagination).await?;
///     
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait CustomerRepository: Send + Sync {
    /// Creates a new customer.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - Snowflake ID for the new customer
    /// * `customer` - Customer data to create
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Customer created successfully
    /// * `Err(Error::Conflict)` - Customer with the same number already exists
    /// * `Err(Error)` - Database error or validation error
    async fn create(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        customer: &CustomerCreate,
    ) -> DomainResult<()>;

    /// Updates an existing customer.
    ///
    /// Only provided fields in `CustomerUpdate` will be updated.
    /// The `updated_at` timestamp is automatically updated.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the customer to update
    /// * `customer` - Update data
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Customer updated successfully
    /// * `Err(Error::NotFound)` - Customer not found or soft-deleted
    /// * `Err(Error::Conflict)` - Updated number conflicts with existing customer
    /// * `Err(Error)` - Database error
    async fn update(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        customer: &CustomerUpdate,
    ) -> DomainResult<()>;

    /// Soft-deletes a customer.
    ///
    /// This method marks the customer as deleted instead of physically removing it.
    /// The `is_deleted` flag is set to true, and `deleted_at` is set to the current timestamp.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the customer to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Customer deleted successfully
    /// * `Err(Error::NotFound)` - Customer not found or already deleted
    /// * `Err(Error)` - Database error
    async fn delete(&self, ctx: &super::RepoCtx<impl ConnectionTrait>, id: i64)
    -> DomainResult<()>;

    /// Retrieves a customer by their unique number.
    ///
    /// The search is case-sensitive and excludes soft-deleted customers.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `number` - Unique customer number to search for
    ///
    /// # Returns
    ///
    /// * `Ok(Some(customer))` - Customer found
    /// * `Ok(None)` - Customer not found or soft-deleted
    /// * `Err(Error)` - Database error
    async fn get_by_number(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        number: &str,
    ) -> DomainResult<Option<Customer>>;

    /// Retrieves a customer by ID (excluding soft-deleted records).
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the customer to retrieve
    ///
    /// # Returns
    ///
    /// * `Ok(Some(customer))` - Customer found
    /// * `Ok(None)` - Customer not found or soft-deleted
    /// * `Err(Error)` - Database error
    async fn get_by_id(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<Customer>>;

    /// Retrieves all customers with filtering and cursor-based pagination.
    ///
    /// Supports filtering by name, number, email, phone, and level.
    /// All filters use partial matching (LIKE) except for level which uses exact matching.
    /// Soft-deleted customers are excluded from results.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `query` - Query options including filter, sort field, sort direction, cursor, and limit
    ///
    /// # Returns
    ///
    /// * `Ok(CustomerPage)` - Page of customers with optional next cursor
    /// * `Err(Error)` - Database error
    async fn get_all(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        query: &CustomerQuery,
    ) -> DomainResult<CustomerPage>;
}
