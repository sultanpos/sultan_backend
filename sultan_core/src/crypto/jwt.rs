use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::fmt;

/// JWT error types
#[derive(Debug)]
pub enum JwtError {
    /// Token has expired
    Expired,
    /// Token is invalid
    Invalid(String),
    /// Failed to encode token
    EncodingFailed(String),
}

impl fmt::Display for JwtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JwtError::Expired => write!(f, "Token has expired"),
            JwtError::Invalid(msg) => write!(f, "Invalid token: {}", msg),
            JwtError::EncodingFailed(msg) => write!(f, "Failed to encode token: {}", msg),
        }
    }
}

impl std::error::Error for JwtError {}

/// Result type for JWT operations
pub type JwtResult<T> = Result<T, JwtError>;

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Expiration time (as Unix timestamp)
    pub exp: i64,
    /// Issued at (as Unix timestamp)
    pub iat: i64,
    /// User ID
    pub user_id: i64,
    /// Username
    pub username: String,
}

/// Configuration for JWT
#[derive(Clone)]
pub struct JwtConfig {
    /// Secret key for signing tokens
    secret: String,
    /// Token expiration in minutes
    expiration_minutes: i64,
}

impl JwtConfig {
    pub fn new(secret: impl Into<String>, expiration_minutes: i64) -> Self {
        Self {
            secret: secret.into(),
            expiration_minutes,
        }
    }

    pub fn expiration_minutes(&self) -> i64 {
        self.expiration_minutes
    }
}

/// JWT token manager
pub trait JwtManager: Send + Sync {
    /// Generate a JWT token for a user
    fn generate_token(&self, user_id: i64, username: &str) -> JwtResult<String>;

    /// Validate and decode a JWT token
    fn validate_token(&self, token: &str) -> JwtResult<Claims>;
}

/// Default JWT manager implementation
#[derive(Clone)]
pub struct DefaultJwtManager {
    config: JwtConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl DefaultJwtManager {
    pub fn new(config: JwtConfig) -> Self {
        let encoding_key = EncodingKey::from_secret(config.secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.secret.as_bytes());

        Self {
            config,
            encoding_key,
            decoding_key,
        }
    }
}

impl JwtManager for DefaultJwtManager {
    fn generate_token(&self, user_id: i64, username: &str) -> JwtResult<String> {
        let now = Utc::now();
        let exp = now + Duration::minutes(self.config.expiration_minutes);

        let claims = Claims {
            sub: user_id.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            user_id,
            username: username.to_string(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| JwtError::EncodingFailed(e.to_string()))
    }

    fn validate_token(&self, token: &str) -> JwtResult<Claims> {
        let validation = Validation::default();

        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtError::Expired,
                _ => JwtError::Invalid(e.to_string()),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_jwt_manager() -> DefaultJwtManager {
        let config = JwtConfig::new("test_secret_key_for_testing_only", 60);
        DefaultJwtManager::new(config)
    }

    #[test]
    fn test_generate_and_validate_token() {
        let manager = create_jwt_manager();

        let token = manager
            .generate_token(123, "testuser")
            .expect("Failed to generate token");

        let claims = manager
            .validate_token(&token)
            .expect("Failed to validate token");

        assert_eq!(claims.user_id, 123);
        assert_eq!(claims.username, "testuser");
        assert_eq!(claims.sub, "123");
    }

    #[test]
    fn test_invalid_token() {
        let manager = create_jwt_manager();

        let result = manager.validate_token("invalid_token");
        assert!(matches!(result, Err(JwtError::Invalid(_))));
    }

    #[test]
    fn test_expired_token() {
        use jsonwebtoken::{EncodingKey, Header, encode};

        // Create expired claims directly
        let config = JwtConfig::new("test_secret", 60);
        let manager = DefaultJwtManager::new(config);

        // Create a token with an expiration time in the past
        let claims = Claims {
            sub: "123".to_string(),
            exp: 0, // Unix timestamp 0 = 1970, definitely expired
            iat: 0,
            user_id: 123,
            username: "testuser".to_string(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret("test_secret".as_bytes()),
        )
        .expect("Failed to encode token");

        let result = manager.validate_token(&token);
        assert!(matches!(result, Err(JwtError::Expired)));
    }
}
