use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// SeaORM entity for Supplier table
///
/// This entity represents the `suppliers` table in the database.
/// It follows the standard Sultan pattern with:
/// - Soft delete support (is_deleted, deleted_at)
/// - Automatic timestamps (created_at, updated_at)
/// - Snowflake ID as primary key
/// - Supplier-specific fields (name, code, contact info, npwp, metadata)
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "suppliers")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub is_deleted: bool,
    pub name: String,
    pub code: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub npwp: Option<String>,
    pub npwp_name: Option<String>,
    pub metadata: Option<String>, // Stored as JSON string
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Converts the SeaORM model to the domain model
    pub fn to_domain(&self) -> crate::domain::model::supplier::Supplier {
        crate::domain::model::supplier::Supplier {
            id: self.id,
            created_at: super::super::parse_sqlite_date(&self.created_at),
            updated_at: super::super::parse_sqlite_date(&self.updated_at),
            deleted_at: self
                .deleted_at
                .as_ref()
                .map(|d| super::super::parse_sqlite_date(d)),
            is_deleted: self.is_deleted,
            name: self.name.clone(),
            code: self.code.clone(),
            email: self.email.clone(),
            address: self.address.clone(),
            phone: self.phone.clone(),
            npwp: self.npwp.clone(),
            npwp_name: self.npwp_name.clone(),
            metadata: self
                .metadata
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok()),
        }
    }
}
