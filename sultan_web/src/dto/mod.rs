pub mod category;
pub mod customer;
pub mod login;
pub mod supplier;

pub use category::{CategoryCreateRequest, CategoryCreateResponse};
pub use customer::{CustomerCreateRequest, CustomerCreateResponse};
pub use login::{LoginRequest, LoginResponse, LogoutRequest, RefreshTokenRequest};
pub use supplier::{SupplierCreateRequest, SupplierCreateResponse};

use serde::Serialize;
use utoipa::ToSchema;

/// Standard error response
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Error message describing what went wrong
    #[schema(example = "Username cannot be empty")]
    pub error: String,
}

pub fn default_page() -> u32 {
    1
}

pub fn default_page_size() -> u32 {
    20
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListResponse<T: Serialize> {
    pub data: Vec<T>,
}
