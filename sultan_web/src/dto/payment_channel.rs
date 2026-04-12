use super::i64_to_string;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sultan_core::domain::model::{Update, payment_channel::PaymentChannel};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct PaymentChannelCreateRequest {
    /// Channel name (e.g. "Cash", "QRIS", "Debit BCA")
    #[validate(length(
        min = 1,
        max = 100,
        message = "Name must be between 1 and 100 characters"
    ))]
    #[schema(example = "Cash")]
    pub name: String,

    /// Display order (lower = shown first)
    #[schema(example = 1)]
    pub priority: i64,

    /// Optional branch scope; null means global
    #[schema(example = "1234567890", value_type = Option<String>)]
    #[serde(default, deserialize_with = "super::option_string_to_i64")]
    pub branch_id: Option<i64>,

    /// Arbitrary extra data
    #[schema(value_type = Option<Value>)]
    pub metadata: Option<Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaymentChannelCreateResponse {
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub id: i64,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct PaymentChannelUpdateRequest {
    /// New name (omit to leave unchanged)
    #[validate(length(
        min = 1,
        max = 100,
        message = "Name must be between 1 and 100 characters"
    ))]
    #[schema(example = "QRIS")]
    pub name: Option<String>,

    /// New priority (omit to leave unchanged)
    #[schema(example = 2)]
    pub priority: Option<i64>,

    /// Set to a value to update, set to null to clear
    #[schema(value_type = Option<Value>)]
    pub metadata: Update<Value>,
}

/// Single item in a bulk priority-update request
#[derive(Debug, Deserialize, ToSchema)]
pub struct PriorityUpdateItem {
    #[schema(example = "1234567890", value_type = String)]
    #[serde(deserialize_with = "super::string_to_i64")]
    pub id: i64,

    #[schema(example = 1)]
    pub priority: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BulkPriorityUpdateRequest {
    pub channels: Vec<PriorityUpdateItem>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaymentChannelResponse {
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub id: i64,

    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,

    #[schema(example = "1234567890", value_type = Option<String>)]
    #[serde(serialize_with = "super::option_i64_to_string")]
    pub branch_id: Option<i64>,

    #[schema(example = "Cash")]
    pub name: String,

    #[schema(example = 1)]
    pub priority: i64,

    #[schema(value_type = Option<Value>)]
    pub metadata: Option<Value>,
}

impl From<PaymentChannel> for PaymentChannelResponse {
    fn from(c: PaymentChannel) -> Self {
        Self {
            id: c.id,
            created_at: c.created_at,
            updated_at: c.updated_at,
            branch_id: c.branch_id,
            name: c.name,
            priority: c.priority,
            metadata: c.metadata,
        }
    }
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct PaymentChannelQueryParams {
    /// Filter by channel name (partial match)
    #[schema(example = "Cash")]
    pub name: Option<String>,

    /// Filter by branch ID; omit for global channels
    #[schema(example = "1234567890", value_type = Option<String>)]
    #[serde(default, deserialize_with = "super::option_string_to_i64")]
    pub branch_id: Option<i64>,
}
