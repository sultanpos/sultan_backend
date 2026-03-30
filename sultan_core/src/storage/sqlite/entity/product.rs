use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// SeaORM entity for Product table
///
/// This entity represents the `products` table in the database.
/// It follows the standard Sultan pattern with:
/// - Soft delete support (is_deleted, deleted_at)
/// - Automatic timestamps (created_at, updated_at)
/// - Snowflake ID as primary key
/// - Product-specific fields (name, type, flags, metadata)
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "products")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub is_deleted: bool,
    pub name: String,
    pub description: Option<String>,
    pub product_type: String,
    pub main_image: Option<String>,
    pub sellable: bool,
    pub buyable: bool,
    pub editable_price: bool,
    pub variant_count: i32,
    pub metadata: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::product_variant::Entity")]
    ProductVariants,
}

impl Related<super::product_variant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProductVariants.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

/// Linked definition for traversing Product → ProductCategory → Category
pub struct ProductToCategories;

impl Linked for ProductToCategories {
    type FromEntity = Entity;
    type ToEntity = super::category::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            super::product_category::Relation::Product.def().rev(),
            super::product_category::Relation::Category.def(),
        ]
    }
}

impl Model {
    /// Converts the SeaORM model to the domain model
    pub fn to_domain(&self) -> crate::domain::model::product::Product {
        crate::domain::model::product::Product {
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
            product_type: self
                .product_type
                .parse()
                .unwrap_or(crate::domain::model::product::ProductType::Product),
            main_image: self.main_image.clone(),
            sellable: self.sellable,
            buyable: self.buyable,
            editable_price: self.editable_price,
            variant_count: self.variant_count,
            metadata: self
                .metadata
                .as_ref()
                .and_then(|m| serde_json::from_str(m).ok()),
            categories: vec![],
            variants: vec![],
        }
    }
}
