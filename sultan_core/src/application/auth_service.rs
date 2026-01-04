use async_trait::async_trait;
use chrono::{Duration, Utc};
use rand::RngCore;

use crate::crypto::{JwtManager, PasswordHash};
use crate::domain::model::token::Token;
use crate::domain::{Context, DomainResult, Error};
use crate::storage::{TokenRepository, UserRepository};

/// Default refresh token expiry in days
const DEFAULT_REFRESH_TOKEN_EXPIRY_DAYS: i64 = 30;

/// Response containing access token and refresh token
#[derive(Debug, Clone)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
}

#[async_trait]
pub trait AuthServiceTrait: Send + Sync {
    async fn login(
        &self,
        ctx: &Context,
        username: &str,
        password: &str,
    ) -> DomainResult<AuthTokens>;
    async fn refresh(&self, ctx: &Context, refresh_token: &str) -> DomainResult<AuthTokens>;
    async fn logout(&self, ctx: &Context, refresh_token: &str) -> DomainResult<()>;
}

/// Auth service handles authentication operations
pub struct AuthService<U, T, P, J, Tx>
where
    U: UserRepository<Tx>,
    T: TokenRepository,
    P: PasswordHash,
    J: JwtManager,
    Tx: Send + Sync,
{
    user_repo: U,
    token_repo: T,
    password_hasher: P,
    jwt_manager: J,
    refresh_token_expiry_days: i64,
    _phantom: std::marker::PhantomData<Tx>,
}

