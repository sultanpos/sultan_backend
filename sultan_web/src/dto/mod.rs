#![allow(dead_code)]
pub mod category;
pub mod customer;
pub mod login;
pub mod supplier;
pub mod user;

pub use category::{CategoryCreateRequest, CategoryCreateResponse};
pub use customer::{CustomerCreateRequest, CustomerCreateResponse};
pub use login::{LoginRequest, LoginResponse, LogoutRequest, RefreshTokenRequest};
use sultan_core::domain::model::Update;
pub use supplier::{SupplierCreateRequest, SupplierCreateResponse};
pub use user::{PermissionCreateRequest, UserCreateRequest, UserResponse, UserUpdateRequest};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use utoipa::ToSchema;

/// Standard error response
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Error message describing what went wrong
    #[schema(example = "Error message")]
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

fn i64_to_string<S>(v: &i64, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&v.to_string())
}

fn string_to_i64<'de, D>(d: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    s.parse().map_err(serde::de::Error::custom)
}

pub fn option_string_to_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        Some(s) => s.parse::<i64>().map(Some).map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

pub fn update_string_to_i64<'de, D>(deserializer: D) -> Result<Update<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        Some(s) => s
            .parse::<i64>()
            .map(Update::Set)
            .map_err(serde::de::Error::custom),
        None => Ok(Update::Clear),
    }
}
