use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// SeaORM entity for Category table
///
/// This entity represents the `categories` table in the database.
/// It follows the standard Sultan pattern with:
/// - Soft delete support (is_deleted, deleted_at)
/// - Automatic timestamps (created_at, updated_at)
/// - Snowflake ID as primary key
/// - Hierarchical structure support (parent_id for tree organization)
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "categories")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub is_deleted: bool,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Converts the SeaORM model to the domain model
    pub fn to_domain(&self) -> crate::domain::model::category::Category {
        crate::domain::model::category::Category {
            id: self.id,
            created_at: super::super::parse_sqlite_date(&self.created_at),
            updated_at: super::super::parse_sqlite_date(&self.updated_at),
            deleted_at: self
                .deleted_at
                .as_ref()
                .map(|d| super::super::parse_sqlite_date(d)),
            is_deleted: self.is_deleted,
            name: self.name.clone(),
            description: self.description.clone(),
            children: Some(Vec::new()), // Initialize with empty children
        }
    }
}
