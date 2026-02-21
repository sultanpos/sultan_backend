use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// SeaORM entity for sell_prices table
///
/// This entity represents the `sell_prices` table in the database.
/// It follows the standard Sultan pattern with:
/// - Soft delete support (is_deleted, deleted_at)
/// - Automatic timestamps (created_at, updated_at)
/// - Snowflake ID as primary key
/// - Sell price specific fields (product_variant_id, uom_id, price, quantity, branch_id)
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "sell_prices")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub is_deleted: bool,
    pub branch_id: Option<i64>,
    pub product_variant_id: i64,
    pub uom_id: Option<i64>,
    pub quantity: i64,
    pub price: i64,
    pub metadata: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::product_variant::Entity",
        from = "Column::ProductVariantId",
        to = "super::product_variant::Column::Id"
    )]
    ProductVariant,

    #[sea_orm(has_many = "super::sell_discount::Entity")]
    SellDiscount,
}

impl Related<super::product_variant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProductVariant.def()
    }
}

impl Related<super::sell_discount::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SellDiscount.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Converts the SeaORM model to the domain model
    pub fn to_domain(&self) -> crate::domain::model::sell_price::SellPrice {
        crate::domain::model::sell_price::SellPrice {
            id: self.id,
            created_at: super::super::parse_sqlite_date(&self.created_at),
            updated_at: super::super::parse_sqlite_date(&self.updated_at),
            deleted_at: self
                .deleted_at
                .as_ref()
                .map(|d| super::super::parse_sqlite_date(d)),
            is_deleted: self.is_deleted,
            branch_id: self.branch_id,
            product_variant_id: self.product_variant_id,
            uom_id: self.uom_id.unwrap_or(0),
            quantity: self.quantity,
            price: self.price,
            metadata: self
                .metadata
                .as_ref()
                .and_then(|m| serde_json::from_str(m).ok()),
            discounts: vec![],
        }
    }
}
