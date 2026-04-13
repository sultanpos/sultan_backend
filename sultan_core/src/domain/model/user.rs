use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::product::SortDirection;
use crate::domain::model::permission::Permission;

/// Sort fields available for user listing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserSortField {
    Id,
    UpdatedAt,
    Name,
}

/// Cursor for keyset (cursor-based) pagination over users.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCursor {
    /// Value of the primary sort field from the last item of the previous page.
    pub field_value: String,
    /// ID of the last item of the previous page (tiebreaker).
    pub id: i64,
}

/// Options for querying a list of users with cursor-based pagination.
#[derive(Debug, Clone)]
pub struct UserQuery {
    pub filter: UserFilter,
    pub sort_field: UserSortField,
    pub sort_direction: SortDirection,
    pub cursor: Option<UserCursor>,
    pub limit: u64,
}

/// A page of users with an optional cursor pointing to the next page.
#[derive(Debug, Clone)]
pub struct UserPage {
    pub items: Vec<User>,
    pub next_cursor: Option<UserCursor>,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub deleted_at: Option<chrono::DateTime<Utc>>,
    pub is_deleted: bool,
    pub username: String,
    pub password: String,
    pub name: String,
    pub email: Option<String>,
    pub photo: Option<String>,
    pub pin: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub permissions: Option<Vec<Permission>>,
}

#[derive(Debug, Clone)]
pub struct UserCreate {
    pub username: String,
    pub password: String,
    pub name: String,
    pub email: Option<String>,
    pub photo: Option<String>,
    pub pin: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UserUpdate {
    pub username: Option<String>,
    pub name: Option<String>,
    pub email: super::Update<String>,
    pub photo: super::Update<String>,
    pub pin: super::Update<String>,
    pub address: super::Update<String>,
    pub phone: super::Update<String>,
}

#[derive(Debug, Clone, Default)]
pub struct UserFilter {
    pub username: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
}

impl UserFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }
}
