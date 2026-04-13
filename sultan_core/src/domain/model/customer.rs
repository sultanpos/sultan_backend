use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Update, product::SortDirection};

/// Sort fields available for customer listing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustomerSortField {
    Id,
    UpdatedAt,
    Name,
}

/// Cursor for keyset (cursor-based) pagination over customers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerCursor {
    /// Value of the primary sort field from the last item of the previous page.
    pub field_value: String,
    /// ID of the last item of the previous page (tiebreaker).
    pub id: i64,
}

/// Options for querying a list of customers with cursor-based pagination.
#[derive(Debug, Clone)]
pub struct CustomerQuery {
    pub filter: CustomerFilter,
    pub sort_field: CustomerSortField,
    pub sort_direction: SortDirection,
    pub cursor: Option<CustomerCursor>,
    pub limit: u64,
}

/// A page of customers with an optional cursor pointing to the next page.
#[derive(Debug, Clone)]
pub struct CustomerPage {
    pub items: Vec<Customer>,
    pub next_cursor: Option<CustomerCursor>,
}

#[derive(Debug, Clone)]
pub struct Customer {
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub deleted_at: Option<chrono::DateTime<Utc>>,
    pub is_deleted: bool,
    pub number: String,
    pub name: String,
    pub address: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub level: i32,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct CustomerCreate {
    pub number: String,
    pub name: String,
    pub address: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub level: i32,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Default)]
pub struct CustomerUpdate {
    pub number: Option<String>,
    pub name: Option<String>,
    pub address: Update<String>,
    pub email: Update<String>,
    pub phone: Update<String>,
    pub level: Option<i32>,
    pub metadata: Update<Value>,
}

#[derive(Debug, Clone, Default)]
pub struct CustomerFilter {
    pub number: Option<String>,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub level: Option<i32>,
}
