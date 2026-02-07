use super::{i64_to_string, option_string_to_i64};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sultan_core::domain::model::Update;
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use super::{default_page, default_page_size};

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

#[derive(Debug, Deserialize, IntoParams)]
pub struct UserQueryParams {
    pub username: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    pub order_by: Option<String>,
    pub order_direction: Option<String>,
}

impl UserQueryParams {
    /// Convert to CustomerFilter
    pub fn to_filter(&self) -> sultan_core::domain::model::user::UserFilter {
        sultan_core::domain::model::user::UserFilter {
            username: self.username.clone(),
            name: self.name.clone(),
            email: self.email.clone(),
        }
    }

    /// Convert to PaginationOptions
    pub fn to_pagination(&self) -> sultan_core::domain::model::pagination::PaginationOptions {
        use sultan_core::domain::model::pagination::{PaginationOptions, PaginationOrder};

        let page_size = self.page_size.min(100); // Cap at 100
        let order = match (self.order_by.as_ref(), self.order_direction.as_ref()) {
            (Some(field), direction) => Some(PaginationOrder {
                field: field.clone(),
                direction: direction.cloned().unwrap_or_else(|| "asc".to_string()),
            }),
            _ => None,
        };

        PaginationOptions::new(self.page, page_size, order)
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserListResponse {
    pub data: Vec<UserResponse>,
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
