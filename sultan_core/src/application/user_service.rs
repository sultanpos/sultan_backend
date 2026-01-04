use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

use crate::application::cache::CacheService;
use crate::crypto::password::PasswordHash;
use crate::domain::model::permission::{Permission, action, resource};
use crate::domain::model::user::{UserCreate, UserUpdate};
use crate::domain::{Context, DomainResult, User};
use crate::snowflake::IdGenerator;
use crate::storage::user_repo::UserRepository;

#[async_trait]
pub trait UserServiceTrait: Send + Sync {
    async fn create(&self, ctx: &Context, user: &UserCreate) -> DomainResult<()>;
    async fn update(&self, ctx: &Context, id: i64, user: &UserUpdate) -> DomainResult<()>;
    async fn get_by_id(&self, ctx: &Context, user_id: i64) -> DomainResult<Option<User>>;
    async fn reset_password(
        &self,
        ctx: &Context,
        user_id: i64,
        new_password: String,
    ) -> DomainResult<()>;
    async fn delete(&self, ctx: &Context, user_id: i64) -> DomainResult<()>;
    async fn get_user_permission(
        &self,
        ctx: &Context,
        user_id: i64,
    ) -> DomainResult<Vec<Permission>>;
}

pub struct UserService<R, P, I, C, Tx> {
    password_hasher: Arc<P>,
    repository: R,
    id_generator: I,
    cache: Arc<C>,
    _phantom: std::marker::PhantomData<Tx>,
}

