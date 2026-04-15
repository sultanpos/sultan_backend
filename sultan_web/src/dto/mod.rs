#![allow(dead_code)]
pub mod branch;
pub mod cashier_session;
pub mod category;
pub mod customer;
pub mod login;
pub mod machine;
pub mod payment_channel;
pub mod product;
pub mod purchase_order;
pub mod supplier;
pub mod user;

pub use branch::{BranchCreateRequest, BranchCreateResponse};
pub use category::{CategoryCreateRequest, CategoryCreateResponse};
pub use customer::{CustomerCreateRequest, CustomerCreateResponse};
pub use login::{LoginRequest, LoginResponse, LogoutRequest, RefreshTokenRequest};
pub use product::{
    ProductCreateRequest, ProductCreateResponse, ProductListResponse, ProductResponse,
};
use sultan_core::domain::model::Update;
pub use supplier::{SupplierCreateRequest, SupplierCreateResponse};
pub use user::{PermissionCreateRequest, UserCreateRequest, UserResponse, UserUpdateRequest};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use utoipa::ToSchema;

/// Parses a sort direction string into the domain `SortDirection` type.
/// Returns a validation error for any value other than `"asc"` or `"desc"`.
pub(crate) fn parse_sort_direction(
    s: &str,
) -> Result<sultan_core::domain::model::product::SortDirection, sultan_core::domain::Error> {
    use sultan_core::domain::model::product::SortDirection;
    match s {
        "asc" => Ok(SortDirection::Asc),
        "desc" => Ok(SortDirection::Desc),
        other => Err(sultan_core::domain::Error::ValidationError(format!(
            "Invalid sort_direction '{}'. Must be 'asc' or 'desc'",
            other
        ))),
    }
}

/// Decodes a base64url-encoded cursor string into the given cursor type.
/// Returns a validation error if decoding or deserialization fails.
pub(crate) fn decode_cursor<C: serde::de::DeserializeOwned>(
    encoded: &str,
) -> Result<C, sultan_core::domain::Error> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| {
            sultan_core::domain::Error::ValidationError("Invalid cursor encoding".to_string())
        })?;
    serde_json::from_slice::<C>(&bytes).map_err(|_| {
        sultan_core::domain::Error::ValidationError("Invalid cursor format".to_string())
    })
}

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

fn option_i64_to_string<S>(v: &Option<i64>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match v {
        Some(id) => s.serialize_some(&id.to_string()),
        None => s.serialize_none(),
    }
}

pub(crate) fn string_to_i64<'de, D>(d: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    s.parse().map_err(serde::de::Error::custom)
}

pub fn vec_string_to_i64<'de, D>(deserializer: D) -> Result<Vec<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let vec = Vec::<String>::deserialize(deserializer)?;
    vec.iter()
        .map(|s| s.parse::<i64>().map_err(serde::de::Error::custom))
        .collect()
}

pub fn option_vec_string_to_i64<'de, D>(deserializer: D) -> Result<Option<Vec<i64>>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<Vec<String>>::deserialize(deserializer)?;
    match opt {
        Some(vec) => vec
            .iter()
            .map(|s| s.parse::<i64>().map_err(serde::de::Error::custom))
            .collect::<Result<Vec<i64>, _>>()
            .map(Some),
        None => Ok(None),
    }
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
