use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// SeaORM entity for stocks table
///
/// This entity represents the `stocks` table in the database.
/// Unlike other entities, stocks do NOT use soft delete and do NOT have `created_at`.
/// Each record represents inventory for a unique (branch_id, product_variant_id) combination.
///
/// # Schema Notes
///
/// - No `created_at` column (only `updated_at`)
/// - No soft delete (`is_deleted`, `deleted_at`)
/// - Unique constraint on `(branch_id, product_variant_id)`
/// - Snowflake ID as primary key
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "stocks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub updated_at: Option<String>,
    pub branch_id: i64,
    pub product_variant_id: i64,
    pub quantity: i64,
    pub min_stock: Option<i64>,
    pub max_stock: Option<i64>,
    pub last_buy_price: Option<i64>,
    pub metadata: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Converts the SeaORM model to the domain model
    pub fn to_domain(&self) -> crate::domain::model::stock::Stock {
        let updated_at = self
            .updated_at
            .as_ref()
            .map(|d| super::super::parse_sqlite_date(d))
            .unwrap_or_default();

        crate::domain::model::stock::Stock {
            id: self.id,
            created_at: updated_at,
            updated_at,
            branch_id: self.branch_id,
            product_variant_id: self.product_variant_id,
            quantity: self.quantity,
            min_stock: self.min_stock,
            max_stock: self.max_stock,
            last_buy_price: self.last_buy_price,
            metadata: self
                .metadata
                .as_ref()
                .and_then(|m| serde_json::from_str(m).ok()),
        }
    }
}
