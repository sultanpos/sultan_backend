use async_trait::async_trait;
use sea_orm::ConnectionTrait;

use crate::domain::DomainResult;
use crate::domain::model::category::{Category, CategoryCreate, CategoryUpdate};

/// Repository trait for Category operations.
///
/// This trait defines the contract for managing categories in the system.
/// Categories support hierarchical structure with parent-child relationships.
/// All methods accept `RepoCtx<impl ConnectionTrait>` to support both direct database
/// access and transactional operations.
///
/// # Implementations
///
/// - SQLite: [`SqliteCategoryRepository`](crate::storage::sqlite::category::SqliteCategoryRepository)
///
/// # Example
///
/// ```rust,ignore
/// use sultan_core::storage::category_repo::{CategoryRepository, RepoCtx};
/// use sultan_core::storage::sqlite::category::SqliteCategoryRepository;
///
/// async fn example(db: &DatabaseConnection) -> DomainResult<()> {
///     let repo = SqliteCategoryRepository::new();
///     let ctx = RepoCtx {
///         ctx: Context::new(),
///         db,
///     };
///     
///     // Create a new category
///     let category = CategoryCreate {
///         name: "Electronics".to_string(),
///         description: Some("Electronic items".to_string()),
///         parent_id: None,
///     };
///     repo.create(&ctx, 12345, &category).await?;
///     
///     // Get all categories (returns tree structure)
///     let categories = repo.get_all(&ctx).await?;
///     
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait CategoryRepository: Send + Sync {
    /// Creates a new category.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - Snowflake ID for the new category
    /// * `category` - Category data to create
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Category created successfully
    /// * `Err(Error)` - Database error or validation error
    async fn create(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        category: &CategoryCreate,
    ) -> DomainResult<()>;

    /// Updates an existing category.
    ///
    /// Only provided fields in `CategoryUpdate` will be updated.
    /// The `updated_at` timestamp is automatically updated.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the category to update
    /// * `category` - Fields to update
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Category updated successfully
    /// * `Err(Error::NotFound)` - Category with given ID not found or already deleted
    /// * `Err(Error)` - Database error
    async fn update(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        category: &CategoryUpdate,
    ) -> DomainResult<()>;

    /// Soft deletes a category by ID.
    ///
    /// Sets `is_deleted=true` and `deleted_at` timestamp. The category remains in the
    /// database but will not appear in queries.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the category to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Category deleted successfully
    /// * `Err(Error::NotFound)` - Category with given ID not found or already deleted
    /// * `Err(Error)` - Database error
    async fn delete(&self, ctx: &super::RepoCtx<impl ConnectionTrait>, id: i64)
    -> DomainResult<()>;

    /// Retrieves all non-deleted categories as a hierarchical tree structure.
    ///
    /// Returns categories with their children populated recursively.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Category>)` - List of root categories with children
    /// * `Err(Error)` - Database error
    async fn get_all(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
    ) -> DomainResult<Vec<Category>>;

    /// Retrieves a single category by ID.
    ///
    /// Returns `None` if the category doesn't exist or is soft-deleted.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the category to retrieve
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Category))` - Category found with children populated
    /// * `Ok(None)` - Category not found or deleted
    /// * `Err(Error)` - Database error
    async fn get_by_id(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<Category>>;
}
