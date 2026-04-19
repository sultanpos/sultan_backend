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

#[derive(Debug, Deserialize, ToSchema, Default)]
#[serde(default)]
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
        if let Some(supplier_id) = self.supplier_id
            && supplier_id <= 0
        {
            let mut e = ValidationError::new("range");
            e.message = Some("Supplier ID must be greater than 0".into());
            errors.add("supplier_id", e);
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sultan_core::domain::model::Update;

    fn default_update_request() -> PurchaseOrderUpdateRequest {
        PurchaseOrderUpdateRequest {
            supplier_id: None,
            reference_number: Update::Unchanged,
            order_date: Update::Unchanged,
            expected_date: Update::Unchanged,
            payment_due_date: Update::Unchanged,
            discount_amount: Update::Unchanged,
            notes: Update::Unchanged,
            metadata: Update::Unchanged,
        }
    }

    // --- PurchaseOrderCreateRequest ---

    #[test]
    fn test_create_request_valid() {
        let req = PurchaseOrderCreateRequest {
            supplier_id: Some(1),
            reference_number: Some("PO-0001".to_string()),
            order_date: None,
            expected_date: None,
            payment_due_date: None,
            discount_amount: Some(0),
            notes: None,
            metadata: None,
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_create_request_reference_number_too_long() {
        let req = PurchaseOrderCreateRequest {
            supplier_id: None,
            reference_number: Some("a".repeat(101)),
            order_date: None,
            expected_date: None,
            payment_due_date: None,
            discount_amount: None,
            notes: None,
            metadata: None,
        };
        let err = req.validate().unwrap_err();
        assert!(err.field_errors().contains_key("reference_number"));
    }

    #[test]
    fn test_create_request_discount_amount_negative() {
        let req = PurchaseOrderCreateRequest {
            supplier_id: None,
            reference_number: None,
            order_date: None,
            expected_date: None,
            payment_due_date: None,
            discount_amount: Some(-1),
            notes: None,
            metadata: None,
        };
        let err = req.validate().unwrap_err();
        assert!(err.field_errors().contains_key("discount_amount"));
    }

    // --- PurchaseOrderUpdateRequest ---

    #[test]
    fn test_update_request_valid_no_supplier() {
        let req = default_update_request();
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_update_request_valid_supplier_positive() {
        let req = PurchaseOrderUpdateRequest {
            supplier_id: Some(5),
            ..default_update_request()
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_update_request_supplier_id_zero() {
        let req = PurchaseOrderUpdateRequest {
            supplier_id: Some(0),
            ..default_update_request()
        };
        let err = req.validate().unwrap_err();
        assert!(err.field_errors().contains_key("supplier_id"));
    }

    #[test]
    fn test_update_request_supplier_id_negative() {
        let req = PurchaseOrderUpdateRequest {
            supplier_id: Some(-1),
            ..default_update_request()
        };
        let err = req.validate().unwrap_err();
        assert!(err.field_errors().contains_key("supplier_id"));
    }
}
