pub mod category;
pub mod customer;
pub mod login;

pub use category::{CategoryCreateRequest, CategoryCreateResponse};
pub use customer::{CustomerCreateRequest, CustomerCreateResponse};
pub use login::{LoginRequest, LoginResponse, LogoutRequest, RefreshTokenRequest};

use serde::Serialize;
use utoipa::ToSchema;

/// Standard error response
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Error message describing what went wrong
    #[schema(example = "Username cannot be empty")]
    pub error: String,
}
