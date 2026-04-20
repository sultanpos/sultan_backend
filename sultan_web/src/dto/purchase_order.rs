use super::{i64_to_string, option_i64_to_string, option_string_to_i64, update_string_to_i64};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sultan_core::domain::model::Update;
use sultan_core::domain::model::purchase_order::{
    PurchaseOrder, PurchaseOrderItem, PurchasePayment,
};
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

#[derive(Debug, Serialize, ToSchema)]
pub struct PurchaseOrderResponse {
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub branch_id: i64,
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "option_i64_to_string")]
    pub supplier_id: Option<i64>,
    pub number: String,
    pub reference_number: Option<String>,
    #[schema(example = "draft")]
    pub status: String,
    pub order_date: Option<chrono::DateTime<Utc>>,
    pub expected_date: Option<chrono::DateTime<Utc>>,
    pub received_date: Option<chrono::DateTime<Utc>>,
    pub subtotal: i64,
    pub discount_amount: i64,
    pub total_amount: i64,
    #[schema(example = "unpaid")]
    pub payment_status: String,
    pub payment_due_date: Option<chrono::DateTime<Utc>>,
    pub paid_amount: i64,
    pub returned_amount: i64,
    pub notes: Option<String>,
    pub metadata: Option<Value>,
    pub items: Vec<PurchaseOrderItemResponse>,
    pub payments: Vec<PurchasePaymentResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PurchaseOrderItemResponse {
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub purchase_order_id: i64,
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub product_variant_id: i64,
    pub product_name: String,
    pub variant_name: Option<String>,
    pub barcode: Option<String>,
    pub quantity: i64,
    pub unit_cost: i64,
    pub discount_amount: i64,
    pub total_cost: i64,
    pub metadata: Option<Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PurchasePaymentResponse {
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub purchase_order_id: i64,
    pub amount: i64,
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub payment_channel_id: i64,
    pub paid_at: chrono::DateTime<Utc>,
    pub reference: Option<String>,
    pub notes: Option<String>,
}

impl From<PurchaseOrder> for PurchaseOrderResponse {
    fn from(po: PurchaseOrder) -> Self {
        Self {
            id: po.id,
            created_at: po.created_at,
            updated_at: po.updated_at,
            branch_id: po.branch_id,
            supplier_id: po.supplier_id,
            number: po.number,
            reference_number: po.reference_number,
            status: po.status.to_string(),
            order_date: po.order_date,
            expected_date: po.expected_date,
            received_date: po.received_date,
            subtotal: po.subtotal,
            discount_amount: po.discount_amount,
            total_amount: po.total_amount,
            payment_status: po.payment_status.to_string(),
            payment_due_date: po.payment_due_date,
            paid_amount: po.paid_amount,
            returned_amount: po.returned_amount,
            notes: po.notes,
            metadata: po.metadata,
            items: po
                .items
                .into_iter()
                .map(PurchaseOrderItemResponse::from)
                .collect(),
            payments: po
                .payments
                .into_iter()
                .map(PurchasePaymentResponse::from)
                .collect(),
        }
    }
}

impl From<PurchaseOrderItem> for PurchaseOrderItemResponse {
    fn from(item: PurchaseOrderItem) -> Self {
        Self {
            id: item.id,
            created_at: item.created_at,
            updated_at: item.updated_at,
            purchase_order_id: item.purchase_order_id,
            product_variant_id: item.product_variant_id,
            product_name: item.product_name,
            variant_name: item.variant_name,
            barcode: item.barcode,
            quantity: item.quantity,
            unit_cost: item.unit_cost,
            discount_amount: item.discount_amount,
            total_cost: item.total_cost,
            metadata: item.metadata,
        }
    }
}

impl From<PurchasePayment> for PurchasePaymentResponse {
    fn from(payment: PurchasePayment) -> Self {
        Self {
            id: payment.id,
            created_at: payment.created_at,
            purchase_order_id: payment.purchase_order_id,
            amount: payment.amount,
            payment_channel_id: payment.payment_channel_id,
            paid_at: payment.paid_at,
            reference: payment.reference,
            notes: payment.notes,
        }
    }
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
    use chrono::Utc;
    use serde_json::json;
    use sultan_core::domain::model::Update;
    use sultan_core::domain::model::purchase_order::{
        PaymentStatus, PurchaseOrder, PurchaseOrderItem, PurchaseOrderStatus, PurchasePayment,
    };

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

    #[test]
    fn test_purchase_order_response_from_domain_maps_fields() {
        let now = Utc::now();
        let item = PurchaseOrderItem {
            id: 11,
            created_at: now,
            updated_at: now,
            purchase_order_id: 1,
            product_variant_id: 101,
            product_name: "Coffee Beans".to_string(),
            variant_name: Some("1kg".to_string()),
            barcode: Some("1234567890123".to_string()),
            quantity: 2,
            unit_cost: 10000,
            discount_amount: 500,
            total_cost: 19500,
            metadata: Some(json!({"batch": "A1"})),
        };
        let payment = PurchasePayment {
            id: 21,
            created_at: now,
            purchase_order_id: 1,
            amount: 19500,
            payment_channel_id: 7,
            paid_at: now,
            reference: Some("PAY-001".to_string()),
            notes: Some("Paid in full".to_string()),
        };
        let po = PurchaseOrder {
            id: 1,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            is_deleted: false,
            branch_id: 2,
            supplier_id: Some(9),
            number: "PO-0001".to_string(),
            reference_number: Some("REF-001".to_string()),
            status: PurchaseOrderStatus::Ordered,
            order_date: Some(now),
            expected_date: Some(now),
            received_date: None,
            subtotal: 20000,
            discount_amount: 500,
            total_amount: 19500,
            payment_status: PaymentStatus::Partial,
            payment_due_date: Some(now),
            paid_amount: 10000,
            returned_amount: 0,
            notes: Some("urgent".to_string()),
            metadata: Some(json!({"source": "test"})),
            items: vec![item],
            payments: vec![payment],
        };

        let response = PurchaseOrderResponse::from(po);

        assert_eq!(response.id, 1);
        assert_eq!(response.branch_id, 2);
        assert_eq!(response.supplier_id, Some(9));
        assert_eq!(response.status, "ordered");
        assert_eq!(response.payment_status, "partial");
        assert_eq!(response.items.len(), 1);
        assert_eq!(response.items[0].product_name, "Coffee Beans");
        assert_eq!(response.payments.len(), 1);
        assert_eq!(response.payments[0].payment_channel_id, 7);
    }

    #[test]
    fn test_purchase_order_response_from_domain_with_none_optional_fields() {
        let now = Utc::now();
        let po = PurchaseOrder {
            id: 2,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            is_deleted: false,
            branch_id: 3,
            supplier_id: None,
            number: "PO-0002".to_string(),
            reference_number: None,
            status: PurchaseOrderStatus::Draft,
            order_date: None,
            expected_date: None,
            received_date: None,
            subtotal: 0,
            discount_amount: 0,
            total_amount: 0,
            payment_status: PaymentStatus::Unpaid,
            payment_due_date: None,
            paid_amount: 0,
            returned_amount: 0,
            notes: None,
            metadata: None,
            items: vec![],
            payments: vec![],
        };

        let response = PurchaseOrderResponse::from(po);

        assert_eq!(response.id, 2);
        assert_eq!(response.supplier_id, None);
        assert_eq!(response.status, "draft");
        assert_eq!(response.payment_status, "unpaid");
        assert!(response.items.is_empty());
        assert!(response.payments.is_empty());
    }
}
