use chrono::Utc;
use serde_json::Value;

use super::Update;

#[derive(Debug, Clone)]
pub struct SellPrice {
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub deleted_at: Option<chrono::DateTime<Utc>>,
    pub is_deleted: bool,
    pub branch_id: Option<i64>,
    pub product_variant_id: i64,
    pub uom_id: i64,
    pub quantity: i64,
    pub price: i64,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct SellPriceCreate {
    pub branch_id: Option<i64>,
    pub product_variant_id: i64,
    pub uom_id: i64,
    pub quantity: i64,
    pub price: i64,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct SellPriceUpdate {
    pub uom_id: Option<i64>,
    pub quantity: Option<i64>,
    pub price: Option<i64>,
    pub metadata: Update<Value>,
}

#[derive(Debug, Clone)]
pub struct SellDiscount {
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub deleted_at: Option<chrono::DateTime<Utc>>,
    pub is_deleted: bool,
    pub price_id: i64,
    pub quantity: i64,
    pub discount_formula: Option<String>,
    pub calculated_price: i64,
    pub customer_level: Option<i64>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct SellDiscountCreate {
    pub price_id: i64,
    pub quantity: i64,
    pub discount_formula: String,
    pub customer_level: Option<i64>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct SellDiscountUpdate {
    pub quantity: i64,
    pub discount_formula: Option<String>,
    pub customer_level: Update<i64>,
    pub metadata: Update<Value>,
}
