use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// SeaORM entity for NumberSequence table
///
/// This entity represents the `number_sequences` table in the database.
/// It follows the standard Sultan pattern with:
/// - Automatic timestamps (created_at, updated_at)
/// - Snowflake ID as primary key
/// - Unique constraint on (prefix, branch_id, year, month)
///
/// Number sequences are used to generate sequential numbers for various entities
/// like customers, suppliers, invoices, etc. They support:
/// - Global numbering (when branch_id is None)
/// - Branch-specific numbering (when branch_id is Some)
/// - Optional month-based segmentation (when month is Some)
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "number_sequences")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub created_at: String,
    pub updated_at: String,
    pub prefix: String,
    pub branch_id: Option<i64>,
    pub year: i32,
    pub month: Option<i32>,
    pub last_number: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Converts the SeaORM model to the domain model
    pub fn to_domain(&self) -> crate::domain::model::number::NumberSequence {
        crate::domain::model::number::NumberSequence {
            id: self.id,
            created_at: super::super::parse_sqlite_date(&self.created_at),
            updated_at: super::super::parse_sqlite_date(&self.updated_at),
            prefix: self.prefix.clone(),
            branch_id: self.branch_id,
            year: self.year,
            month: self.month,
            last_number: self.last_number,
        }
    }
}
