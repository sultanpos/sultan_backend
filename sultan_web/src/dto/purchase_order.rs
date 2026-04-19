use super::{i64_to_string, option_string_to_i64, update_string_to_i64};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sultan_core::domain::model::Update;
use utoipa::ToSchema;
use validator::{Validate, ValidationError, ValidationErrors};

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct PurchaseOrderCreateRequest {
    #[schema(example = "15")]
    #[serde(default, deserialize_with = "option_string_to_i64")]
    pub supplier_id: Option<i64>,
    #[validate(length(
        min = 1,
        max = 100,
        message = "Number must be between 1 and 100 characters"
    ))]
    #[schema(example = "PO-0001")]
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

#[derive(Debug, Deserialize, ToSchema)]
pub struct PurchaseOrderUpdateRequest {
    #[schema(example = "15")]
    #[serde(default, deserialize_with = "option_string_to_i64")]
    pub supplier_id: Option<i64>,
    #[schema(example = "PO-0001")]
    pub reference_number: Update<String>,
    pub order_date: Update<String>,
    pub expected_date: Update<String>,
    pub payment_due_date: Update<String>,
    #[serde(default, deserialize_with = "update_string_to_i64")]
    pub discount_amount: Update<i64>,
    pub notes: Update<String>,
    pub metadata: Update<Value>,
}

impl Validate for PurchaseOrderUpdateRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        if let Update::Set(amount) = self.discount_amount
            && amount < 0
        {
            let mut e = ValidationError::new("range");
            e.message = Some("Discount amount must be non-negative".into());
            errors.add("discount_amount", e);
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
