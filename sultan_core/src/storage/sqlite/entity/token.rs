use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use crate::domain::Error;

/// SeaORM entity for Token (refresh_tokens) table
///
/// This entity represents the `refresh_tokens` table in the database.
/// Unlike most Sultan entities, tokens:
/// - Use auto-increment primary key (not Snowflake ID)
/// - Are physically deleted (not soft-deleted)
/// - Don't have created_at/updated_at/deleted_at timestamps
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "refresh_tokens")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub user_id: i64,
    pub expired_at: String,
    pub token: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Converts the SeaORM model to the domain model
    ///
    /// # Errors
    ///
    /// Returns an error if the `expired_at` timestamp cannot be parsed as RFC3339.
    pub fn to_domain(&self) -> Result<crate::domain::model::token::Token, Error> {
        let expired_at = chrono::DateTime::parse_from_rfc3339(&self.expired_at)
            .map_err(|e| Error::Internal(format!("Failed to parse expired_at: {}", e)))?
            .with_timezone(&chrono::Utc);

        Ok(crate::domain::model::token::Token {
            id: self.id,
            user_id: self.user_id,
            expired_at,
            token: self.token.clone(),
        })
    }
}
