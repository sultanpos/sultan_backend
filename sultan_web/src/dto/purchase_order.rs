use super::i64_to_string;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct PurchaseOrderCreateRequest {
    pub branch_id: i64,
    pub supplier_id: Option<i64>,
    #[validate(length(
        min = 1,
        max = 100,
        message = "Number must be between 1 and 100 characters"
    ))]
    #[schema(example = "PO-0001")]
    pub number: String,
    pub reference_number: Option<String>,
    pub order_date: Option<String>,
    pub expected_date: Option<String>,
    pub payment_due_date: Option<String>,
    #[validate(range(min = 0, message = "Discount amount must be non-negative"))]
    pub discount_amount: Option<i64>,
    pub notes: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PurchaseOrderCreateResponse {
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub id: i64,
}
