use crate::snowflake::SnowflakeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Cancelled: {0}")]
    Cancelled(String),
}

impl From<SnowflakeError> for Error {
    fn from(e: SnowflakeError) -> Self {
        Error::Internal(e.to_string())
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Error::Database(err.to_string())
    }
}

pub type DomainResult<T> = Result<T, Error>;
