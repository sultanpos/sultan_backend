use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "cashier_sessions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub is_deleted: bool,
    pub branch_id: i64,
    pub user_id: i64,
    pub opened_at: String,
    pub closed_at: Option<String>,
    pub status: String,
    pub opening_cash: i64,
    pub closing_cash: Option<i64>,
    pub notes: Option<String>,
    pub metadata: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn to_domain(&self) -> crate::domain::model::cashier_session::CashierSession {
        use crate::domain::model::cashier_session::SessionStatus;
        use std::str::FromStr;

        crate::domain::model::cashier_session::CashierSession {
            id: self.id,
            created_at: super::super::parse_sqlite_date(&self.created_at),
            updated_at: super::super::parse_sqlite_date(&self.updated_at),
            deleted_at: self
                .deleted_at
                .as_ref()
                .map(|d| super::super::parse_sqlite_date(d)),
            is_deleted: self.is_deleted,
            branch_id: self.branch_id,
            user_id: self.user_id,
            opened_at: super::super::parse_sqlite_date(&self.opened_at),
            closed_at: self
                .closed_at
                .as_ref()
                .map(|d| super::super::parse_sqlite_date(d)),
            status: SessionStatus::from_str(&self.status).unwrap_or_default(),
            opening_cash: self.opening_cash,
            closing_cash: self.closing_cash,
            notes: self.notes.clone(),
        }
    }
}
