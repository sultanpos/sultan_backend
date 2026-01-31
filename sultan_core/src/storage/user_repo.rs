use async_trait::async_trait;
use sea_orm::ConnectionTrait;

use crate::domain::DomainResult;
use crate::domain::model::pagination::PaginationOptions;
use crate::domain::model::permission::Permission;
use crate::domain::model::user::{User, UserCreate, UserFilter, UserUpdate};

/// Repository trait for User operations.
///
/// This trait defines the contract for managing users in the system.
/// All methods accept `RepoCtx<impl ConnectionTrait>` to support both direct database
/// access and transactional operations.
///
/// # Implementations
///
/// - SQLite: [`SqliteUserRepository`](crate::storage::sqlite::user::SqliteUserRepository)
///
/// # Example
///
/// ```rust,ignore
/// use sultan_core::storage::user_repo::{UserRepository, RepoCtx};
/// use sultan_core::storage::sqlite::user::SqliteUserRepository;
///
/// async fn example(db: &DatabaseConnection) -> DomainResult<()> {
///     let repo = SqliteUserRepository::new();
///     let ctx = RepoCtx {
///         ctx: Context::new(),
///         db,
///     };
///     
///     // Create a new user
///     let user = UserCreate {
///         username: "johndoe".to_string(),
///         password: "hashed_password".to_string(),
///         name: "John Doe".to_string(),
///         email: Some("john@example.com".to_string()),
///         photo: None,
///         pin: None,
///         address: None,
///         phone: None,
///     };
///     repo.create(&ctx, 12345, &user).await?;
///     
///     // Get the user by username
///     let user = repo.get_by_username(&ctx, "johndoe").await?;
///     
///     // List all users with filtering
///     let filter = UserFilter::default();
///     let pagination = PaginationOptions::new(1, 20, None);
///     let users = repo.get_all(&ctx, &filter, &pagination).await?;
///     
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Creates a new user.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - Snowflake ID for the new user
    /// * `user` - User data to create
    ///
    /// # Returns
    ///
    /// * `Ok(())` - User created successfully
    /// * `Err(Error)` - Database error or validation error (e.g., duplicate username)
    async fn create(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        user: &UserCreate,
    ) -> DomainResult<()>;

    /// Retrieves a user by username (excluding soft-deleted records).
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `username` - Username to search for
    ///
    /// # Returns
    ///
    /// * `Ok(Some(user))` - User found
    /// * `Ok(None)` - User not found or soft-deleted
    /// * `Err(Error)` - Database error
    async fn get_by_username(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        username: &str,
    ) -> DomainResult<Option<User>>;

    /// Updates an existing user.
    ///
    /// Only provided fields in `UserUpdate` will be updated.
    /// The `updated_at` timestamp is automatically updated.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the user to update
    /// * `user` - Update data
    ///
    /// # Returns
    ///
    /// * `Ok(())` - User updated successfully
    /// * `Err(Error::NotFound)` - User not found or soft-deleted
    /// * `Err(Error)` - Database error
    async fn update(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        user: &UserUpdate,
    ) -> DomainResult<()>;

    /// Updates a user's password.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the user
    /// * `password_hash` - New hashed password
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Password updated successfully
    /// * `Err(Error::NotFound)` - User not found
    /// * `Err(Error)` - Database error
    async fn update_password(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        password_hash: &str,
    ) -> DomainResult<()>;

    /// Soft-deletes a user.
    ///
    /// This method marks the user as deleted instead of physically removing it.
    /// The `is_deleted` flag is set to true, and `deleted_at` is set to the current timestamp.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the user to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - User deleted successfully
    /// * `Err(Error::NotFound)` - User not found or already deleted
    /// * `Err(Error)` - Database error
    async fn delete(&self, ctx: &super::RepoCtx<impl ConnectionTrait>, id: i64)
    -> DomainResult<()>;

    /// Retrieves all users with filtering and pagination.
    ///
    /// Supports filtering by username, name, and email.
    /// Username and email use exact matching, name uses partial matching (LIKE).
    /// Soft-deleted users are excluded from results.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `filter` - Filter criteria (all fields are optional)
    /// * `pagination` - Pagination options (page, page_size, order)
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<User>)` - List of users matching the criteria
    /// * `Err(Error)` - Database error
    async fn get_all(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        filter: &UserFilter,
        pagination: &PaginationOptions,
    ) -> DomainResult<Vec<User>>;

    /// Retrieves a user by ID (excluding soft-deleted records).
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the user to retrieve
    ///
    /// # Returns
    ///
    /// * `Ok(Some(user))` - User found
    /// * `Ok(None)` - User not found or soft-deleted
    /// * `Err(Error)` - Database error
    async fn get_by_id(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<User>>;

    /// Saves or updates a user permission.
    ///
    /// If a permission with the same user_id, branch_id, and resource already exists,
    /// it will be replaced with the new action value.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `user_id` - ID of the user
    /// * `branch_id` - Optional branch ID (None for global permissions)
    /// * `resource` - Resource identifier
    /// * `action` - Action bitmask
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Permission saved successfully
    /// * `Err(Error)` - Database error
    async fn save_permission(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        user_id: i64,
        branch_id: Option<i64>,
        resource: i32,
        action: i32,
    ) -> DomainResult<()>;

    /// Deletes a user permission.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `user_id` - ID of the user
    /// * `branch_id` - Optional branch ID (None for global permissions)
    /// * `resource` - Resource identifier
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Permission deleted successfully
    /// * `Err(Error::NotFound)` - Permission not found
    /// * `Err(Error)` - Database error
    async fn delete_permission(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        user_id: i64,
        branch_id: Option<i64>,
        resource: i32,
    ) -> DomainResult<()>;

    /// Retrieves all permissions for a user.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `user_id` - ID of the user
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Permission>)` - List of user's permissions
    /// * `Err(Error)` - Database error
    async fn get_permissions(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        user_id: i64,
    ) -> DomainResult<Vec<Permission>>;
}
