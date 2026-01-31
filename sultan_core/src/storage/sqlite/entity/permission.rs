use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// SeaORM entity for Permission table.
///
/// This entity represents the `permissions` table in the database.
/// It stores user permissions with optional branch scope.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "permissions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub user_id: i64,
    pub branch_id: Option<i64>,
    pub resource: i32,
    pub action: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Converts the SeaORM model to the domain model.
    pub fn to_domain(&self) -> crate::domain::model::permission::Permission {
        crate::domain::model::permission::Permission {
            user_id: self.user_id,
            branch_id: self.branch_id,
            resource: self.resource,
            action: self.action,
        }
    }
}
