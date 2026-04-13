use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Update, product::SortDirection};

/// Sort fields available for supplier listing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupplierSortField {
    Id,
    UpdatedAt,
    Name,
}

/// Cursor for keyset (cursor-based) pagination over suppliers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierCursor {
    /// Value of the primary sort field from the last item of the previous page.
    pub field_value: String,
    /// ID of the last item of the previous page (tiebreaker).
    pub id: i64,
}

/// Options for querying a list of suppliers with cursor-based pagination.
#[derive(Debug, Clone)]
pub struct SupplierQuery {
    pub filter: SupplierFilter,
    pub sort_field: SupplierSortField,
    pub sort_direction: SortDirection,
    pub cursor: Option<SupplierCursor>,
    pub limit: u64,
}

/// A page of suppliers with an optional cursor pointing to the next page.
#[derive(Debug, Clone)]
pub struct SupplierPage {
    pub items: Vec<Supplier>,
    pub next_cursor: Option<SupplierCursor>,
}

#[derive(Debug, Clone)]
pub struct Supplier {
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub deleted_at: Option<chrono::DateTime<Utc>>,
    pub is_deleted: bool,
    pub name: String,
    pub code: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub npwp: Option<String>,
    pub npwp_name: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct SupplierCreate {
    pub name: String,
    pub code: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub npwp: Option<String>,
    pub npwp_name: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Default)]
pub struct SupplierUpdate {
    pub name: Option<String>,
    pub code: Update<String>,
    pub email: Update<String>,
    pub address: Update<String>,
    pub phone: Update<String>,
    pub npwp: Update<String>,
    pub npwp_name: Update<String>,
    pub metadata: Update<Value>,
}

#[derive(Debug, Clone, Default)]
pub struct SupplierFilter {
    pub name: Option<String>,
    pub code: Option<String>,
    pub phone: Option<String>,
    pub npwp: Option<String>,
    pub email: Option<String>,
}
