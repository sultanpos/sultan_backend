use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "purchase_orders")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub is_deleted: bool,
    pub branch_id: i64,
    pub supplier_id: Option<i64>,
    pub number: String,
    pub reference_number: Option<String>,
    pub status: String,
    pub order_date: Option<String>,
    pub expected_date: Option<String>,
    pub received_date: Option<String>,
    pub subtotal: i64,
    pub discount_amount: i64,
    pub total_amount: i64,
    pub payment_status: String,
    pub payment_due_date: Option<String>,
    pub paid_amount: i64,
    pub returned_amount: i64,
    pub notes: Option<String>,
    pub metadata: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::purchase_order_item::Entity")]
    PurchaseOrderItems,
    #[sea_orm(has_many = "super::purchase_payment::Entity")]
    PurchasePayments,
}

impl Related<super::purchase_order_item::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PurchaseOrderItems.def()
    }
}

impl Related<super::purchase_payment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PurchasePayments.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn to_domain(
        &self,
        items: Vec<crate::domain::model::purchase_order::PurchaseOrderItem>,
        payments: Vec<crate::domain::model::purchase_order::PurchasePayment>,
    ) -> crate::domain::model::purchase_order::PurchaseOrder {
        use crate::domain::model::purchase_order::{
            PaymentStatus, PurchaseOrder, PurchaseOrderStatus,
        };
        use std::str::FromStr;

        PurchaseOrder {
            id: self.id,
            created_at: super::super::parse_sqlite_date(&self.created_at),
            updated_at: super::super::parse_sqlite_date(&self.updated_at),
            deleted_at: self
                .deleted_at
                .as_ref()
                .map(|d| super::super::parse_sqlite_date(d)),
            is_deleted: self.is_deleted,
            branch_id: self.branch_id,
            supplier_id: self.supplier_id,
            number: self.number.clone(),
            reference_number: self.reference_number.clone(),
            status: PurchaseOrderStatus::from_str(&self.status).unwrap_or_default(),
            order_date: self
                .order_date
                .as_ref()
                .map(|d| super::super::parse_sqlite_date(d)),
            expected_date: self
                .expected_date
                .as_ref()
                .map(|d| super::super::parse_sqlite_date(d)),
            received_date: self
                .received_date
                .as_ref()
                .map(|d| super::super::parse_sqlite_date(d)),
            subtotal: self.subtotal,
            discount_amount: self.discount_amount,
            total_amount: self.total_amount,
            payment_status: PaymentStatus::from_str(&self.payment_status).unwrap_or_default(),
            payment_due_date: self
                .payment_due_date
                .as_ref()
                .map(|d| super::super::parse_sqlite_date(d)),
            paid_amount: self.paid_amount,
            returned_amount: self.returned_amount,
            notes: self.notes.clone(),
            metadata: self
                .metadata
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok()),
            items,
            payments,
        }
    }
}
