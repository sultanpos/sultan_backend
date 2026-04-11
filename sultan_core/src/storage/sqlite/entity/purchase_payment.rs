use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "purchase_payments")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub created_at: String,
    pub purchase_order_id: i64,
    pub amount: i64,
    pub payment_channel_id: i64,
    pub paid_at: String,
    pub reference: Option<String>,
    pub notes: Option<String>,
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
    pub fn to_domain(&self) -> crate::domain::model::purchase_order::PurchasePayment {
        crate::domain::model::purchase_order::PurchasePayment {
            id: self.id,
            created_at: super::super::parse_sqlite_date(&self.created_at),
            purchase_order_id: self.purchase_order_id,
            amount: self.amount,
            payment_channel_id: self.payment_channel_id,
            paid_at: super::super::parse_sqlite_date(&self.paid_at),
            reference: self.reference.clone(),
            notes: self.notes.clone(),
        }
    }
}
