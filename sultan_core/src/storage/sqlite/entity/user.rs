use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// SeaORM entity for User table.
///
/// This entity represents the `users` table in the database.
/// It follows the standard Sultan pattern with:
/// - Soft delete support (is_deleted, deleted_at)
/// - Automatic timestamps (created_at, updated_at)
/// - Snowflake ID as primary key
/// - User-specific fields (username, password, contact info)
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub is_deleted: bool,
    pub username: String,
    pub password: String,
    pub name: String,
    pub email: Option<String>,
    pub photo: Option<String>,
    pub pin: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Converts the SeaORM model to the domain model.
    pub fn to_domain(&self) -> crate::domain::model::user::User {
        crate::domain::model::user::User {
            id: self.id,
            created_at: super::super::parse_sqlite_date(&self.created_at),
            updated_at: super::super::parse_sqlite_date(&self.updated_at),
            deleted_at: self
                .deleted_at
                .as_ref()
                .map(|d| super::super::parse_sqlite_date(d)),
            is_deleted: self.is_deleted,
            username: self.username.clone(),
            password: self.password.clone(),
            name: self.name.clone(),
            email: self.email.clone(),
            photo: self.photo.clone(),
            pin: self.pin.clone(),
            address: self.address.clone(),
            phone: self.phone.clone(),
            permissions: None,
        }
    }
}
