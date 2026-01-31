use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// SeaORM entity for sell_discounts table
///
/// This entity represents the `sell_discounts` table in the database.
/// It follows the standard Sultan pattern with:
/// - Soft delete support (is_deleted, deleted_at)
/// - Automatic timestamps (created_at, updated_at)
/// - Snowflake ID as primary key
/// - Discount specific fields (sell_price_id, discount_formula, calculated_price, customer_level)
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "sell_discounts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub is_deleted: bool,
    pub sell_price_id: i64,
    pub quantity: Option<i64>,
    pub discount_formula: String,
    pub calculated_price: i64,
    pub customer_level: Option<i64>,
    pub metadata: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Converts the SeaORM model to the domain model
    pub fn to_domain(&self) -> crate::domain::model::sell_price::SellDiscount {
        crate::domain::model::sell_price::SellDiscount {
            id: self.id,
            created_at: super::super::parse_sqlite_date(&self.created_at),
            updated_at: super::super::parse_sqlite_date(&self.updated_at),
            deleted_at: self
                .deleted_at
                .as_ref()
                .map(|d| super::super::parse_sqlite_date(d)),
            is_deleted: self.is_deleted,
            sell_price_id: self.sell_price_id,
            quantity: self.quantity.unwrap_or(0),
            discount_formula: Some(self.discount_formula.clone()),
            calculated_price: self.calculated_price,
            customer_level: self.customer_level,
            metadata: self
                .metadata
                .as_ref()
                .and_then(|m| serde_json::from_str(m).ok()),
        }
    }
}
