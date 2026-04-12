use chrono::Utc;
use serde_json::Value;

use super::Update;

#[derive(Debug, Clone)]
pub struct PaymentChannel {
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub deleted_at: Option<chrono::DateTime<Utc>>,
    pub is_deleted: bool,
    pub branch_id: Option<i64>,
    pub name: String,
    pub priority: i64,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct PaymentChannelCreate {
    pub branch_id: Option<i64>,
    pub name: String,
    pub priority: i64,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Default)]
pub struct PaymentChannelUpdate {
    pub name: Option<String>,
    pub priority: Option<i64>,
    pub metadata: Update<Value>,
}

/// Used for bulk-updating the priority ordering of multiple channels at once.
#[derive(Debug, Clone)]
pub struct PaymentChannelPriorityUpdate {
    pub id: i64,
    pub priority: i64,
}

#[derive(Debug, Clone, Default)]
pub struct PaymentChannelFilter {
    pub branch_id: Option<i64>,
    pub name: Option<String>,
}
