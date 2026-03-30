use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use std::time::Duration;

use crate::application::ServiceDbHelper;
use crate::application::cache::CacheService;
use crate::crypto::password::PasswordHash;
use crate::domain::model::pagination::PaginationOptions;
use crate::domain::model::permission::{Permission, PermissionCreate, action, resource};
use crate::domain::model::user::{UserCreate, UserFilter, UserUpdate};
use crate::domain::{Context, DomainResult, User};
use crate::snowflake::IdGenerator;
use crate::storage::UserRepository;

/// Trait for user service operations.
///
/// This trait provides methods for user management including
/// creation, updates, password resets, deletion, and permissions.
#[async_trait]
pub trait UserServiceTrait: Send + Sync {
    async fn create(
        &self,
        ctx: &Context,
        user: &UserCreate,
        permissions: &[PermissionCreate],
    ) -> DomainResult<i64>;
    async fn update(
        &self,
        ctx: &Context,
        id: i64,
        user: &UserUpdate,
        permissions: Option<Vec<PermissionCreate>>,
    ) -> DomainResult<()>;
    async fn get_by_id(&self, ctx: &Context, user_id: i64) -> DomainResult<Option<User>>;
    async fn get_all(
        &self,
        ctx: &Context,
        filter: &UserFilter,
        pagination: &PaginationOptions,
    ) -> DomainResult<Vec<User>>;
    async fn reset_password(
        &self,
        ctx: &Context,
        user_id: i64,
        new_password: String,
    ) -> DomainResult<()>;
    async fn change_my_password(
        &self,
        ctx: &Context,
        old_password: String,
        new_password: String,
    ) -> DomainResult<()>;
    async fn delete(&self, ctx: &Context, user_id: i64) -> DomainResult<()>;
    async fn get_user_permission(
        &self,
        ctx: &Context,
        user_id: i64,
    ) -> DomainResult<Vec<Permission>>;
}

/// User service implementation.
///
/// Provides business logic for user operations with caching support
/// for permissions data.
pub struct UserService<R, P, I, C> {
    repository: R,
    password_hasher: Arc<P>,
    id_generator: I,
    cache: Arc<C>,
    db: DatabaseConnection,
}

impl<R, P, I, C> UserService<R, P, I, C>
where
    R: UserRepository,
    P: PasswordHash,
    I: IdGenerator,
    C: CacheService<i64>,
{
    pub fn new(
        repository: R,
        password_hasher: Arc<P>,
        id_generator: I,
        cache: Arc<C>,
        db: DatabaseConnection,
    ) -> Self {
        Self {
            repository,
            password_hasher,
            id_generator,
            cache,
            db,
        }
    }
}

