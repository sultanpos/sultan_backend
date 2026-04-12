use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "purchase_order_items")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub created_at: String,
    pub updated_at: String,
    pub purchase_order_id: i64,
    pub product_variant_id: i64,
    pub product_name: String,
    pub variant_name: Option<String>,
    pub barcode: Option<String>,
    pub quantity: i64,
    pub unit_cost: i64,
    pub discount_amount: i64,
    pub total_cost: i64,
    pub metadata: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::purchase_order::Entity",
        from = "Column::PurchaseOrderId",
        to = "super::purchase_order::Column::Id"
    )]
    PurchaseOrder,
}

impl Related<super::purchase_order::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PurchaseOrder.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn to_domain(&self) -> crate::domain::model::purchase_order::PurchaseOrderItem {
        crate::domain::model::purchase_order::PurchaseOrderItem {
            id: self.id,
            created_at: super::super::parse_sqlite_date(&self.created_at),
            updated_at: super::super::parse_sqlite_date(&self.updated_at),
            purchase_order_id: self.purchase_order_id,
            product_variant_id: self.product_variant_id,
            product_name: self.product_name.clone(),
            variant_name: self.variant_name.clone(),
            barcode: self.barcode.clone(),
            quantity: self.quantity,
            unit_cost: self.unit_cost,
            discount_amount: self.discount_amount,
            total_cost: self.total_cost,
            metadata: self
                .metadata
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok()),
        }
    }
}
