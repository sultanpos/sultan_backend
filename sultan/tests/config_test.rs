use serial_test::serial;
use std::env;
use sultan::config::AppConfig;

/// Helper to set environment variables for tests
struct EnvGuard {
    keys: Vec<String>,
}

impl EnvGuard {
    fn new() -> Self {
        Self { keys: Vec::new() }
    }

    fn set(&mut self, key: &str, value: &str) {
        unsafe {
            env::set_var(key, value);
        }
        self.keys.push(key.to_string());
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for key in &self.keys {
            unsafe {
                env::remove_var(key);
            }
        }
    }
}

#[test]
#[serial]
fn test_from_env_with_defaults() {
    let mut guard = EnvGuard::new();
    guard.set("JWT_SECRET", "test_secret_key_123");
    guard.set("DATABASE_URL", "sqlite:test.db");

    let config = AppConfig::from_env();

    assert_eq!(config.jwt_secret, "test_secret_key_123");
    assert_eq!(config.database_url, "sqlite:test.db");
    assert_eq!(config.access_token_ttl.as_seconds_f64() as i64, 900);
    assert_eq!(
        config.refresh_token_ttl.as_seconds_f64() as i64,
        30 * 24 * 60 * 60
    );
    assert_eq!(config.database_max_connections, 5);
    assert!(!config.write_log_to_file);
}

#[test]
#[serial]
fn test_from_env_with_custom_values() {
    let mut guard = EnvGuard::new();
    guard.set("JWT_SECRET", "custom_secret");
    guard.set("DATABASE_URL", "sqlite:custom.db");
    guard.set("REFRESH_TOKEN_TTL_DAYS", "60");
    guard.set("ACCESS_TOKEN_TTL_SECS", "1800");
    guard.set("DATABASE_MAX_CONNECTIONS", "10");
    guard.set("WRITE_LOG_TO_FILE", "1");

    let config = AppConfig::from_env();

    assert_eq!(config.jwt_secret, "custom_secret");
    assert_eq!(config.database_url, "sqlite:custom.db");
    assert_eq!(config.access_token_ttl.as_seconds_f64() as i64, 1800);
    assert_eq!(
        config.refresh_token_ttl.as_seconds_f64() as i64,
        60 * 24 * 60 * 60
    );
    assert_eq!(config.database_max_connections, 10);
    assert!(config.write_log_to_file);
}

#[test]
#[serial]
fn test_from_env_write_log_to_file_true() {
    let mut guard = EnvGuard::new();
    guard.set("JWT_SECRET", "test_secret");
    guard.set("DATABASE_URL", "sqlite:test.db");
    guard.set("WRITE_LOG_TO_FILE", "true");

    let config = AppConfig::from_env();
    assert!(config.write_log_to_file);
}

#[test]
#[serial]
fn test_from_env_write_log_to_file_yes() {
    let mut guard = EnvGuard::new();
    guard.set("JWT_SECRET", "test_secret");
    guard.set("DATABASE_URL", "sqlite:test.db");
    guard.set("WRITE_LOG_TO_FILE", "yes");

    let config = AppConfig::from_env();
    assert!(config.write_log_to_file);
}

#[test]
#[serial]
fn test_from_env_write_log_to_file_uppercase() {
    let mut guard = EnvGuard::new();
    guard.set("JWT_SECRET", "test_secret");
    guard.set("DATABASE_URL", "sqlite:test.db");
    guard.set("WRITE_LOG_TO_FILE", "TRUE");

    let config = AppConfig::from_env();
    assert!(config.write_log_to_file);
}

#[test]
#[serial]
fn test_from_env_write_log_to_file_zero() {
    let mut guard = EnvGuard::new();
    guard.set("JWT_SECRET", "test_secret");
    guard.set("DATABASE_URL", "sqlite:test.db");
    guard.set("WRITE_LOG_TO_FILE", "0");

    let config = AppConfig::from_env();
    assert!(!config.write_log_to_file);
}

#[test]
#[serial]
fn test_from_env_write_log_to_file_false() {
    let mut guard = EnvGuard::new();
    guard.set("JWT_SECRET", "test_secret");
    guard.set("DATABASE_URL", "sqlite:test.db");
    guard.set("WRITE_LOG_TO_FILE", "false");

    let config = AppConfig::from_env();
    assert!(!config.write_log_to_file);
}

#[test]
#[serial]
#[should_panic(expected = "JWT_SECRET must be set")]
fn test_from_env_missing_jwt_secret() {
    let mut guard = EnvGuard::new();
    guard.set("DATABASE_URL", "sqlite:test.db");

    // Remove JWT_SECRET if it exists
    unsafe {
        env::remove_var("JWT_SECRET");
    }

    // This should panic
    AppConfig::from_env();
}

#[test]
#[serial]
#[should_panic(expected = "DATABASE_URL must be set")]
fn test_from_env_missing_database_url() {
    let mut guard = EnvGuard::new();
    guard.set("JWT_SECRET", "test_secret");

    // Remove DATABASE_URL if it exists
    unsafe {
        env::remove_var("DATABASE_URL");
    }

    // This should panic
    AppConfig::from_env();
}

#[test]
#[serial]
#[should_panic(expected = "REFRESH_TOKEN_TTL_DAYS must be a valid number")]
fn test_from_env_invalid_refresh_ttl() {
    let mut guard = EnvGuard::new();
    guard.set("JWT_SECRET", "test_secret");
    guard.set("DATABASE_URL", "sqlite:test.db");
    guard.set("REFRESH_TOKEN_TTL_DAYS", "not_a_number");

    AppConfig::from_env();
}

#[test]
#[serial]
#[should_panic(expected = "ACCESS_TOKEN_TTL_SECS must be a valid number")]
fn test_from_env_invalid_access_ttl() {
    let mut guard = EnvGuard::new();
    guard.set("JWT_SECRET", "test_secret");
    guard.set("DATABASE_URL", "sqlite:test.db");
    guard.set("ACCESS_TOKEN_TTL_SECS", "invalid");

    AppConfig::from_env();
}

#[test]
#[serial]
#[should_panic(expected = "DATABASE_MAX_CONNECTIONS must be a valid number")]
fn test_from_env_invalid_max_connections() {
    let mut guard = EnvGuard::new();
    guard.set("JWT_SECRET", "test_secret");
    guard.set("DATABASE_URL", "sqlite:test.db");
    guard.set("DATABASE_MAX_CONNECTIONS", "abc");

    AppConfig::from_env();
}