impl<R, P, I, C> ServiceDbHelper for UserService<R, P, I, C>
where
    R: UserRepository,
    P: PasswordHash,
    I: IdGenerator,
    C: CacheService<i64>,
{
    fn database(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl<R, P, I, C> UserServiceTrait for UserService<R, P, I, C>
where
    R: UserRepository,
    P: PasswordHash + Send + Sync,
    I: IdGenerator,
    C: CacheService<i64>,
{
    async fn create(
        &self,
        ctx: &Context,
        user: &UserCreate,
        permissions: &[PermissionCreate],
    ) -> DomainResult<i64> {
        ctx.require_access(None, resource::USER, action::CREATE)?;
        let password_hash = self.password_hasher.hash_password(&user.password)?;
        let mut user_with_password = user.clone();
        let id = self.id_generator.generate()?;
        user_with_password.password = password_hash;

        let repo_ctx = self.txn_repo_ctx(ctx).await?;
        self.repository
            .create(&repo_ctx, id, &user_with_password)
            .await?;

        self.repository
            .save_permissions(&repo_ctx, id, permissions)
            .await?;

        repo_ctx.db.commit().await?;

        Ok(id)
    }

    async fn update(
        &self,
        ctx: &Context,
        id: i64,
        user: &UserUpdate,
        permissions: Option<Vec<PermissionCreate>>,
    ) -> DomainResult<()> {
        ctx.require_access(None, resource::USER, action::UPDATE)?;

        let repo_ctx = self.txn_repo_ctx(ctx).await?;
        self.repository.update(&repo_ctx, id, user).await?;

        if let Some(perms) = permissions {
            self.repository
                .delete_permission_by_user_id(&repo_ctx, id)
                .await?;
            self.repository
                .save_permissions(&repo_ctx, id, &perms)
                .await?;
        }

        repo_ctx.db.commit().await?;

        let _ = self.cache.delete(&id).await;

        Ok(())
    }

    async fn get_by_id(&self, ctx: &Context, user_id: i64) -> DomainResult<Option<User>> {
        ctx.require_access(None, resource::USER, action::READ)?;

        let repo_ctx = self.repo_ctx(ctx);
        self.repository.get_by_id(&repo_ctx, user_id).await
    }

    async fn get_all(
        &self,
        ctx: &Context,
        filter: &UserFilter,
        pagination: &PaginationOptions,
    ) -> DomainResult<Vec<User>> {
        ctx.require_access(None, resource::USER, action::READ)?;

        let repo_ctx = self.repo_ctx(ctx);
        self.repository.get_all(&repo_ctx, filter, pagination).await
    }

    async fn reset_password(
        &self,
        ctx: &Context,
        user_id: i64,
        new_password: String,
    ) -> DomainResult<()> {
        ctx.require_access(None, resource::USER, action::UPDATE)?;
        let password_hash = self.password_hasher.hash_password(&new_password)?;

        let repo_ctx = self.repo_ctx(ctx);
        self.repository
            .update_password(&repo_ctx, user_id, &password_hash)
            .await?;

        // Invalidate cache when password is reset
        let _ = self.cache.delete(&user_id).await;

        Ok(())
    }

    async fn change_my_password(
        &self,
        ctx: &Context,
        old_password: String,
        new_password: String,
    ) -> DomainResult<()> {
        let user_id = ctx.user_id().ok_or_else(|| {
            crate::domain::Error::BadRequest("User ID not found in context".to_string())
        })?;

        let repo_ctx = self.repo_ctx(ctx);
        let user = self
            .repository
            .get_by_id(&repo_ctx, user_id)
            .await?
            .ok_or_else(|| {
                crate::domain::Error::NotFound(format!("User with id {} not found", user_id))
            })?;

        // Verify old password
        let is_valid = self
            .password_hasher
            .verify_password(&old_password, &user.password)?;
        if !is_valid {
            return Err(crate::domain::Error::BadRequest(
                "Old password is incorrect".to_string(),
            ));
        }

        // Hash new password and update
        let new_password_hash = self.password_hasher.hash_password(&new_password)?;
        self.repository
            .update_password(&repo_ctx, user_id, &new_password_hash)
            .await?;

        // Invalidate cache when password is changed
        let _ = self.cache.delete(&user_id).await;

        Ok(())
    }

    async fn delete(&self, ctx: &Context, user_id: i64) -> DomainResult<()> {
        ctx.require_access(None, resource::USER, action::DELETE)?;

        let repo_ctx = self.repo_ctx(ctx);
        self.repository.delete(&repo_ctx, user_id).await?;

        // Invalidate cache when user is deleted
        let _ = self.cache.delete(&user_id).await;

        Ok(())
    }

    async fn get_user_permission(
        &self,
        ctx: &Context,
        user_id: i64,
    ) -> DomainResult<Vec<Permission>> {
        ctx.require_access(None, resource::USER, action::READ)?;

        // Try to get from cache first
        if let Some(cached_permissions) = self.cache.get::<Vec<Permission>>(&user_id).await {
            return Ok(cached_permissions);
        }

        // Cache miss - fetch from repository
        let repo_ctx = self.repo_ctx(ctx);
        let permissions = self.repository.get_permissions(&repo_ctx, user_id).await?;

        // Store in cache with 5 minute TTL
        let _ = self
            .cache
            .set(&user_id, permissions.clone(), Duration::from_secs(300))
            .await;

        Ok(permissions)
    }
}

// =============================================================================
// Tests use manual mock implementations since mockall doesn't work with
// `impl ConnectionTrait` parameters in the trait. See testing/storage/user.rs
// for test helpers and tests/user_repo.rs for integration tests.
// =============================================================================

#[cfg(test)]
#[allow(clippy::type_complexity)]
mod tests {
    use super::*;
    use crate::application::InMemoryCache;
    use crate::domain::Error;
    use crate::domain::model::pagination::PaginationOptions;
    use crate::domain::model::permission::Permission;
    use crate::domain::model::user::UserFilter;
    use crate::storage::RepoCtx;
    use sea_orm::ConnectionTrait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ==========================================================================
    // Manual Mock Implementations
    // ==========================================================================

    /// Manual mock for UserRepository since mockall doesn't work with `impl Trait`
    struct MockUserRepository {
        create_fn: Mutex<Option<Box<dyn Fn(i64, &UserCreate) -> DomainResult<()> + Send + Sync>>>,
        get_by_id_fn: Mutex<Option<Box<dyn Fn(i64) -> DomainResult<Option<User>> + Send + Sync>>>,
        update_fn: Mutex<Option<Box<dyn Fn(i64, &UserUpdate) -> DomainResult<()> + Send + Sync>>>,
        update_password_fn: Mutex<Option<Box<dyn Fn(i64, &str) -> DomainResult<()> + Send + Sync>>>,
        delete_fn: Mutex<Option<Box<dyn Fn(i64) -> DomainResult<()> + Send + Sync>>>,
        get_permissions_fn:
            Mutex<Option<Box<dyn Fn(i64) -> DomainResult<Vec<Permission>> + Send + Sync>>>,
        save_permissions_fn:
            Mutex<Option<Box<dyn Fn(i64, &[PermissionCreate]) -> DomainResult<()> + Send + Sync>>>,
        delete_permission_by_user_id_fn:
            Mutex<Option<Box<dyn Fn(i64) -> DomainResult<()> + Send + Sync>>>,
        get_all_fn: Mutex<Option<Box<dyn Fn() -> DomainResult<Vec<User>> + Send + Sync>>>,
    }

    impl MockUserRepository {
        fn new() -> Self {
            Self {
                create_fn: Mutex::new(None),
                get_by_id_fn: Mutex::new(None),
                update_fn: Mutex::new(None),
                update_password_fn: Mutex::new(None),
                delete_fn: Mutex::new(None),
                get_permissions_fn: Mutex::new(None),
                save_permissions_fn: Mutex::new(None),
                delete_permission_by_user_id_fn: Mutex::new(None),
                get_all_fn: Mutex::new(None),
            }
        }

        fn expect_create<F>(&self, f: F)
        where
            F: Fn(i64, &UserCreate) -> DomainResult<()> + Send + Sync + 'static,
        {
            *self.create_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_get_by_id<F>(&self, f: F)
        where
            F: Fn(i64) -> DomainResult<Option<User>> + Send + Sync + 'static,
        {
            *self.get_by_id_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_update<F>(&self, f: F)
        where
            F: Fn(i64, &UserUpdate) -> DomainResult<()> + Send + Sync + 'static,
        {
            *self.update_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_update_password<F>(&self, f: F)
        where
            F: Fn(i64, &str) -> DomainResult<()> + Send + Sync + 'static,
        {
            *self.update_password_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_delete<F>(&self, f: F)
        where
            F: Fn(i64) -> DomainResult<()> + Send + Sync + 'static,
        {
            *self.delete_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_get_permissions<F>(&self, f: F)
        where
            F: Fn(i64) -> DomainResult<Vec<Permission>> + Send + Sync + 'static,
        {
            *self.get_permissions_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_save_permissions<F>(&self, f: F)
        where
            F: Fn(i64, &[PermissionCreate]) -> DomainResult<()> + Send + Sync + 'static,
        {
            *self.save_permissions_fn.lock().unwrap() = Some(Box::new(f));
        }

        #[allow(dead_code)]
        fn expect_delete_permission_by_user_id<F>(&self, f: F)
        where
            F: Fn(i64) -> DomainResult<()> + Send + Sync + 'static,
        {
            *self.delete_permission_by_user_id_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_get_all<F>(&self, f: F)
        where
            F: Fn() -> DomainResult<Vec<User>> + Send + Sync + 'static,
        {
            *self.get_all_fn.lock().unwrap() = Some(Box::new(f));
        }
    }

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn create(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
            user: &UserCreate,
        ) -> DomainResult<()> {
            let guard = self.create_fn.lock().unwrap();
            if let Some(f) = guard.as_ref() {
                f(id, user)
            } else {
                panic!("create not mocked")
            }
        }

        async fn get_by_username(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            _username: &str,
        ) -> DomainResult<Option<User>> {
            panic!("get_by_username not mocked")
        }

        async fn update(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
            user: &UserUpdate,
        ) -> DomainResult<()> {
            let guard = self.update_fn.lock().unwrap();
            if let Some(f) = guard.as_ref() {
                f(id, user)
            } else {
                panic!("update not mocked")
            }
        }

        async fn update_password(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
            password_hash: &str,
        ) -> DomainResult<()> {
            let guard = self.update_password_fn.lock().unwrap();
            if let Some(f) = guard.as_ref() {
                f(id, password_hash)
            } else {
                panic!("update_password not mocked")
            }
        }

        async fn delete(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            user_id: i64,
        ) -> DomainResult<()> {
            let guard = self.delete_fn.lock().unwrap();
            if let Some(f) = guard.as_ref() {
                f(user_id)
            } else {
                panic!("delete not mocked")
            }
        }

        async fn get_all(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            _filter: &UserFilter,
            _pagination: &PaginationOptions,
        ) -> DomainResult<Vec<User>> {
            let guard = self.get_all_fn.lock().unwrap();
            if let Some(f) = guard.as_ref() {
                f()
            } else {
                panic!("get_all not mocked")
            }
        }

        async fn get_by_id(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            user_id: i64,
        ) -> DomainResult<Option<User>> {
            let guard = self.get_by_id_fn.lock().unwrap();
            if let Some(f) = guard.as_ref() {
                f(user_id)
            } else {
                panic!("get_by_id not mocked")
            }
        }

        async fn delete_permission_by_user_id(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            user_id: i64,
        ) -> DomainResult<()> {
            let guard = self.delete_permission_by_user_id_fn.lock().unwrap();
            if let Some(f) = guard.as_ref() {
                f(user_id)
            } else {
                Ok(()) // Default to success for this optional operation
            }
        }

        async fn save_permissions(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            user_id: i64,
            permissions: &[PermissionCreate],
        ) -> DomainResult<()> {
            let guard = self.save_permissions_fn.lock().unwrap();
            if let Some(f) = guard.as_ref() {
                f(user_id, permissions)
            } else {
                Ok(()) // Default to success for this optional operation
            }
        }

        async fn get_permissions(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            user_id: i64,
        ) -> DomainResult<Vec<Permission>> {
            let guard = self.get_permissions_fn.lock().unwrap();
            if let Some(f) = guard.as_ref() {
                f(user_id)
            } else {
                panic!("get_permissions not mocked")
            }
        }
    }

    /// Mock password hasher
    struct MockPasswordHasher {
        hash_fn: Mutex<Option<Box<dyn Fn(&str) -> DomainResult<String> + Send + Sync>>>,
        verify_fn: Mutex<Option<Box<dyn Fn(&str, &str) -> DomainResult<bool> + Send + Sync>>>,
    }

    impl MockPasswordHasher {
        fn new() -> Self {
            Self {
                hash_fn: Mutex::new(None),
                verify_fn: Mutex::new(None),
            }
        }

        fn expect_hash<F>(&self, f: F)
        where
            F: Fn(&str) -> DomainResult<String> + Send + Sync + 'static,
        {
            *self.hash_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_verify<F>(&self, f: F)
        where
            F: Fn(&str, &str) -> DomainResult<bool> + Send + Sync + 'static,
        {
            *self.verify_fn.lock().unwrap() = Some(Box::new(f));
        }
    }

    impl PasswordHash for MockPasswordHasher {
        fn hash_password(&self, password: &str) -> DomainResult<String> {
            let guard = self.hash_fn.lock().unwrap();
            if let Some(f) = guard.as_ref() {
                f(password)
            } else {
                Ok(format!("hashed_{}", password))
            }
        }

        fn verify_password(&self, password: &str, hash: &str) -> DomainResult<bool> {
            let guard = self.verify_fn.lock().unwrap();
            if let Some(f) = guard.as_ref() {
                f(password, hash)
            } else {
                Ok(true)
            }
        }
    }

    /// Mock ID generator
    struct MockIdGenerator {
        id: i64,
    }

    impl MockIdGenerator {
        fn new(id: i64) -> Self {
            Self { id }
        }
    }

    impl IdGenerator for MockIdGenerator {
        fn generate(&self) -> Result<i64, crate::snowflake::SnowflakeError> {
            Ok(self.id)
        }
    }

    // ==========================================================================
    // Test Helpers
    // ==========================================================================

    async fn create_test_db() -> DatabaseConnection {
        sea_orm::Database::connect("sqlite::memory:")
            .await
            .expect("Failed to connect to test database")
    }

    fn create_test_context() -> Context {
        let mut permissions = HashMap::new();
        permissions.insert((resource::USER, None), 0b1111);
        Context::new_with_all(None, permissions, HashMap::new())
    }

    fn create_no_permission_context() -> Context {
        Context::new()
    }

    fn create_test_user() -> UserCreate {
        UserCreate {
            username: "testuser".to_string(),
            name: "Test User".to_string(),
            email: Some("test@example.com".to_string()),
            password: "plainpassword".to_string(),
            photo: None,
            pin: None,
            address: None,
            phone: None,
        }
    }

    fn create_full_user() -> User {
        User {
            id: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
            is_deleted: false,
            username: "testuser".to_string(),
            password: "hashed_password".to_string(),
            name: "Test User".to_string(),
            email: Some("test@example.com".to_string()),
            photo: None,
            pin: None,
            address: None,
            phone: None,
            permissions: None,
        }
    }

    fn create_user_update() -> UserUpdate {
        use crate::domain::model::Update;
        UserUpdate {
            username: Some("updated_user".to_string()),
            name: Some("Updated User".to_string()),
            email: Update::Unchanged,
            photo: Update::Unchanged,
            pin: Update::Unchanged,
            address: Update::Unchanged,
            phone: Update::Unchanged,
        }
    }

    // ==========================================================================
    // Create Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_create_user_success() {
        let mock_repo = MockUserRepository::new();
        let mock_hasher = MockPasswordHasher::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_hasher.expect_hash(|p| {
            if p == "plainpassword" {
                Ok("hashed_password".to_string())
            } else {
                Err(Error::Internal("Unexpected password".to_string()))
            }
        });

        mock_repo.expect_create(|id, user| {
            assert_eq!(id, 12345);
            assert_eq!(user.password, "hashed_password");
            Ok(())
        });

        mock_repo.expect_save_permissions(|id, perms| {
            assert_eq!(id, 12345);
            assert_eq!(perms.len(), 1);
            assert_eq!(perms[0].resource, 1);
            assert_eq!(perms[0].action, 2);
            Ok(())
        });

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            MockIdGenerator::new(12345),
            Arc::new(InMemoryCache::<i64>::new()),
            db,
        );
        let permissions = [PermissionCreate {
            branch_id: None,
            resource: 1,
            action: 2,
        }];

        let user = create_test_user();
        let result = service.create(&ctx, &user, &permissions).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 12345);
    }

    #[tokio::test]
    async fn test_create_user_no_permission() {
        let mock_repo = MockUserRepository::new();
        let mock_hasher = MockPasswordHasher::new();
        let db = create_test_db().await;
        let ctx = create_no_permission_context();

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            MockIdGenerator::new(12345),
            Arc::new(InMemoryCache::<i64>::new()),
            db,
        );

        let user = create_test_user();
        let permissions = [PermissionCreate {
            branch_id: None,
            resource: 1,
            action: 2,
        }];
        let result = service.create(&ctx, &user, &permissions).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_create_user_hash_error() {
        let mock_repo = MockUserRepository::new();
        let mock_hasher = MockPasswordHasher::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_hasher.expect_hash(|_| Err(Error::Internal("Hash failed".to_string())));

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            MockIdGenerator::new(12345),
            Arc::new(InMemoryCache::<i64>::new()),
            db,
        );

        let user = create_test_user();
        let permissions = [PermissionCreate {
            branch_id: None,
            resource: 1,
            action: 2,
        }];
        let result = service.create(&ctx, &user, &permissions).await;

        assert!(matches!(result, Err(Error::Internal(_))));
    }

    #[tokio::test]
    async fn test_create_user_repo_error() {
        let mock_repo = MockUserRepository::new();
        let mock_hasher = MockPasswordHasher::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_hasher.expect_hash(|_| Ok("hashed".to_string()));
        mock_repo.expect_create(|_, _| Err(Error::Database("DB Error".to_string())));

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            MockIdGenerator::new(12345),
            Arc::new(InMemoryCache::<i64>::new()),
            db,
        );

        let user = create_test_user();
        let permissions = [PermissionCreate {
            branch_id: None,
            resource: 1,
            action: 2,
        }];
        let result = service.create(&ctx, &user, &permissions).await;

        assert!(matches!(result, Err(Error::Database(_))));
    }

    // ==========================================================================
    // Update Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_update_user_success() {
        let mock_repo = MockUserRepository::new();
        let mock_hasher = MockPasswordHasher::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_update(|id, _| {
            assert_eq!(id, 1);
            Ok(())
        });

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            MockIdGenerator::new(12345),
            Arc::new(InMemoryCache::<i64>::new()),
            db,
        );

        let user = create_user_update();
        let result = service.update(&ctx, 1, &user, None).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_user_no_permission() {
        let mock_repo = MockUserRepository::new();
        let mock_hasher = MockPasswordHasher::new();
        let db = create_test_db().await;
        let ctx = create_no_permission_context();

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            MockIdGenerator::new(12345),
            Arc::new(InMemoryCache::<i64>::new()),
            db,
        );

        let user = create_user_update();
        let result = service.update(&ctx, 1, &user, None).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    // ==========================================================================
    // Get By Id Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_get_by_id_success() {
        let mock_repo = MockUserRepository::new();
        let mock_hasher = MockPasswordHasher::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        let expected_user = create_full_user();
        let user_clone = expected_user.clone();
        mock_repo.expect_get_by_id(move |id| {
            assert_eq!(id, 1);
            Ok(Some(user_clone.clone()))
        });

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            MockIdGenerator::new(12345),
            Arc::new(InMemoryCache::<i64>::new()),
            db,
        );

        let result = service.get_by_id(&ctx, 1).await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert!(user.is_some());
        assert_eq!(user.unwrap().username, expected_user.username);
    }

    #[tokio::test]
    async fn test_get_by_id_not_found() {
        let mock_repo = MockUserRepository::new();
        let mock_hasher = MockPasswordHasher::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_get_by_id(|_| Ok(None));

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            MockIdGenerator::new(12345),
            Arc::new(InMemoryCache::<i64>::new()),
            db,
        );

        let result = service.get_by_id(&ctx, 999).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_by_id_no_permission() {
        let mock_repo = MockUserRepository::new();
        let mock_hasher = MockPasswordHasher::new();
        let db = create_test_db().await;
        let ctx = create_no_permission_context();

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            MockIdGenerator::new(12345),
            Arc::new(InMemoryCache::<i64>::new()),
            db,
        );

        let result = service.get_by_id(&ctx, 1).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    // ==========================================================================
    // Reset Password Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_reset_password_success() {
        let mock_repo = MockUserRepository::new();
        let mock_hasher = MockPasswordHasher::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_hasher.expect_hash(|p| {
            if p == "newpassword" {
                Ok("new_hashed_password".to_string())
            } else {
                Err(Error::Internal("Unexpected password".to_string()))
            }
        });

        mock_repo.expect_update_password(|id, hash| {
            assert_eq!(id, 1);
            assert_eq!(hash, "new_hashed_password");
            Ok(())
        });

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            MockIdGenerator::new(12345),
            Arc::new(InMemoryCache::<i64>::new()),
            db,
        );

        let result = service
            .reset_password(&ctx, 1, "newpassword".to_string())
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reset_password_no_permission() {
        let mock_repo = MockUserRepository::new();
        let mock_hasher = MockPasswordHasher::new();
        let db = create_test_db().await;
        let ctx = create_no_permission_context();

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            MockIdGenerator::new(12345),
            Arc::new(InMemoryCache::<i64>::new()),
            db,
        );

        let result = service
            .reset_password(&ctx, 1, "newpassword".to_string())
            .await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_reset_password_hash_error() {
        let mock_repo = MockUserRepository::new();
        let mock_hasher = MockPasswordHasher::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_hasher.expect_hash(|_| Err(Error::Internal("Hash failed".to_string())));

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            MockIdGenerator::new(12345),
            Arc::new(InMemoryCache::<i64>::new()),
            db,
        );

        let result = service
            .reset_password(&ctx, 1, "newpassword".to_string())
            .await;

        assert!(matches!(result, Err(Error::Internal(_))));
    }

    // ==========================================================================
    // Delete Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_delete_user_success() {
        let mock_repo = MockUserRepository::new();
        let mock_hasher = MockPasswordHasher::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_delete(|id| {
            assert_eq!(id, 1);
            Ok(())
        });

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            MockIdGenerator::new(12345),
            Arc::new(InMemoryCache::<i64>::new()),
            db,
        );

        let result = service.delete(&ctx, 1).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_user_no_permission() {
        let mock_repo = MockUserRepository::new();
        let mock_hasher = MockPasswordHasher::new();
        let db = create_test_db().await;
        let ctx = create_no_permission_context();

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            MockIdGenerator::new(12345),
            Arc::new(InMemoryCache::<i64>::new()),
            db,
        );

        let result = service.delete(&ctx, 1).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_delete_user_repo_error() {
        let mock_repo = MockUserRepository::new();
        let mock_hasher = MockPasswordHasher::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_delete(|_| Err(Error::Database("DB Error".to_string())));

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            MockIdGenerator::new(12345),
            Arc::new(InMemoryCache::<i64>::new()),
            db,
        );

        let result = service.delete(&ctx, 1).await;

        assert!(matches!(result, Err(Error::Database(_))));
    }

    // ==========================================================================
    // Permission Caching Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_get_user_permission_caching() {
        let mock_repo = MockUserRepository::new();
        let mock_hasher = MockPasswordHasher::new();
        let db = create_test_db().await;
        let ctx = create_test_context();
        let cache = Arc::new(InMemoryCache::<i64>::new());

        // Set up the mock to track call count using a flag
        let expected_permissions = vec![
            Permission {
                user_id: 1,
                branch_id: None,
                resource: resource::USER,
                action: action::READ,
            },
            Permission {
                user_id: 1,
                branch_id: None,
                resource: resource::USER,
                action: action::CREATE,
            },
        ];
        let perms_clone = expected_permissions.clone();
        mock_repo.expect_get_permissions(move |_| Ok(perms_clone.clone()));

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            MockIdGenerator::new(12345),
            cache.clone(),
            db,
        );

        // First call - should hit repository
        let result1 = service.get_user_permission(&ctx, 1).await;
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap().len(), 2);

        // Second call - should hit cache
        let result2 = service.get_user_permission(&ctx, 1).await;
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_cache_invalidation_on_update() {
        let mock_repo = MockUserRepository::new();
        let mock_hasher = MockPasswordHasher::new();
        let db = create_test_db().await;
        let ctx = create_test_context();
        let cache = Arc::new(InMemoryCache::<i64>::new());

        // Pre-populate cache
        let permissions = vec![Permission {
            user_id: 1,
            branch_id: None,
            resource: resource::USER,
            action: action::READ,
        }];
        cache
            .set(&1i64, permissions.clone(), Duration::from_secs(300))
            .await
            .unwrap();

        // Verify cache has data
        let cached: Option<Vec<Permission>> = cache.get(&1i64).await;
        assert!(cached.is_some());

        mock_repo.expect_update(|_, _| Ok(()));

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            MockIdGenerator::new(12345),
            cache.clone(),
            db,
        );

        // Update user - should invalidate cache
        let user = create_user_update();
        let result = service.update(&ctx, 1, &user, None).await;
        assert!(result.is_ok());

        // Cache should be cleared
        let cached: Option<Vec<Permission>> = cache.get(&1i64).await;
        assert!(cached.is_none());
    }

    #[tokio::test]
    async fn test_cache_invalidation_on_delete() {
        let mock_repo = MockUserRepository::new();
        let mock_hasher = MockPasswordHasher::new();
        let db = create_test_db().await;
        let ctx = create_test_context();
        let cache = Arc::new(InMemoryCache::<i64>::new());

        // Pre-populate cache
        let permissions = vec![Permission {
            user_id: 1,
            branch_id: None,
            resource: resource::USER,
            action: action::READ,
        }];
        cache
            .set(&1i64, permissions.clone(), Duration::from_secs(300))
            .await
            .unwrap();

        mock_repo.expect_delete(|_| Ok(()));

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            MockIdGenerator::new(12345),
            cache.clone(),
            db,
        );

        // Delete user - should invalidate cache
        let result = service.delete(&ctx, 1).await;
        assert!(result.is_ok());

        // Cache should be cleared
        let cached: Option<Vec<Permission>> = cache.get(&1i64).await;
        assert!(cached.is_none());
    }

    #[tokio::test]
    async fn test_cache_invalidation_on_reset_password() {
        let mock_repo = MockUserRepository::new();
        let mock_hasher = MockPasswordHasher::new();
        let db = create_test_db().await;
        let ctx = create_test_context();
        let cache = Arc::new(InMemoryCache::<i64>::new());

        // Pre-populate cache
        let permissions = vec![Permission {
            user_id: 1,
            branch_id: None,
            resource: resource::USER,
            action: action::READ,
        }];
        cache
            .set(&1i64, permissions.clone(), Duration::from_secs(300))
            .await
            .unwrap();

        mock_hasher.expect_hash(|_| Ok("new_hash".to_string()));
        mock_repo.expect_update_password(|_, _| Ok(()));

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            MockIdGenerator::new(12345),
            cache.clone(),
            db,
        );

        // Reset password - should invalidate cache
        let result = service.reset_password(&ctx, 1, "newpass".to_string()).await;
        assert!(result.is_ok());

        // Cache should be cleared
        let cached: Option<Vec<Permission>> = cache.get(&1i64).await;
        assert!(cached.is_none());
    }

    // ==========================================================================
    // Get All Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_get_all_success() {
        let mock_repo = MockUserRepository::new();
        let mock_hasher = MockPasswordHasher::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        let users = vec![create_full_user()];
        let users_clone = users.clone();
        mock_repo.expect_get_all(move || Ok(users_clone.clone()));

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            MockIdGenerator::new(12345),
            Arc::new(InMemoryCache::<i64>::new()),
            db,
        );

        let filter = UserFilter::default();
        let pagination = PaginationOptions::new(1, 20, None);
        let result = service.get_all(&ctx, &filter, &pagination).await;

        assert!(result.is_ok());
        let user_list = result.unwrap();
        assert_eq!(user_list.len(), 1);
        assert_eq!(user_list[0].username, "testuser");
    }

    #[tokio::test]
    async fn test_get_all_no_permission() {
        let mock_repo = MockUserRepository::new();
        let mock_hasher = MockPasswordHasher::new();
        let db = create_test_db().await;
        let ctx = create_no_permission_context();

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            MockIdGenerator::new(12345),
            Arc::new(InMemoryCache::<i64>::new()),
            db,
        );

        let filter = UserFilter::default();
        let pagination = PaginationOptions::new(1, 20, None);
        let result = service.get_all(&ctx, &filter, &pagination).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_get_all_empty() {
        let mock_repo = MockUserRepository::new();
        let mock_hasher = MockPasswordHasher::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_get_all(|| Ok(vec![]));

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            MockIdGenerator::new(12345),
            Arc::new(InMemoryCache::<i64>::new()),
            db,
        );

        let filter = UserFilter::default();
        let pagination = PaginationOptions::new(1, 20, None);
        let result = service.get_all(&ctx, &filter, &pagination).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    // ==========================================================================
    // Change My Password Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_change_my_password_success() {
        let mock_repo = MockUserRepository::new();
        let mock_hasher = MockPasswordHasher::new();
        let db = create_test_db().await;
        let mut permissions = HashMap::new();
        permissions.insert((resource::USER, None), 0b1111);
        let ctx = Context::new_with_all(Some(1), permissions, HashMap::new());

        let user = create_full_user();
        let user_clone = user.clone();
        mock_repo.expect_get_by_id(move |id| {
            assert_eq!(id, 1);
            Ok(Some(user_clone.clone()))
        });

        mock_hasher.expect_verify(|password, hash| {
            assert_eq!(password, "oldpassword");
            assert_eq!(hash, "hashed_password");
            Ok(true)
        });

        mock_hasher.expect_hash(|password| {
            assert_eq!(password, "newpassword");
            Ok("new_hashed_password".to_string())
        });

        mock_repo.expect_update_password(|id, hash| {
            assert_eq!(id, 1);
            assert_eq!(hash, "new_hashed_password");
            Ok(())
        });

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            MockIdGenerator::new(12345),
            Arc::new(InMemoryCache::<i64>::new()),
            db,
        );

        let result = service
            .change_my_password(&ctx, "oldpassword".to_string(), "newpassword".to_string())
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_change_my_password_no_user_id_in_context() {
        let mock_repo = MockUserRepository::new();
        let mock_hasher = MockPasswordHasher::new();
        let db = create_test_db().await;
        let ctx = Context::new();

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            MockIdGenerator::new(12345),
            Arc::new(InMemoryCache::<i64>::new()),
            db,
        );

        let result = service
            .change_my_password(&ctx, "oldpassword".to_string(), "newpassword".to_string())
            .await;

        assert!(matches!(result, Err(Error::BadRequest(_))));
    }

    #[tokio::test]
    async fn test_change_my_password_user_not_found() {
        let mock_repo = MockUserRepository::new();
        let mock_hasher = MockPasswordHasher::new();
        let db = create_test_db().await;
        let ctx = Context::new_with_all(Some(1), HashMap::new(), HashMap::new());

        mock_repo.expect_get_by_id(|_| Ok(None));

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            MockIdGenerator::new(12345),
            Arc::new(InMemoryCache::<i64>::new()),
            db,
        );

        let result = service
            .change_my_password(&ctx, "oldpassword".to_string(), "newpassword".to_string())
            .await;

        assert!(matches!(result, Err(Error::NotFound(_))));
    }

    #[tokio::test]
    async fn test_change_my_password_wrong_old_password() {
        let mock_repo = MockUserRepository::new();
        let mock_hasher = MockPasswordHasher::new();
        let db = create_test_db().await;
        let ctx = Context::new_with_all(Some(1), HashMap::new(), HashMap::new());

        let user = create_full_user();
        let user_clone = user.clone();
        mock_repo.expect_get_by_id(move |_| Ok(Some(user_clone.clone())));

        mock_hasher.expect_verify(|_, _| Ok(false));

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            MockIdGenerator::new(12345),
            Arc::new(InMemoryCache::<i64>::new()),
            db,
        );

        let result = service
            .change_my_password(&ctx, "wrongpassword".to_string(), "newpassword".to_string())
            .await;

        assert!(matches!(result, Err(Error::BadRequest(_))));
        if let Err(Error::BadRequest(msg)) = result {
            assert_eq!(msg, "Old password is incorrect");
        }
    }

    #[tokio::test]
    async fn test_change_my_password_hash_error() {
        let mock_repo = MockUserRepository::new();
        let mock_hasher = MockPasswordHasher::new();
        let db = create_test_db().await;
        let ctx = Context::new_with_all(Some(1), HashMap::new(), HashMap::new());

        let user = create_full_user();
        let user_clone = user.clone();
        mock_repo.expect_get_by_id(move |_| Ok(Some(user_clone.clone())));

        mock_hasher.expect_verify(|_, _| Ok(true));
        mock_hasher.expect_hash(|_| Err(Error::Internal("Hash failed".to_string())));

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            MockIdGenerator::new(12345),
            Arc::new(InMemoryCache::<i64>::new()),
            db,
        );

        let result = service
            .change_my_password(&ctx, "oldpassword".to_string(), "newpassword".to_string())
            .await;

        assert!(matches!(result, Err(Error::Internal(_))));
    }
}
