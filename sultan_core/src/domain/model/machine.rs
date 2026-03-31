use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Update, product::SortDirection};

/// Sort fields available for machine listing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MachineSortField {
    Name,
    CreatedAt,
}

/// Cursor for keyset (cursor-based) pagination over machines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineCursor {
    /// Value of the primary sort field from the last item of the previous page.
    pub field_value: String,
    /// ID of the last item of the previous page (tiebreaker).
    pub id: i64,
}

/// Options for querying a list of machines with cursor-based pagination.
#[derive(Debug, Clone)]
pub struct MachineQuery {
    pub filter: MachineFilter,
    pub sort_field: MachineSortField,
    pub sort_direction: SortDirection,
    pub cursor: Option<MachineCursor>,
    pub limit: u64,
}

#[derive(Debug, Clone)]
pub struct Machine {
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub deleted_at: Option<chrono::DateTime<Utc>>,
    pub is_deleted: bool,
    pub branch_id: i64,
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct MachineCreate {
    pub branch_id: i64,
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Default)]
pub struct MachineUpdate {
    pub key: Option<String>,
    pub name: Option<String>,
    pub description: Update<String>,
    pub metadata: Update<Value>,
}

#[derive(Debug, Clone, Default)]
pub struct MachineFilter {
    pub branch_id: Option<i64>,
    pub name: Option<String>,
}

/// A page of machines with an optional cursor pointing to the next page.
#[derive(Debug, Clone)]
pub struct MachinePage {
    pub items: Vec<Machine>,
    pub next_cursor: Option<MachineCursor>,
}
