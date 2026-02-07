use async_trait::async_trait;
use sea_orm::ConnectionTrait;

use crate::domain::DomainResult;
use crate::domain::model::branch::{Branch, BranchCreate, BranchUpdate};

/// Repository trait for Branch operations.
///
/// This trait defines the contract for managing branches in the system.
/// All methods accept `RepoCtx<impl ConnectionTrait>` to support both direct database
/// access and transactional operations.
///
/// # Implementations
///
/// - SQLite: [`SqliteBranchRepository`](crate::storage::sqlite::branch::SqliteBranchRepository)
///
/// # Example
///
/// ```rust,ignore
/// use sultan_core::storage::branch_repo::{BranchRepository, RepoCtx};
/// use sultan_core::storage::sqlite::branch::SqliteBranchRepository;
///
/// async fn example(db: &DatabaseConnection) -> DomainResult<()> {
///     let repo = SqliteBranchRepository::new();
///     let ctx = RepoCtx {
///         ctx: Context::new(),
///         db,
///     };
///     
///     // Create a new branch
///     let branch = BranchCreate {
///         name: "Main Branch".to_string(),
///         code: "MB001".to_string(),
///         is_main: true,
///         address: Some("123 Main St".to_string()),
///         phone: Some("+1234567890".to_string()),
///         npwp: None,
///         image: None,
///     };
///     repo.create(&ctx, 12345, &branch).await?;
///     
///     // Get the branch
///     let branch = repo.get_by_id(&ctx, 12345).await?;
///     
///     // List all branches
///     let branches = repo.get_all(&ctx).await?;
///     
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait BranchRepository: Send + Sync {
    /// Creates a new branch.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - Snowflake ID for the new branch
    /// * `branch` - Branch data to create
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Branch created successfully
    /// * `Err(Error)` - Database error or validation error
    async fn create(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        branch: &BranchCreate,
    ) -> DomainResult<()>;

    /// Updates an existing branch.
    ///
    /// Only provided fields in `BranchUpdate` will be updated.
    /// The `updated_at` timestamp is automatically updated.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the branch to update
    /// * `branch` - Update data
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Branch updated successfully
    /// * `Err(Error::NotFound)` - Branch not found or soft-deleted
    /// * `Err(Error)` - Database error
    async fn update(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        branch: &BranchUpdate,
    ) -> DomainResult<()>;

    /// Soft-deletes a branch.
    ///
    /// This method marks the branch as deleted instead of physically removing it.
    /// The `is_deleted` flag is set to true, and `deleted_at` is set to the current timestamp.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the branch to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Branch deleted successfully
    /// * `Err(Error::NotFound)` - Branch not found or already deleted
    /// * `Err(Error)` - Database error
    async fn delete(&self, ctx: &super::RepoCtx<impl ConnectionTrait>, id: i64)
    -> DomainResult<()>;

    /// Lists all non-deleted branches.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Branch>)` - List of all active branches
    /// * `Err(Error)` - Database error
    async fn get_all(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
    ) -> DomainResult<Vec<Branch>>;

    /// Retrieves a branch by ID (excluding soft-deleted records).
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the branch to retrieve
    ///
    /// # Returns
    ///
    /// * `Ok(Some(branch))` - Branch found
    /// * `Ok(None)` - Branch not found or soft-deleted
    /// * `Err(Error)` - Database error
    async fn get_by_id(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<Branch>>;

    /// Sets `is_main` to false for all branches except the specified one.
    ///
    /// This is used to ensure only one branch can be the main branch at a time.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `except_id` - Optional ID of branch to exclude from the update (None to update all)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Update successful
    /// * `Err(Error)` - Database error
    async fn set_all_is_main_false(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        except_id: Option<i64>,
    ) -> DomainResult<()>;
}
