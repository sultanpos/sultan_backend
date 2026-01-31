use async_trait::async_trait;
use sea_orm::ConnectionTrait;

use crate::domain::{
    DomainResult,
    model::{
        pagination::PaginationOptions,
        supplier::{Supplier, SupplierCreate, SupplierFilter, SupplierUpdate},
    },
};

/// Repository trait for Supplier operations.
///
/// This trait defines the contract for managing suppliers in the system.
/// All methods accept `RepoCtx<impl ConnectionTrait>` to support both direct database
/// access and transactional operations.
///
/// # Implementations
///
/// - SQLite: [`SqliteSupplierRepository`](crate::storage::sqlite::supplier::SqliteSupplierRepository)
///
/// # Example
///
/// ```rust,ignore
/// use sultan_core::storage::supplier_repo::{SupplierRepository, RepoCtx};
/// use sultan_core::storage::sqlite::supplier::SqliteSupplierRepository;
///
/// async fn example(db: &DatabaseConnection) -> DomainResult<()> {
///     let repo = SqliteSupplierRepository::new();
///     let ctx = RepoCtx {
///         ctx: Context::new(),
///         db,
///     };
///     
///     // Create a new supplier
///     let supplier = SupplierCreate {
///         name: "Main Supplier".to_string(),
///         code: Some("SUP001".to_string()),
///         address: Some("123 Main St".to_string()),
///         email: Some("supplier@example.com".to_string()),
///         phone: Some("+1234567890".to_string()),
///         npwp: Some("12345678901234".to_string()),
///         npwp_name: Some("PT Main Supplier".to_string()),
///         metadata: None,
///     };
///     repo.create(&ctx, 12345, &supplier).await?;
///     
///     // Get the supplier by ID
///     let supplier = repo.get_by_id(&ctx, 12345).await?;
///     
///     // List all suppliers with filtering
///     let filter = SupplierFilter::default();
///     let pagination = PaginationOptions::new(1, 20, None);
///     let suppliers = repo.get_all(&ctx, &filter, &pagination).await?;
///     
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait SupplierRepository: Send + Sync {
    /// Creates a new supplier.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - Snowflake ID for the new supplier
    /// * `supplier` - Supplier data to create
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Supplier created successfully
    /// * `Err(Error)` - Database error or validation error
    async fn create(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        supplier: &SupplierCreate,
    ) -> DomainResult<()>;

    /// Updates an existing supplier.
    ///
    /// Only provided fields in `SupplierUpdate` will be updated.
    /// The `updated_at` timestamp is automatically updated.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the supplier to update
    /// * `supplier` - Update data
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Supplier updated successfully
    /// * `Err(Error::NotFound)` - Supplier not found or soft-deleted
    /// * `Err(Error)` - Database error
    async fn update(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        supplier: &SupplierUpdate,
    ) -> DomainResult<()>;

    /// Soft-deletes a supplier.
    ///
    /// This method marks the supplier as deleted instead of physically removing it.
    /// The `is_deleted` flag is set to true, and `deleted_at` is set to the current timestamp.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the supplier to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Supplier deleted successfully
    /// * `Err(Error::NotFound)` - Supplier not found or already deleted
    /// * `Err(Error)` - Database error
    async fn delete(&self, ctx: &super::RepoCtx<impl ConnectionTrait>, id: i64)
    -> DomainResult<()>;

    /// Retrieves all suppliers with filtering and pagination.
    ///
    /// Supports filtering by name, code, email, phone, and npwp.
    /// All filters use partial matching (LIKE).
    /// Soft-deleted suppliers are excluded from results.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `filter` - Filter criteria (all fields are optional)
    /// * `pagination` - Pagination options (page, page_size, order)
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Supplier>)` - List of suppliers matching the criteria
    /// * `Err(Error)` - Database error
    async fn get_all(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        filter: &SupplierFilter,
        pagination: &PaginationOptions,
    ) -> DomainResult<Vec<Supplier>>;

    /// Retrieves a supplier by ID (excluding soft-deleted records).
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the supplier to retrieve
    ///
    /// # Returns
    ///
    /// * `Ok(Some(supplier))` - Supplier found
    /// * `Ok(None)` - Supplier not found or soft-deleted
    /// * `Err(Error)` - Database error
    async fn get_by_id(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<Supplier>>;
}