impl<U, T, P, J, Tx> AuthService<U, T, P, J, Tx>
where
    U: UserRepository<Tx>,
    T: TokenRepository,
    P: PasswordHash,
    J: JwtManager,
    Tx: Send + Sync,
{
    /// Creates a new AuthService with default configuration.
    ///
    /// The default refresh token expiry is [`DEFAULT_REFRESH_TOKEN_EXPIRY_DAYS`] (30 days).
    pub fn new(user_repo: U, token_repo: T, password_hasher: P, jwt_manager: J) -> Self {
        Self {
            user_repo,
            token_repo,
            password_hasher,
            jwt_manager,
            refresh_token_expiry_days: DEFAULT_REFRESH_TOKEN_EXPIRY_DAYS,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set custom refresh token expiry.
    ///
    /// The default value is [`DEFAULT_REFRESH_TOKEN_EXPIRY_DAYS`] (30 days).
    pub fn with_refresh_token_expiry_days(mut self, days: i64) -> Self {
        self.refresh_token_expiry_days = days;
        self
    }

    /// Generate access token and refresh token
    async fn generate_tokens(
        &self,
        ctx: &Context,
        user_id: i64,
        username: &str,
    ) -> DomainResult<AuthTokens> {
        // Generate access token (JWT)
        let access_token = self
            .jwt_manager
            .generate_token(user_id, username)
            .map_err(|e| Error::Internal(e.to_string()))?;

        // Generate refresh token (random string)
        let refresh_token = Self::generate_refresh_token();
        let refresh_token_hash = Self::hash_token(&refresh_token);

        // Calculate expiry
        let expired_at = Utc::now() + Duration::days(self.refresh_token_expiry_days);

        // Store hashed refresh token in database
        let token = Token {
            id: 0,
            user_id,
            token: refresh_token_hash,
            expired_at,
        };

        self.token_repo.save(ctx, &token).await?;

        Ok(AuthTokens {
            access_token,
            refresh_token,
        })
    }

    /// Generate a random refresh token
    fn generate_refresh_token() -> String {
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 32];
        rng.fill_bytes(&mut bytes);
        hex::encode(bytes)
    }

    /// Hash a token using MD5
    fn hash_token(token: &str) -> String {
        let digest = md5::compute(token.as_bytes());
        format!("{:x}", digest)
    }
}

#[async_trait]
impl<U, T, P, J, Tx> AuthServiceTrait for AuthService<U, T, P, J, Tx>
where
    U: UserRepository<Tx>,
    T: TokenRepository,
    P: PasswordHash + Send + Sync,
    J: JwtManager + Send + Sync,
    Tx: Send + Sync,
{
    /// Login with username and password
    /// Returns JWT access token and a refresh token
    async fn login(
        &self,
        ctx: &Context,
        username: &str,
        password: &str,
    ) -> DomainResult<AuthTokens> {
        // Get user by username
        let user = self
            .user_repo
            .get_user_by_username(ctx, username)
            .await?
            .ok_or(Error::Unauthorized(
                "Invalid username or password".to_string(),
            ))?;

        // Verify password
        let is_valid = self
            .password_hasher
            .verify_password(password, &user.password)?;
        if !is_valid {
            return Err(Error::Unauthorized(
                "Invalid username or password".to_string(),
            ));
        }

        // Generate tokens
        self.generate_tokens(ctx, user.id, &user.username).await
    }

    /// Login with refresh token
    /// Returns new JWT access token and a new refresh token
    async fn refresh(&self, ctx: &Context, refresh_token: &str) -> DomainResult<AuthTokens> {
        // Hash the refresh token for lookup
        let token_hash = Self::hash_token(refresh_token);

        // Find the token in database
        let stored_token = self
            .token_repo
            .get_by_token(ctx, &token_hash)
            .await?
            .ok_or_else(|| Error::Unauthorized("Invalid refresh token".to_string()))?;

        // Check if token is expired
        if stored_token.expired_at < Utc::now() {
            // Delete expired token
            self.token_repo.delete(ctx, stored_token.id).await?;
            return Err(Error::Unauthorized("Refresh token has expired".to_string()));
        }

        // Get user
        let user = self
            .user_repo
            .get_by_id(ctx, stored_token.user_id)
            .await?
            .ok_or_else(|| Error::Unauthorized("User not found".to_string()))?;

        // Delete old refresh token
        self.token_repo.delete(ctx, stored_token.id).await?;

        // Generate new tokens
        self.generate_tokens(ctx, user.id, &user.username).await
    }

    /// Logout - invalidate refresh token
    async fn logout(&self, ctx: &Context, refresh_token: &str) -> DomainResult<()> {
        let token_hash = Self::hash_token(refresh_token);

        if let Some(stored_token) = self.token_repo.get_by_token(ctx, &token_hash).await? {
            self.token_repo.delete(ctx, stored_token.id).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::password::PasswordHash;
    use crate::domain::model::user::{User, UserCreate, UserUpdate};
    use async_trait::async_trait;

    // Mock User Repository
    struct MockUserRepo {
        user: Option<User>,
    }

    // Use a unit type for mock transaction since mocks don't use real transactions
    #[async_trait]
    impl UserRepository<()> for MockUserRepo {
        async fn create_user(
            &self,
            _ctx: &Context,
            _id: i64,
            _user: &UserCreate,
        ) -> DomainResult<()> {
            Ok(())
        }

        async fn create_user_tx(
            &self,
            _ctx: &Context,
            _id: i64,
            _user: &UserCreate,
            _tx: &mut (),
        ) -> DomainResult<()> {
            Ok(())
        }

        async fn get_user_by_username(
            &self,
            _ctx: &Context,
            _username: &str,
        ) -> DomainResult<Option<User>> {
            Ok(self.user.clone())
        }

        async fn update_user(
            &self,
            _ctx: &Context,
            _id: i64,
            _user: &UserUpdate,
        ) -> DomainResult<()> {
            Ok(())
        }

        async fn update_password(
            &self,
            _ctx: &Context,
            _id: i64,
            _password_hash: &str,
        ) -> DomainResult<()> {
            Ok(())
        }

        async fn delete_user(&self, _ctx: &Context, _user_id: i64) -> DomainResult<()> {
            Ok(())
        }

        async fn delete_user_tx(
            &self,
            _ctx: &Context,
            _user_id: i64,
            _tx: &mut (),
        ) -> DomainResult<()> {
            Ok(())
        }

        async fn get_all(
            &self,
            _ctx: &Context,
            _filter: crate::domain::model::user::UserFilter,
            _pagination: crate::domain::model::pagination::PaginationOptions,
        ) -> DomainResult<Vec<User>> {
            Ok(vec![])
        }

        async fn get_by_id(&self, _ctx: &Context, _user_id: i64) -> DomainResult<Option<User>> {
            Ok(self.user.clone())
        }

        async fn save_user_permission(
            &self,
            _ctx: &Context,
            _user_id: i64,
            _branch_id: Option<i64>,
            _permission: i32,
            _action: i32,
        ) -> DomainResult<()> {
            Ok(())
        }

        async fn delete_user_permission(
            &self,
            _ctx: &Context,
            _user_id: i64,
            _branch_id: Option<i64>,
            _permission: i32,
        ) -> DomainResult<()> {
            Ok(())
        }

        async fn get_user_permission(
            &self,
            _ctx: &Context,
            _user_id: i64,
        ) -> DomainResult<Vec<crate::domain::model::permission::Permission>> {
            Ok(vec![])
        }
    }

    // Mock Token Repository
    struct MockTokenRepo {
        stored_token: std::sync::Mutex<Option<Token>>,
    }

    impl MockTokenRepo {
        fn new() -> Self {
            Self {
                stored_token: std::sync::Mutex::new(None),
            }
        }
    }

    #[async_trait]
    impl TokenRepository for MockTokenRepo {
        async fn save(&self, _ctx: &Context, token: &Token) -> DomainResult<()> {
            *self.stored_token.lock().unwrap() = Some(token.clone());
            Ok(())
        }

        async fn delete(&self, _ctx: &Context, _id: i64) -> DomainResult<()> {
            *self.stored_token.lock().unwrap() = None;
            Ok(())
        }

        async fn get_by_token(&self, _ctx: &Context, token: &str) -> DomainResult<Option<Token>> {
            let stored = self.stored_token.lock().unwrap();
            Ok(stored.as_ref().filter(|t| t.token == token).cloned())
        }
    }

    // Mock Password Hasher
    struct MockPasswordHasher {
        valid_password: String,
    }

    impl PasswordHash for MockPasswordHasher {
        fn hash_password(&self, password: &str) -> DomainResult<String> {
            Ok(format!("hashed_{}", password))
        }

        fn verify_password(&self, password: &str, _hash: &str) -> DomainResult<bool> {
            Ok(password == self.valid_password)
        }
    }

    // Mock JWT Manager
    struct MockJwtManager;

    impl JwtManager for MockJwtManager {
        fn generate_token(&self, user_id: i64, username: &str) -> crate::crypto::JwtResult<String> {
            Ok(format!("jwt_{}_{}", user_id, username))
        }

        fn validate_token(&self, _token: &str) -> crate::crypto::JwtResult<crate::crypto::Claims> {
            Ok(crate::crypto::Claims {
                sub: "1".to_string(),
                exp: 0,
                iat: 0,
                user_id: 1,
                username: "test".to_string(),
            })
        }
    }

    fn create_test_user(password_hash: &str) -> User {
        User {
            id: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            is_deleted: false,
            username: "testuser".to_string(),
            password: password_hash.to_string(),
            name: "Test User".to_string(),
            email: Some("test@example.com".to_string()),
            photo: None,
            pin: None,
            address: None,
            phone: None,
            permissions: None,
        }
    }

    #[tokio::test]
    async fn test_login_success() {
        let user = create_test_user("hashed_password123");
        let user_repo = MockUserRepo { user: Some(user) };
        let token_repo = MockTokenRepo::new();
        let password_hasher = MockPasswordHasher {
            valid_password: "password123".to_string(),
        };
        let jwt_manager = MockJwtManager;

        let service = AuthService::new(user_repo, token_repo, password_hasher, jwt_manager);

        let ctx = Context::new();
        let result = service.login(&ctx, "testuser", "password123").await;

        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert_eq!(tokens.access_token, "jwt_1_testuser");
        assert!(!tokens.refresh_token.is_empty());
    }

    #[tokio::test]
    async fn test_login_invalid_username() {
        let user_repo = MockUserRepo { user: None };
        let token_repo = MockTokenRepo::new();
        let password_hasher = MockPasswordHasher {
            valid_password: "password123".to_string(),
        };
        let jwt_manager = MockJwtManager;

        let service = AuthService::new(user_repo, token_repo, password_hasher, jwt_manager);

        let ctx = Context::new();
        let result = service.login(&ctx, "nonexistent", "password123").await;

        assert!(matches!(result, Err(Error::Unauthorized(_))));
    }

    #[tokio::test]
    async fn test_login_invalid_password() {
        let user = create_test_user("hashed_password123");
        let user_repo = MockUserRepo { user: Some(user) };
        let token_repo = MockTokenRepo::new();
        let password_hasher = MockPasswordHasher {
            valid_password: "password123".to_string(),
        };
        let jwt_manager = MockJwtManager;

        let service = AuthService::new(user_repo, token_repo, password_hasher, jwt_manager);

        let ctx = Context::new();
        let result = service.login(&ctx, "testuser", "wrong_password").await;

        assert!(matches!(result, Err(Error::Unauthorized(_))));
    }

    #[tokio::test]
    async fn test_refresh_token() {
        let user = create_test_user("hashed_password");
        let user_repo = MockUserRepo {
            user: Some(user.clone()),
        };
        let token_repo = MockTokenRepo::new();
        let password_hasher = MockPasswordHasher {
            valid_password: "password".to_string(),
        };
        let jwt_manager = MockJwtManager;

        let service = AuthService::new(user_repo, token_repo, password_hasher, jwt_manager);

        let ctx = Context::new();

        // First login
        let tokens = service.login(&ctx, "testuser", "password").await.unwrap();

        // Use refresh token
        let new_tokens = service.refresh(&ctx, &tokens.refresh_token).await.unwrap();

        assert!(!new_tokens.access_token.is_empty());
        assert!(!new_tokens.refresh_token.is_empty());
        // New refresh token should be different
        assert_ne!(tokens.refresh_token, new_tokens.refresh_token);
    }

    #[tokio::test]
    async fn test_refresh_invalid_token() {
        let user_repo = MockUserRepo { user: None };
        let token_repo = MockTokenRepo::new();
        let password_hasher = MockPasswordHasher {
            valid_password: "password".to_string(),
        };
        let jwt_manager = MockJwtManager;

        let service = AuthService::new(user_repo, token_repo, password_hasher, jwt_manager);

        let ctx = Context::new();
        let result = service.refresh(&ctx, "invalid_refresh_token").await;

        assert!(matches!(result, Err(Error::Unauthorized(_))));
    }

    #[tokio::test]
    async fn test_logout() {
        let user = create_test_user("hashed_password");
        let user_repo = MockUserRepo { user: Some(user) };
        let token_repo = MockTokenRepo::new();
        let password_hasher = MockPasswordHasher {
            valid_password: "password".to_string(),
        };
        let jwt_manager = MockJwtManager;

        let service = AuthService::new(user_repo, token_repo, password_hasher, jwt_manager);

        let ctx = Context::new();

        // Login first
        let tokens = service.login(&ctx, "testuser", "password").await.unwrap();

        // Logout
        let result = service.logout(&ctx, &tokens.refresh_token).await;
        assert!(result.is_ok());

        // Try to use refresh token after logout - should fail
        let refresh_result = service.refresh(&ctx, &tokens.refresh_token).await;
        assert!(matches!(refresh_result, Err(Error::Unauthorized(_))));
    }

    #[test]
    fn test_hash_token() {
        let token = "test_token";
        let hash1 = AuthService::<
            MockUserRepo,
            MockTokenRepo,
            MockPasswordHasher,
            MockJwtManager,
            (),
        >::hash_token(token);
        let hash2 = AuthService::<
            MockUserRepo,
            MockTokenRepo,
            MockPasswordHasher,
            MockJwtManager,
            (),
        >::hash_token(token);

        // Same token should produce same hash
        assert_eq!(hash1, hash2);

        // Hash should be different from input
        assert_ne!(hash1, token);

        // Hash should be hex encoded MD5 (32 characters)
        assert_eq!(hash1.len(), 32);
    }
}
