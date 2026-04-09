use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::{product::SortDirection, update::Update};

// ============================================================================
// Enums
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PurchaseOrderStatus {
    #[default]
    Draft,
    Ordered,
    Received,
    Cancelled,
}

impl PurchaseOrderStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            PurchaseOrderStatus::Draft => "draft",
            PurchaseOrderStatus::Ordered => "ordered",
            PurchaseOrderStatus::Received => "received",
            PurchaseOrderStatus::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for PurchaseOrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for PurchaseOrderStatus {
    type Err = crate::domain::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(PurchaseOrderStatus::Draft),
            "ordered" => Ok(PurchaseOrderStatus::Ordered),
            "received" => Ok(PurchaseOrderStatus::Received),
            "cancelled" => Ok(PurchaseOrderStatus::Cancelled),
            other => Err(crate::domain::Error::ValidationError(format!(
                "Invalid purchase order status '{}'. Must be one of: draft, ordered, received, cancelled",
                other
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    #[default]
    Unpaid,
    Partial,
    Paid,
}

impl PaymentStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            PaymentStatus::Unpaid => "unpaid",
            PaymentStatus::Partial => "partial",
            PaymentStatus::Paid => "paid",
        }
    }
}

impl std::fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for PaymentStatus {
    type Err = crate::domain::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "unpaid" => Ok(PaymentStatus::Unpaid),
            "partial" => Ok(PaymentStatus::Partial),
            "paid" => Ok(PaymentStatus::Paid),
            other => Err(crate::domain::Error::ValidationError(format!(
                "Invalid payment status '{}'. Must be one of: unpaid, partial, paid",
                other
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PurchasePaymentChannel {
    #[default]
    Cash,
    BankTransfer,
    Card,
    Other,
}

impl PurchasePaymentChannel {
    pub fn as_str(self) -> &'static str {
        match self {
            PurchasePaymentChannel::Cash => "cash",
            PurchasePaymentChannel::BankTransfer => "bank_transfer",
            PurchasePaymentChannel::Card => "card",
            PurchasePaymentChannel::Other => "other",
        }
    }
}

impl std::fmt::Display for PurchasePaymentChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for PurchasePaymentChannel {
    type Err = crate::domain::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cash" => Ok(PurchasePaymentChannel::Cash),
            "bank_transfer" => Ok(PurchasePaymentChannel::BankTransfer),
            "card" => Ok(PurchasePaymentChannel::Card),
            "other" => Ok(PurchasePaymentChannel::Other),
            other => Err(crate::domain::Error::ValidationError(format!(
                "Invalid payment channel '{}'. Must be one of: cash, bank_transfer, card, other",
                other
            ))),
        }
    }
}

// ============================================================================
// Domain structs
// ============================================================================

#[derive(Debug, Clone)]
pub struct PurchaseOrderItem {
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub purchase_order_id: i64,
    pub product_variant_id: i64,
    pub product_name: String,
    pub variant_name: Option<String>,
    pub barcode: Option<String>,
    pub quantity: i64,
    pub unit_cost: i64,
    pub discount_amount: i64,
    pub total_cost: i64,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct PurchasePayment {
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub purchase_order_id: i64,
    pub amount: i64,
    pub channel: PurchasePaymentChannel,
    pub paid_at: chrono::DateTime<Utc>,
    pub reference: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PurchaseOrder {
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub deleted_at: Option<chrono::DateTime<Utc>>,
    pub is_deleted: bool,
    pub branch_id: i64,
    pub supplier_id: Option<i64>,
    pub number: String,
    pub reference_number: Option<String>,
    pub status: PurchaseOrderStatus,
    pub order_date: Option<chrono::DateTime<Utc>>,
    pub expected_date: Option<chrono::DateTime<Utc>>,
    pub received_date: Option<chrono::DateTime<Utc>>,
    pub subtotal: i64,
    pub discount_amount: i64,
    pub total_amount: i64,
    pub payment_status: PaymentStatus,
    pub payment_due_date: Option<chrono::DateTime<Utc>>,
    pub paid_amount: i64,
    pub returned_amount: i64,
    pub notes: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub items: Vec<PurchaseOrderItem>,
    pub payments: Vec<PurchasePayment>,
}

// ============================================================================
// Create / update structs
// ============================================================================

#[derive(Debug, Clone)]
pub struct PurchaseOrderItemCreate {
    pub product_variant_id: i64,
    pub product_name: String,
    pub variant_name: Option<String>,
    pub barcode: Option<String>,
    pub quantity: i64,
    pub unit_cost: i64,
    pub discount_amount: i64,
}

impl PurchaseOrderItemCreate {
    pub fn total_cost(&self) -> i64 {
        (self.unit_cost * self.quantity) - self.discount_amount
    }
}

#[derive(Debug, Clone)]
pub struct PurchaseOrderCreate {
    pub branch_id: i64,
    pub supplier_id: Option<i64>,
    pub number: String,
    pub reference_number: Option<String>,
    pub order_date: Option<String>,
    pub expected_date: Option<String>,
    pub payment_due_date: Option<String>,
    pub discount_amount: i64,
    pub notes: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default)]
pub struct PurchaseOrderUpdate {
    pub supplier_id: Option<i64>,
    pub reference_number: Update<String>,
    pub status: Option<PurchaseOrderStatus>,
    pub order_date: Update<String>,
    pub expected_date: Update<String>,
    pub received_date: Update<String>,
    pub subtotal: Option<i64>,
    pub discount_amount: Option<i64>,
    pub total_amount: Option<i64>,
    pub payment_status: Option<PaymentStatus>,
    pub payment_due_date: Update<String>,
    pub paid_amount: Option<i64>,
    pub returned_amount: Option<i64>,
    pub notes: Update<String>,
    pub metadata: Update<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct PurchasePaymentCreate {
    pub amount: i64,
    pub channel: PurchasePaymentChannel,
    pub paid_at: String,
    pub reference: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct PurchaseOrderItemUpdate {
    pub quantity: Option<i64>,
    pub unit_cost: Option<i64>,
    pub discount_amount: Option<i64>,
    pub product_name: Option<String>,
    pub variant_name: Update<String>,
    pub barcode: Update<String>,
    pub metadata: Update<serde_json::Value>,
}

#[derive(Debug, Clone, Default)]
pub struct PurchasePaymentUpdate {
    pub amount: Option<i64>,
    pub channel: Option<PurchasePaymentChannel>,
    pub paid_at: Option<String>,
    pub reference: Update<String>,
    pub notes: Update<String>,
}

// ============================================================================
// Query / pagination
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct PurchaseOrderFilter {
    pub supplier_id: Option<i64>,
    pub number: Option<String>,
    pub status: Option<PurchaseOrderStatus>,
    pub reference_number: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PurchaseOrderSortField {
    #[default]
    CreatedAt,
    OrderDate,
    PaymentDueDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrderCursor {
    pub field_value: String,
    pub id: i64,
}

#[derive(Debug, Clone)]
pub struct PurchaseOrderQuery {
    pub filter: PurchaseOrderFilter,
    pub sort_field: PurchaseOrderSortField,
    pub sort_direction: SortDirection,
    pub cursor: Option<PurchaseOrderCursor>,
    pub limit: u64,
}

#[derive(Debug, Clone)]
pub struct PurchaseOrderPage {
    pub items: Vec<PurchaseOrder>,
    pub next_cursor: Option<PurchaseOrderCursor>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    // =========================================================================
    // PurchaseOrderStatus
    // =========================================================================

    #[test]
    fn test_purchase_order_status_as_str() {
        assert_eq!(PurchaseOrderStatus::Draft.as_str(), "draft");
        assert_eq!(PurchaseOrderStatus::Ordered.as_str(), "ordered");
        assert_eq!(PurchaseOrderStatus::Received.as_str(), "received");
        assert_eq!(PurchaseOrderStatus::Cancelled.as_str(), "cancelled");
    }

    #[test]
    fn test_purchase_order_status_display() {
        assert_eq!(PurchaseOrderStatus::Draft.to_string(), "draft");
        assert_eq!(PurchaseOrderStatus::Ordered.to_string(), "ordered");
        assert_eq!(PurchaseOrderStatus::Received.to_string(), "received");
        assert_eq!(PurchaseOrderStatus::Cancelled.to_string(), "cancelled");
    }

    #[test]
    fn test_purchase_order_status_from_str_valid() {
        assert_eq!(
            PurchaseOrderStatus::from_str("draft").unwrap(),
            PurchaseOrderStatus::Draft
        );
        assert_eq!(
            PurchaseOrderStatus::from_str("ordered").unwrap(),
            PurchaseOrderStatus::Ordered
        );
        assert_eq!(
            PurchaseOrderStatus::from_str("received").unwrap(),
            PurchaseOrderStatus::Received
        );
        assert_eq!(
            PurchaseOrderStatus::from_str("cancelled").unwrap(),
            PurchaseOrderStatus::Cancelled
        );
    }

    #[test]
    fn test_purchase_order_status_from_str_invalid() {
        let err = PurchaseOrderStatus::from_str("unknown").unwrap_err();
        assert!(matches!(err, crate::domain::Error::ValidationError(_)));
        let msg = err.to_string();
        assert!(msg.contains("unknown"));
        assert!(msg.contains("draft"));
    }

    #[test]
    fn test_purchase_order_status_default() {
        assert_eq!(PurchaseOrderStatus::default(), PurchaseOrderStatus::Draft);
    }

    #[test]
    fn test_purchase_order_status_serde_roundtrip() {
        let statuses = [
            PurchaseOrderStatus::Draft,
            PurchaseOrderStatus::Ordered,
            PurchaseOrderStatus::Received,
            PurchaseOrderStatus::Cancelled,
        ];
        for s in statuses {
            let json = serde_json::to_string(&s).unwrap();
            let back: PurchaseOrderStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
        }
    }

    // =========================================================================
    // PaymentStatus
    // =========================================================================

    #[test]
    fn test_payment_status_as_str() {
        assert_eq!(PaymentStatus::Unpaid.as_str(), "unpaid");
        assert_eq!(PaymentStatus::Partial.as_str(), "partial");
        assert_eq!(PaymentStatus::Paid.as_str(), "paid");
    }

    #[test]
    fn test_payment_status_display() {
        assert_eq!(PaymentStatus::Unpaid.to_string(), "unpaid");
        assert_eq!(PaymentStatus::Partial.to_string(), "partial");
        assert_eq!(PaymentStatus::Paid.to_string(), "paid");
    }

    #[test]
    fn test_payment_status_from_str_valid() {
        assert_eq!(
            PaymentStatus::from_str("unpaid").unwrap(),
            PaymentStatus::Unpaid
        );
        assert_eq!(
            PaymentStatus::from_str("partial").unwrap(),
            PaymentStatus::Partial
        );
        assert_eq!(
            PaymentStatus::from_str("paid").unwrap(),
            PaymentStatus::Paid
        );
    }

    #[test]
    fn test_payment_status_from_str_invalid() {
        let err = PaymentStatus::from_str("settled").unwrap_err();
        assert!(matches!(err, crate::domain::Error::ValidationError(_)));
        let msg = err.to_string();
        assert!(msg.contains("settled"));
        assert!(msg.contains("unpaid"));
    }

    #[test]
    fn test_payment_status_default() {
        assert_eq!(PaymentStatus::default(), PaymentStatus::Unpaid);
    }

    #[test]
    fn test_payment_status_serde_roundtrip() {
        let statuses = [
            PaymentStatus::Unpaid,
            PaymentStatus::Partial,
            PaymentStatus::Paid,
        ];
        for s in statuses {
            let json = serde_json::to_string(&s).unwrap();
            let back: PaymentStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
        }
    }

    // =========================================================================
    // PurchasePaymentChannel
    // =========================================================================

    #[test]
    fn test_purchase_payment_channel_as_str() {
        assert_eq!(PurchasePaymentChannel::Cash.as_str(), "cash");
        assert_eq!(
            PurchasePaymentChannel::BankTransfer.as_str(),
            "bank_transfer"
        );
        assert_eq!(PurchasePaymentChannel::Card.as_str(), "card");
        assert_eq!(PurchasePaymentChannel::Other.as_str(), "other");
    }

    #[test]
    fn test_purchase_payment_channel_display() {
        assert_eq!(PurchasePaymentChannel::Cash.to_string(), "cash");
        assert_eq!(
            PurchasePaymentChannel::BankTransfer.to_string(),
            "bank_transfer"
        );
        assert_eq!(PurchasePaymentChannel::Card.to_string(), "card");
        assert_eq!(PurchasePaymentChannel::Other.to_string(), "other");
    }

    #[test]
    fn test_purchase_payment_channel_from_str_valid() {
        assert_eq!(
            PurchasePaymentChannel::from_str("cash").unwrap(),
            PurchasePaymentChannel::Cash
        );
        assert_eq!(
            PurchasePaymentChannel::from_str("bank_transfer").unwrap(),
            PurchasePaymentChannel::BankTransfer
        );
        assert_eq!(
            PurchasePaymentChannel::from_str("card").unwrap(),
            PurchasePaymentChannel::Card
        );
        assert_eq!(
            PurchasePaymentChannel::from_str("other").unwrap(),
            PurchasePaymentChannel::Other
        );
    }

    #[test]
    fn test_purchase_payment_channel_from_str_invalid() {
        let err = PurchasePaymentChannel::from_str("crypto").unwrap_err();
        assert!(matches!(err, crate::domain::Error::ValidationError(_)));
        let msg = err.to_string();
        assert!(msg.contains("crypto"));
        assert!(msg.contains("cash"));
    }

    #[test]
    fn test_purchase_payment_channel_default() {
        assert_eq!(
            PurchasePaymentChannel::default(),
            PurchasePaymentChannel::Cash
        );
    }

    #[test]
    fn test_purchase_payment_channel_serde_roundtrip() {
        let channels = [
            PurchasePaymentChannel::Cash,
            PurchasePaymentChannel::BankTransfer,
            PurchasePaymentChannel::Card,
            PurchasePaymentChannel::Other,
        ];
        for c in channels {
            let json = serde_json::to_string(&c).unwrap();
            let back: PurchasePaymentChannel = serde_json::from_str(&json).unwrap();
            assert_eq!(c, back);
        }
    }

    // =========================================================================
    // PurchaseOrderItemCreate::total_cost
    // =========================================================================

    #[test]
    fn test_purchase_order_item_create_total_cost_no_discount() {
        let item = PurchaseOrderItemCreate {
            product_variant_id: 1,
            product_name: "Widget".to_string(),
            variant_name: None,
            barcode: None,
            quantity: 5,
            unit_cost: 1000,
            discount_amount: 0,
        };
        assert_eq!(item.total_cost(), 5000);
    }

    #[test]
    fn test_purchase_order_item_create_total_cost_with_discount() {
        let item = PurchaseOrderItemCreate {
            product_variant_id: 1,
            product_name: "Widget".to_string(),
            variant_name: None,
            barcode: None,
            quantity: 3,
            unit_cost: 2000,
            discount_amount: 500,
        };
        // 3 * 2000 - 500 = 5500
        assert_eq!(item.total_cost(), 5500);
    }

    #[test]
    fn test_purchase_order_item_create_total_cost_full_discount() {
        let item = PurchaseOrderItemCreate {
            product_variant_id: 1,
            product_name: "Free item".to_string(),
            variant_name: None,
            barcode: None,
            quantity: 2,
            unit_cost: 100,
            discount_amount: 200,
        };
        assert_eq!(item.total_cost(), 0);
    }

    #[test]
    fn test_purchase_order_item_create_total_cost_single_unit() {
        let item = PurchaseOrderItemCreate {
            product_variant_id: 1,
            product_name: "Single".to_string(),
            variant_name: Some("Red".to_string()),
            barcode: Some("1234567890".to_string()),
            quantity: 1,
            unit_cost: 9999,
            discount_amount: 99,
        };
        assert_eq!(item.total_cost(), 9900);
    }
}
