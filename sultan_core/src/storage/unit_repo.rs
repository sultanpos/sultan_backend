use crate::domain::{
    Context, DomainResult,
    model::product::{UnitOfMeasure, UnitOfMeasureCreate, UnitOfMeasureUpdate},
};
use async_trait::async_trait;
use sea_orm::ConnectionTrait;

/// Repository context that wraps both the domain context and database connection.
///
/// The `RepoCtx` provides a unified way to pass both business context (authorization, cancellation)
/// and database access (connection or transaction) to repository methods.
///
/// # Generic Parameter
///
/// * `T: ConnectionTrait` - Any SeaORM connection type (DatabaseConnection, DatabaseTransaction, etc.)
///
/// # Usage with Direct Connection
///
/// ```rust,ignore
/// let ctx = RepoCtx {
///     ctx: Context::new(),
///     db: &database_connection,
/// };
/// repo.create(&ctx, id, &data).await?;
/// ```
///
/// # Usage with Transaction
///
/// ```rust,ignore
/// let txn = database_connection.begin().await?;
/// let ctx = RepoCtx {
///     ctx: Context::new(),
///     db: &txn,
/// };
/// repo.create(&ctx, id, &data).await?;
/// repo.update(&ctx, id, &update_data).await?;
/// txn.commit().await?;
/// ```
pub struct RepoCtx<T: ConnectionTrait> {
    pub ctx: Context,
    pub db: T,
}

/// Repository trait for Unit of Measure operations.
///
/// This trait defines the contract for managing units of measure in the system.
/// All methods accept `RepoCtx<impl ConnectionTrait>` to support both direct database
/// access and transactional operations.
///
/// # Implementations
///
/// - SQLite: [`SqliteUnitOfMeasureRepository`](crate::storage::sqlite::unit_repo::SqliteUnitOfMeasureRepository)
///
/// # Example
///
/// ```rust,ignore
/// use sultan_core::storage::unit_repo::{UnitOfMeasureRepository, RepoCtx};
/// use sultan_core::storage::sqlite::unit_repo::SqliteUnitOfMeasureRepository;
///
/// async fn example(db: &DatabaseConnection) -> DomainResult<()> {
///     let repo = SqliteUnitOfMeasureRepository::new();
///     let ctx = RepoCtx {
///         ctx: Context::new(),
///         db,
///     };
///     
///     // Create a new unit
///     let uom = UnitOfMeasureCreate {
///         name: "Kilogram".to_string(),
///         description: Some("Weight measurement".to_string()),
///     };
///     repo.create(&ctx, 12345, &uom).await?;
///     
///     // Get the unit
///     let unit = repo.get_by_id(&ctx, 12345).await?;
///     
///     // List all units
///     let units = repo.list(&ctx).await?;
///     
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait UnitOfMeasureRepository: Send + Sync {
    /// Creates a new unit of measure.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - Snowflake ID for the new unit
    /// * `uom` - Unit of measure data to create
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Unit created successfully
    /// * `Err(Error)` - Database error or validation error
    async fn create(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        uom: &UnitOfMeasureCreate,
    ) -> DomainResult<()>;

    /// Retrieves a unit of measure by ID (excluding soft-deleted records).
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the unit to retrieve
    ///
    /// # Returns
    ///
    /// * `Ok(Some(unit))` - Unit found
    /// * `Ok(None)` - Unit not found or soft-deleted
    /// * `Err(Error)` - Database error
    async fn get_by_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<UnitOfMeasure>>;

    /// Updates an existing unit of measure.
    ///
    /// Only provided fields in `UnitOfMeasureUpdate` will be updated.
    /// The `updated_at` timestamp is automatically updated.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the unit to update
    /// * `uom` - Update data
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Unit updated successfully
    /// * `Err(Error::NotFound)` - Unit not found or soft-deleted
    /// * `Err(Error)` - Database error
    async fn update(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        uom: &UnitOfMeasureUpdate,
    ) -> DomainResult<()>;

    /// Soft-deletes a unit of measure.
    ///
    /// This method marks the unit as deleted instead of physically removing it.
    /// The `is_deleted` flag is set to true, and `deleted_at` is set to the current timestamp.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the unit to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Unit deleted successfully
    /// * `Err(Error::NotFound)` - Unit not found or already deleted
    /// * `Err(Error)` - Database error
    async fn delete(&self, ctx: &RepoCtx<impl ConnectionTrait>, id: i64) -> DomainResult<()>;

    /// Lists all non-deleted units of measure.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<unit>)` - List of all active units
    /// * `Err(Error)` - Database error
    async fn list(&self, ctx: &RepoCtx<impl ConnectionTrait>) -> DomainResult<Vec<UnitOfMeasure>>;
}
