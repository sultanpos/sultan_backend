use super::{i64_to_string, option_string_to_i64};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sultan_core::domain::model::Update;
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct PermissionCreateRequest {
    #[schema(example = "1234567890", value_type = Option<String>)]
    #[serde(default, deserialize_with = "option_string_to_i64")]
    pub branch_id: Option<i64>,
    #[serde(default)]
    pub resource: i32,
    #[serde(default)]
    pub action: i32,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UserCreateRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Username must be between 1 and 100 characters"
    ))]
    #[schema(example = "sultan")]
    #[serde(default)]
    pub username: String,
    #[validate(length(
        min = 1,
        max = 100,
        message = "Password must be between 1 and 100 characters"
    ))]
    #[schema(example = "sultan")]
    #[serde(default)]
    pub password: String,
    #[validate(length(
        min = 1,
        max = 100,
        message = "Name must be between 1 and 100 characters"
    ))]
    #[schema(example = "Sultan")]
    #[serde(default)]
    pub name: String,
    pub email: Option<String>,
    pub photo: Option<String>,
    pub pin: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    #[serde(default)]
    pub permissions: Vec<PermissionCreateRequest>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserCreateResponse {
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub id: i64,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UserUpdateRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Username must be between 1 and 100 characters"
    ))]
    #[schema(example = "sultan")]
    pub username: Option<String>,
    #[validate(length(
        min = 1,
        max = 100,
        message = "Name must be between 1 and 100 characters"
    ))]
    #[schema(example = "Sultan")]
    pub name: Option<String>,
    #[schema(value_type = Option<String>)]
    pub email: Update<String>,
    #[schema(value_type = Option<String>)]
    pub photo: Update<String>,
    #[schema(value_type = Option<String>)]
    pub pin: Update<String>,
    #[schema(value_type = Option<String>)]
    pub address: Update<String>,
    #[schema(value_type = Option<String>)]
    pub phone: Update<String>,
    pub permissions: Option<Vec<PermissionCreateRequest>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub username: String,
    pub password: String,
    pub name: String,
    pub email: Option<String>,
    pub photo: Option<String>,
    pub pin: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
}

impl From<sultan_core::domain::model::user::User> for UserResponse {
    fn from(user: sultan_core::domain::model::user::User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            password: user.password,
            name: user.name,
            email: user.email,
            photo: user.photo,
            pin: user.pin,
            address: user.address,
            phone: user.phone,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

fn default_user_sort_field() -> String {
    "id".to_string()
}

fn default_sort_direction() -> String {
    "asc".to_string()
}

fn default_limit() -> u32 {
    20
}

/// Query parameters for listing users with cursor-based pagination.
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct UserQueryParams {
    /// Username filter (exact match)
    pub username: Option<String>,
    /// Name filter (partial match)
    pub name: Option<String>,
    /// Email filter (exact match)
    pub email: Option<String>,

    /// Sort field: "id", "updated_at", or "name" (default: "id")
    #[serde(default = "default_user_sort_field")]
    #[schema(example = "id")]
    pub sort_field: String,

    /// Sort direction: "asc" or "desc" (default: "asc")
    #[serde(default = "default_sort_direction")]
    #[schema(example = "asc")]
    pub sort_direction: String,

    /// Opaque cursor from the previous page's `next_cursor` (omit for the first page)
    #[schema(example = "eyJmaWVsZF92YWx1ZSI6IjEyMzQ1NiIsImlkIjoxMjM0NX0")]
    pub cursor: Option<String>,

    /// Maximum number of items per page (default: 20, max: 100)
    #[serde(default = "default_limit")]
    #[schema(example = 20)]
    pub limit: u32,
}

impl UserQueryParams {
    /// Convert to UserQuery for cursor-based pagination.
    pub fn to_query(
        &self,
    ) -> Result<sultan_core::domain::model::user::UserQuery, sultan_core::domain::Error> {
        use sultan_core::domain::model::user::{UserCursor, UserFilter, UserQuery, UserSortField};

        let sort_field = match self.sort_field.as_str() {
            "id" => UserSortField::Id,
            "updated_at" => UserSortField::UpdatedAt,
            "name" => UserSortField::Name,
            other => {
                return Err(sultan_core::domain::Error::ValidationError(format!(
                    "Invalid sort_field '{}'. Must be one of: id, updated_at, name",
                    other
                )));
            }
        };

        let sort_direction = super::parse_sort_direction(&self.sort_direction)?;
        let cursor = self
            .cursor
            .as_deref()
            .map(super::decode_cursor::<UserCursor>)
            .transpose()?;

        Ok(UserQuery {
            filter: UserFilter {
                username: self.username.clone(),
                name: self.name.clone(),
                email: self.email.clone(),
            },
            sort_field,
            sort_direction,
            cursor,
            limit: self.limit.clamp(1, 100) as u64,
        })
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserListResponse {
    pub items: Vec<UserResponse>,
    /// Opaque cursor to fetch the next page. `null` when there are no more pages.
    #[schema(example = "eyJmaWVsZF92YWx1ZSI6IjEyMzQ1NiIsImlkIjoxMjM0NX0")]
    pub next_cursor: Option<String>,
}

impl UserListResponse {
    pub fn from_page(page: sultan_core::domain::model::user::UserPage) -> Self {
        use base64::Engine;

        let next_cursor = page.next_cursor.map(|c| {
            let json = serde_json::to_vec(&c).expect("cursor is always serializable");
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json)
        });

        Self {
            items: page.items.into_iter().map(UserResponse::from).collect(),
            next_cursor,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserPermissionResponse {
    pub branch_id: Option<i64>,
    pub resource: i32,
    pub action: i32,
}

impl From<sultan_core::domain::model::permission::Permission> for UserPermissionResponse {
    fn from(permission: sultan_core::domain::model::permission::Permission) -> Self {
        Self {
            branch_id: permission.branch_id,
            resource: permission.resource,
            action: permission.action,
        }
    }
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ChangeMyPasswordRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Old password must be between 1 and 100 characters"
    ))]
    #[schema(example = "password")]
    pub old_password: String,
    #[validate(length(
        min = 1,
        max = 100,
        message = "New password must be between 1 and 100 characters"
    ))]
    #[schema(example = "password")]
    pub new_password: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ResetPasswordRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "New password must be between 1 and 100 characters"
    ))]
    #[schema(example = "password")]
    pub new_password: String,
}