impl<R: UserRepository<Tx>, P: PasswordHash, I: IdGenerator, C: CacheService<i64>, Tx: Send + Sync>
    UserService<R, P, I, C, Tx>
{
    pub fn new(repository: R, password_hasher: Arc<P>, id_generator: I, cache: Arc<C>) -> Self {
        Self {
            repository,
            password_hasher,
            id_generator,
            cache,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<R, P, I, C, Tx> UserServiceTrait for UserService<R, P, I, C, Tx>
where
    R: UserRepository<Tx>,
    P: PasswordHash + Send + Sync,
    I: IdGenerator,
    C: CacheService<i64>,
    Tx: Send + Sync,
{
    async fn create(&self, ctx: &Context, user: &UserCreate) -> DomainResult<()> {
        ctx.require_access(None, resource::USER, action::CREATE)?;
        let password_hash = self.password_hasher.hash_password(&user.password)?;
        let mut user_with_password = user.clone();
        let id = self.id_generator.generate()?;
        user_with_password.password = password_hash;
        self.repository
            .create_user(ctx, id, &user_with_password)
            .await?;

        // Invalidate cache for new user
        let _ = self.cache.delete(&id).await;

        Ok(())
    }

    async fn update(&self, ctx: &Context, id: i64, user: &UserUpdate) -> DomainResult<()> {
        ctx.require_access(None, resource::USER, action::UPDATE)?;
        self.repository.update_user(ctx, id, user).await?;

        // Invalidate cache when user is updated
        let _ = self.cache.delete(&id).await;

        Ok(())
    }

    async fn get_by_id(&self, ctx: &Context, user_id: i64) -> DomainResult<Option<User>> {
        ctx.require_access(None, resource::USER, action::READ)?;
        self.repository.get_by_id(ctx, user_id).await
    }

    async fn reset_password(
        &self,
        ctx: &Context,
        user_id: i64,
        new_password: String,
    ) -> DomainResult<()> {
        ctx.require_access(None, resource::USER, action::UPDATE)?;
        let password_hash = self.password_hasher.hash_password(&new_password)?;
        self.repository
            .update_password(ctx, user_id, &password_hash)
            .await?;

        // Invalidate cache when password is reset
        let _ = self.cache.delete(&user_id).await;

        Ok(())
    }

    async fn delete(&self, ctx: &Context, user_id: i64) -> DomainResult<()> {
        ctx.require_access(None, resource::USER, action::DELETE)?;
        self.repository.delete_user(ctx, user_id).await?;

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
        let permissions = self.repository.get_user_permission(ctx, user_id).await?;

        // Store in cache with 5 minute TTL
        let _ = self
            .cache
            .set(&user_id, permissions.clone(), Duration::from_secs(300))
            .await;

        Ok(permissions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::InMemoryCache;
    use crate::domain::Error;
    use crate::domain::model::pagination::PaginationOptions;
    use crate::domain::model::permission::Permission;
    use crate::domain::model::user::UserFilter;
    use async_trait::async_trait;
    use chrono::Utc;
    use mockall::mock;
    use std::collections::HashMap;

    // Use a unit type for mock transaction since mocks don't use real transactions
    mock! {
        pub UserRepo {}
        #[async_trait]
        impl UserRepository<()> for UserRepo {
            async fn create_user(&self, ctx: &Context, id: i64, user: &UserCreate) -> DomainResult<()>;
            async fn create_user_tx(&self, ctx: &Context, id: i64, user: &UserCreate, tx: &mut ()) -> DomainResult<()>;
            async fn get_user_by_username(&self, ctx: &Context, username: &str) -> DomainResult<Option<User>>;
            async fn update_user(&self, ctx: &Context, id: i64, user: &UserUpdate) -> DomainResult<()>;
            async fn update_password(&self, ctx: &Context, id: i64, password_hash: &str) -> DomainResult<()>;
            async fn delete_user(&self, ctx: &Context, user_id: i64) -> DomainResult<()>;
            async fn delete_user_tx(&self, ctx: &Context, user_id: i64, tx: &mut ()) -> DomainResult<()>;
            async fn get_all(&self, ctx: &Context, filter: UserFilter, pagination: PaginationOptions) -> DomainResult<Vec<User>>;
            async fn get_by_id(&self, ctx: &Context, user_id: i64) -> DomainResult<Option<User>>;
            async fn save_user_permission(&self, ctx: &Context, user_id: i64, branch_id: Option<i64>, permission: i32, action: i32) -> DomainResult<()>;
            async fn delete_user_permission(&self, ctx: &Context, user_id: i64, branch_id: Option<i64>, permission: i32) -> DomainResult<()>;
            async fn get_user_permission(&self, ctx: &Context, user_id: i64) -> DomainResult<Vec<Permission>>;
        }
    }

    mock! {
        pub Hasher {}
        impl PasswordHash for Hasher {
            fn hash_password(&self, password: &str) -> DomainResult<String>;
            fn verify_password(&self, password: &str, hash: &str) -> DomainResult<bool>;
        }
    }

    mock! {
        pub IdGen {}
        impl IdGenerator for IdGen {
            fn generate(&self) -> Result<i64, crate::snowflake::SnowflakeError>;
        }
    }

    fn create_mock_id_gen() -> MockIdGen {
        let mut mock = MockIdGen::new();
        mock.expect_generate().returning(|| Ok(12345));
        mock
    }

    /// Creates a test context with full permissions for USER resource
    fn create_test_context() -> Context {
        let mut permissions = HashMap::new();
        permissions.insert((resource::USER, None), 0b1111);
        Context::new_with_all(None, permissions, HashMap::new())
    }

    /// Creates a test context with no permissions
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
            created_at: Utc::now(),
            updated_at: Utc::now(),
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

    #[tokio::test]
    async fn test_create_user_success() {
        let mut mock_repo = MockUserRepo::new();
        let mut mock_hasher = MockHasher::new();
        let ctx = create_test_context();

        mock_hasher
            .expect_hash_password()
            .withf(|p| p == "plainpassword")
            .times(1)
            .returning(|_| Ok("hashed_password".to_string()));

        mock_repo
            .expect_create_user()
            .withf(|_, _, user| user.password == "hashed_password")
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            create_mock_id_gen(),
            Arc::new(InMemoryCache::<i64>::new()),
        );
        let user = create_test_user();
        let result = service.create(&ctx, &user).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_user_no_permission() {
        let mock_repo = MockUserRepo::new();
        let mock_hasher = MockHasher::new();
        let ctx = create_no_permission_context();

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            create_mock_id_gen(),
            Arc::new(InMemoryCache::<i64>::new()),
        );
        let user = create_test_user();
        let result = service.create(&ctx, &user).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_create_user_hash_error() {
        let mock_repo = MockUserRepo::new();
        let mut mock_hasher = MockHasher::new();
        let ctx = create_test_context();

        mock_hasher
            .expect_hash_password()
            .times(1)
            .returning(|_| Err(Error::Internal("Hash failed".to_string())));

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            create_mock_id_gen(),
            Arc::new(InMemoryCache::<i64>::new()),
        );
        let user = create_test_user();
        let result = service.create(&ctx, &user).await;

        assert!(matches!(result, Err(Error::Internal(_))));
    }

    #[tokio::test]
    async fn test_create_user_repo_error() {
        let mut mock_repo = MockUserRepo::new();
        let mut mock_hasher = MockHasher::new();
        let ctx = create_test_context();

        mock_hasher
            .expect_hash_password()
            .times(1)
            .returning(|_| Ok("hashed".to_string()));

        mock_repo
            .expect_create_user()
            .times(1)
            .returning(|_, _, _| Err(Error::Database("DB Error".to_string())));

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            create_mock_id_gen(),
            Arc::new(InMemoryCache::<i64>::new()),
        );
        let user = create_test_user();
        let result = service.create(&ctx, &user).await;

        assert!(matches!(result, Err(Error::Database(_))));
    }

    #[tokio::test]
    async fn test_update_user_success() {
        let mut mock_repo = MockUserRepo::new();
        let mock_hasher = MockHasher::new();
        let ctx = create_test_context();

        mock_repo
            .expect_update_user()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(1),
                mockall::predicate::always(),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            create_mock_id_gen(),
            Arc::new(InMemoryCache::<i64>::new()),
        );
        let user = create_user_update();
        let result = service.update(&ctx, 1, &user).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_user_no_permission() {
        let mock_repo = MockUserRepo::new();
        let mock_hasher = MockHasher::new();
        let ctx = create_no_permission_context();

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            create_mock_id_gen(),
            Arc::new(InMemoryCache::<i64>::new()),
        );
        let user = create_user_update();
        let result = service.update(&ctx, 1, &user).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_get_by_id_success() {
        let mut mock_repo = MockUserRepo::new();
        let mock_hasher = MockHasher::new();
        let ctx = create_test_context();

        let expected_user = create_full_user();
        let user_clone = expected_user.clone();

        mock_repo
            .expect_get_by_id()
            .with(mockall::predicate::always(), mockall::predicate::eq(1))
            .times(1)
            .returning(move |_, _| Ok(Some(user_clone.clone())));

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            create_mock_id_gen(),
            Arc::new(InMemoryCache::<i64>::new()),
        );
        let result = service.get_by_id(&ctx, 1).await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert!(user.is_some());
        assert_eq!(user.unwrap().username, expected_user.username);
    }

    #[tokio::test]
    async fn test_get_by_id_not_found() {
        let mut mock_repo = MockUserRepo::new();
        let mock_hasher = MockHasher::new();
        let ctx = create_test_context();

        mock_repo
            .expect_get_by_id()
            .times(1)
            .returning(|_, _| Ok(None));

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            create_mock_id_gen(),
            Arc::new(InMemoryCache::<i64>::new()),
        );
        let result = service.get_by_id(&ctx, 999).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_by_id_no_permission() {
        let mock_repo = MockUserRepo::new();
        let mock_hasher = MockHasher::new();
        let ctx = create_no_permission_context();

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            create_mock_id_gen(),
            Arc::new(InMemoryCache::<i64>::new()),
        );
        let result = service.get_by_id(&ctx, 1).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_reset_password_success() {
        let mut mock_repo = MockUserRepo::new();
        let mut mock_hasher = MockHasher::new();
        let ctx = create_test_context();

        mock_hasher
            .expect_hash_password()
            .withf(|p| p == "newpassword")
            .times(1)
            .returning(|_| Ok("new_hashed_password".to_string()));

        mock_repo
            .expect_update_password()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(1),
                mockall::predicate::eq("new_hashed_password"),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            create_mock_id_gen(),
            Arc::new(InMemoryCache::<i64>::new()),
        );
        let result = service
            .reset_password(&ctx, 1, "newpassword".to_string())
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reset_password_no_permission() {
        let mock_repo = MockUserRepo::new();
        let mock_hasher = MockHasher::new();
        let ctx = create_no_permission_context();

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            create_mock_id_gen(),
            Arc::new(InMemoryCache::<i64>::new()),
        );
        let result = service
            .reset_password(&ctx, 1, "newpassword".to_string())
            .await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_reset_password_hash_error() {
        let mock_repo = MockUserRepo::new();
        let mut mock_hasher = MockHasher::new();
        let ctx = create_test_context();

        mock_hasher
            .expect_hash_password()
            .times(1)
            .returning(|_| Err(Error::Internal("Hash failed".to_string())));

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            create_mock_id_gen(),
            Arc::new(InMemoryCache::<i64>::new()),
        );
        let result = service
            .reset_password(&ctx, 1, "newpassword".to_string())
            .await;

        assert!(matches!(result, Err(Error::Internal(_))));
    }

    #[tokio::test]
    async fn test_delete_user_success() {
        let mut mock_repo = MockUserRepo::new();
        let mock_hasher = MockHasher::new();
        let ctx = create_test_context();

        mock_repo
            .expect_delete_user()
            .with(mockall::predicate::always(), mockall::predicate::eq(1))
            .times(1)
            .returning(|_, _| Ok(()));

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            create_mock_id_gen(),
            Arc::new(InMemoryCache::<i64>::new()),
        );
        let result = service.delete(&ctx, 1).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_user_no_permission() {
        let mock_repo = MockUserRepo::new();
        let mock_hasher = MockHasher::new();
        let ctx = create_no_permission_context();

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            create_mock_id_gen(),
            Arc::new(InMemoryCache::<i64>::new()),
        );
        let result = service.delete(&ctx, 1).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_delete_user_repo_error() {
        let mut mock_repo = MockUserRepo::new();
        let mock_hasher = MockHasher::new();
        let ctx = create_test_context();

        mock_repo
            .expect_delete_user()
            .times(1)
            .returning(|_, _| Err(Error::Database("DB Error".to_string())));

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            create_mock_id_gen(),
            Arc::new(InMemoryCache::<i64>::new()),
        );
        let result = service.delete(&ctx, 1).await;

        assert!(matches!(result, Err(Error::Database(_))));
    }

    #[tokio::test]
    async fn test_get_user_permission_caching() {
        let mut mock_repo = MockUserRepo::new();
        let mock_hasher = MockHasher::new();
        let ctx = create_test_context();
        let cache = Arc::new(InMemoryCache::<i64>::new());

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

        // Repository should only be called once
        mock_repo
            .expect_get_user_permission()
            .with(mockall::predicate::always(), mockall::predicate::eq(1))
            .times(1)
            .returning(move |_, _| Ok(perms_clone.clone()));

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            create_mock_id_gen(),
            cache,
        );

        // First call - should hit repository
        let result1 = service.get_user_permission(&ctx, 1).await;
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap().len(), 2);

        // Second call - should hit cache (mock expects only 1 call)
        let result2 = service.get_user_permission(&ctx, 1).await;
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_cache_invalidation_on_update() {
        let mut mock_repo = MockUserRepo::new();
        let mock_hasher = MockHasher::new();
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

        mock_repo
            .expect_update_user()
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            create_mock_id_gen(),
            cache.clone(),
        );

        // Update user - should invalidate cache
        let user = create_user_update();
        let result = service.update(&ctx, 1, &user).await;
        assert!(result.is_ok());

        // Cache should be cleared
        let cached: Option<Vec<Permission>> = cache.get(&1i64).await;
        assert!(cached.is_none());
    }

    #[tokio::test]
    async fn test_cache_invalidation_on_delete() {
        let mut mock_repo = MockUserRepo::new();
        let mock_hasher = MockHasher::new();
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

        mock_repo
            .expect_delete_user()
            .times(1)
            .returning(|_, _| Ok(()));

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            create_mock_id_gen(),
            cache.clone(),
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
        let mut mock_repo = MockUserRepo::new();
        let mut mock_hasher = MockHasher::new();
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

        mock_hasher
            .expect_hash_password()
            .times(1)
            .returning(|_| Ok("new_hash".to_string()));

        mock_repo
            .expect_update_password()
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = UserService::new(
            mock_repo,
            Arc::new(mock_hasher),
            create_mock_id_gen(),
            cache.clone(),
        );

        // Reset password - should invalidate cache
        let result = service.reset_password(&ctx, 1, "newpass".to_string()).await;
        assert!(result.is_ok());

        // Cache should be cleared
        let cached: Option<Vec<Permission>> = cache.get(&1i64).await;
        assert!(cached.is_none());
    }
}
