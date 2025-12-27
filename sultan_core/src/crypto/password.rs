use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

use crate::domain::{DomainResult, Error};

pub trait PasswordHash {
    fn hash_password(&self, password: &str) -> DomainResult<String>;
    fn verify_password(&self, password: &str, hash: &str) -> DomainResult<bool>;
}

#[derive(Default)]
pub struct Argon2PasswordHasher {
    hasher: Argon2<'static>,
}

impl PasswordHash for Argon2PasswordHasher {
    fn hash_password(&self, password: &str) -> DomainResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = self
            .hasher
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| Error::Internal("Password hashing failed".into()))?
            .to_string();

        Ok(hash)
    }

    fn verify_password(&self, password: &str, hash: &str) -> DomainResult<bool> {
        let parsed_hash = argon2::PasswordHash::new(hash)
            .map_err(|_| Error::Internal("Invalid password hash".into()))?;

        Ok(self
            .hasher
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password_returns_hash() {
        let hasher = Argon2PasswordHasher::default();
        let password = "my_secure_password";

        let result = hasher.hash_password(password);

        assert!(result.is_ok());
        let hash = result.unwrap();
        assert!(!hash.is_empty());
        // Argon2 hashes start with '$argon2'
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn test_hash_password_different_calls_produce_different_hashes() {
        let hasher = Argon2PasswordHasher::default();
        let password = "my_secure_password";

        let hash1 = hasher.hash_password(password).unwrap();
        let hash2 = hasher.hash_password(password).unwrap();

        // Different salts should produce different hashes even for the same password
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_password_with_empty_string() {
        let hasher = Argon2PasswordHasher::default();

        let result = hasher.hash_password("");

        assert!(result.is_ok());
        let hash = result.unwrap();
        assert!(!hash.is_empty());
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn test_hash_password_with_long_password() {
        let hasher = Argon2PasswordHasher::default();
        let long_password = "a".repeat(1000);

        let result = hasher.hash_password(&long_password);

        assert!(result.is_ok());
        let hash = result.unwrap();
        assert!(!hash.is_empty());
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn test_hash_password_with_special_characters() {
        let hasher = Argon2PasswordHasher::default();
        let password_with_special_chars = "P@ssw0rd!#$%^&*()_+-=[]{}|;:',.<>?/`~";

        let result = hasher.hash_password(password_with_special_chars);

        assert!(result.is_ok());
        let hash = result.unwrap();
        assert!(!hash.is_empty());
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn test_hash_password_with_unicode_characters() {
        let hasher = Argon2PasswordHasher::default();
        let password_with_unicode = "パスワード密码🔐";

        let result = hasher.hash_password(password_with_unicode);

        assert!(result.is_ok());
        let hash = result.unwrap();
        assert!(!hash.is_empty());
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn test_verify_password_success() {
        let hasher = Argon2PasswordHasher::default();
        let password = "my_secure_password";
        let hash = hasher.hash_password(password).unwrap();

        let result = hasher.verify_password(password, &hash);

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_password_failure() {
        let hasher = Argon2PasswordHasher::default();
        let password = "my_secure_password";
        let hash = hasher.hash_password(password).unwrap();

        let result = hasher.verify_password("wrong_password", &hash);

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_verify_password_with_invalid_hash_format() {
        let hasher = Argon2PasswordHasher::default();
        let result = hasher.verify_password("password", "invalid_hash_format");

        assert!(result.is_err());
    }
}
