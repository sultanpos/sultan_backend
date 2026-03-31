use chrono::Utc;
use serde_json::Value;

use super::Update;

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
