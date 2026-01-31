use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// SeaORM entity for Branch table
///
/// This entity represents the `branches` table in the database.
/// It follows the standard Sultan pattern with:
/// - Soft delete support (is_deleted, deleted_at)
/// - Automatic timestamps (created_at, updated_at)
/// - Snowflake ID as primary key
/// - Branch-specific fields (is_main, code, contact info)
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "branches")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub is_deleted: bool,
    pub is_main: bool,
    pub name: String,
    pub code: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub npwp: Option<String>,
    pub image: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Converts the SeaORM model to the domain model
    pub fn to_domain(&self) -> crate::domain::model::branch::Branch {
        crate::domain::model::branch::Branch {
            id: self.id,
            created_at: super::super::parse_sqlite_date(&self.created_at),
            updated_at: super::super::parse_sqlite_date(&self.updated_at),
            deleted_at: self
                .deleted_at
                .as_ref()
                .map(|d| super::super::parse_sqlite_date(d)),
            is_deleted: self.is_deleted,
            is_main: self.is_main,
            name: self.name.clone(),
            code: self.code.clone(),
            address: self.address.clone(),
            phone: self.phone.clone(),
            npwp: self.npwp.clone(),
            image: self.image.clone(),
        }
    }
}
