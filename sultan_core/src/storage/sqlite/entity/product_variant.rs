use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// SeaORM entity for ProductVariant table
///
/// This entity represents the `product_variants` table in the database.
/// It follows the standard Sultan pattern with:
/// - Soft delete support (is_deleted, deleted_at)
/// - Automatic timestamps (created_at, updated_at)
/// - Snowflake ID as primary key
/// - Reference to parent Product
/// - Variant-specific fields (barcode, name, metadata)
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "product_variants")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub is_deleted: bool,
    pub product_id: i64,
    pub barcode: Option<String>,
    pub name: Option<String>,
    pub metadata: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::product::Entity",
        from = "Column::ProductId",
        to = "super::product::Column::Id"
    )]
    Product,

    #[sea_orm(has_many = "super::sell_price::Entity")]
    SellPrice,
}

impl Related<super::product::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Product.def()
    }
}

impl Related<super::sell_price::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SellPrice.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Converts the SeaORM model to a domain ProductVariant
    pub fn to_domain(&self) -> crate::domain::model::product::ProductVariant {
        crate::domain::model::product::ProductVariant {
            id: self.id,
            created_at: super::super::parse_sqlite_date(&self.created_at),
            updated_at: super::super::parse_sqlite_date(&self.updated_at),
            deleted_at: self
                .deleted_at
                .as_ref()
                .map(|d| super::super::parse_sqlite_date(d)),
            is_deleted: self.is_deleted,
            barcode: self.barcode.clone(),
            name: self.name.clone(),
            metadata: self
                .metadata
                .as_ref()
                .and_then(|m| serde_json::from_str(m).ok()),
            sell_prices: vec![],
        }
    }
}
