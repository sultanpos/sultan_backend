use std::env;
use time::Duration;

#[derive(Clone)]
pub struct AppConfig {
    pub jwt_secret: String,
    pub access_token_ttl: Duration,
    pub refresh_token_ttl: Duration,
    pub database_url: String,
    pub write_log_to_file: bool,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let refresh_token_ttl_days: i64 = env::var("REFRESH_TOKEN_TTL_DAYS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .expect("REFRESH_TOKEN_TTL_DAYS must be a valid number");

        let access_token_ttl_secs: i64 = env::var("ACCESS_TOKEN_TTL_SECS")
            .unwrap_or_else(|_| "900".to_string())
            .parse()
            .expect("ACCESS_TOKEN_TTL_SECS must be a valid number");

        let write_log_to_file = env::var("WRITE_LOG_TO_FILE")
            .unwrap_or_else(|_| "0".to_string())
            .to_lowercase();
        let write_log_to_file = matches!(write_log_to_file.as_str(), "1" | "true" | "yes");

        Self {
            jwt_secret,
            access_token_ttl: Duration::seconds(access_token_ttl_secs),
            refresh_token_ttl: Duration::days(refresh_token_ttl_days),
            database_url,
            write_log_to_file,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_clone() {
        let config = AppConfig {
            jwt_secret: "secret123".to_string(),
            access_token_ttl: Duration::seconds(900),
            refresh_token_ttl: Duration::days(30),
            database_url: "sqlite:test.db".to_string(),
            write_log_to_file: false,
        };

        let cloned = config.clone();
        assert_eq!(config.jwt_secret, cloned.jwt_secret);
        assert_eq!(config.database_url, cloned.database_url);
        assert_eq!(config.write_log_to_file, cloned.write_log_to_file);
        assert_eq!(config.access_token_ttl, cloned.access_token_ttl);
        assert_eq!(config.refresh_token_ttl, cloned.refresh_token_ttl);
    }

    #[test]
    fn test_duration_calculations() {
        let access_ttl = Duration::seconds(900);
        assert_eq!(access_ttl.as_seconds_f64() as i64, 900);

        let refresh_ttl = Duration::days(30);
        assert_eq!(refresh_ttl.as_seconds_f64() as i64, 30 * 24 * 60 * 60);
    }

    #[test]
    fn test_string_matching_for_log_to_file() {
        // Test the matching logic used in from_env
        let test_values = vec!["1", "true", "yes", "TRUE", "True", "YES"];
        for value in test_values {
            let is_enabled = matches!(value.to_lowercase().as_str(), "1" | "true" | "yes");
            assert!(
                is_enabled,
                "Value '{}' should be recognized as enabled",
                value
            );
        }

        let disabled_values = vec!["0", "false", "no", "anything_else", ""];
        for value in disabled_values {
            let is_enabled = matches!(value.to_lowercase().as_str(), "1" | "true" | "yes");
            assert!(
                !is_enabled,
                "Value '{}' should be recognized as disabled",
                value
            );
        }
    }
}
