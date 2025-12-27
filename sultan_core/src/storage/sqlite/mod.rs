pub mod branch;
pub mod category;
pub mod customer;
pub mod product;
pub mod supplier;
pub mod token;
pub mod transaction;
pub mod unit;
pub mod user;

pub use branch::SqliteBranchRepository;
pub use category::SqliteCategoryRepository;
pub use customer::SqliteCustomerRepository;
pub use product::SqliteProductRepository;
pub use supplier::SqliteSupplierRepository;
pub use token::SqliteTokenRepository;
pub use unit::SqliteUnitOfMeasureRepository;
pub use user::SqliteUserRepository;

use chrono::{DateTime, NaiveDateTime, Utc};
use serde_json::Value;
use sqlx::{Executor, QueryBuilder, Sqlite};

use crate::domain::{DomainResult, Error};

pub fn parse_sqlite_date(date_str: &str) -> DateTime<Utc> {
    NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%fZ")
        .unwrap_or_default()
        .and_utc()
}

/// Extension trait for QueryBuilder to add common filter patterns
pub trait QueryBuilderExt {
    /// Add a LIKE filter clause if the value is Some
    fn push_like_filter(&mut self, column: &str, value: &Option<String>) -> &mut Self;
}

impl QueryBuilderExt for QueryBuilder<'_, Sqlite> {
    fn push_like_filter(&mut self, column: &str, value: &Option<String>) -> &mut Self {
        if let Some(v) = value {
            self.push(" AND ");
            self.push(column);
            self.push(" LIKE ");
            self.push_bind(format!("%{}%", v));
        }
        self
    }
}

/// Check if a query affected rows, return error if not
pub fn check_rows_affected(
    rows: u64,
    entity: &str,
    id: impl std::fmt::Display,
) -> DomainResult<()> {
    if rows == 0 {
        return Err(Error::NotFound(format!(
            "{} with id {} not found",
            entity, id
        )));
    }
    Ok(())
}

/// Enum representing valid table names in the database.
/// This prevents SQL injection by ensuring only whitelisted tables can be used.
#[derive(Debug, Clone, Copy)]
pub enum TableName {
    Branches,
    Categories,
    Customers,
    Suppliers,
    Users,
    Tokens,
    Permissions,
    Units,
    Products,
    ProductVariants,
}

impl TableName {
    /// Returns the actual table name as a string.
    /// This is safe because all values are hardcoded.
    pub fn as_str(&self) -> &'static str {
        match self {
            TableName::Branches => "branches",
            TableName::Categories => "categories",
            TableName::Customers => "customers",
            TableName::Suppliers => "suppliers",
            TableName::Users => "users",
            TableName::Tokens => "tokens",
            TableName::Permissions => "permissions",
            TableName::Units => "units",
            TableName::Products => "products",
            TableName::ProductVariants => "product_variants",
        }
    }
}

/// Execute a soft delete query.
///
/// # Safety
/// This function uses a whitelist enum (`TableName`) to prevent SQL injection.
/// Only predefined table names can be used.
pub async fn soft_delete<'a, E>(
    executor: E,
    table: TableName,
    id: i64,
) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error>
where
    E: Executor<'a, Database = Sqlite>,
{
    let sql = format!(
        r#"
        UPDATE {} SET
            is_deleted = 1,
            deleted_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
        WHERE id = ? AND is_deleted = 0
        "#,
        table.as_str()
    );
    sqlx::query(&sql).bind(id).execute(executor).await
}

/// Helper to map query results to domain models
pub fn map_results<DbModel, DomainModel>(results: Vec<DbModel>) -> Vec<DomainModel>
where
    DbModel: Into<DomainModel>,
{
    results.into_iter().map(|x| x.into()).collect()
}

/// Helper to convert Update<Value> metadata to Option<String> for database binding
pub fn serialize_metadata_update(
    metadata: &crate::domain::model::update::Update<Value>,
) -> Option<String> {
    metadata
        .to_bind_value()
        .map(|m: Value| serde_json::to_string(&m).unwrap_or_default())
}

/// Serialize metadata JSON to string for database storage
pub fn serialize_metadata(metadata: &Option<serde_json::Value>) -> Option<String> {
    metadata
        .as_ref()
        .map(|m| serde_json::to_string(m).unwrap_or_default())
}
