use chrono::Utc;
use serde_json::Value;

use crate::domain::model::Update;

#[derive(Debug, Clone)]
pub struct Stock {
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub branch_id: i64,
    pub product_variant_id: i64,
    pub quantity: i64,
    pub min_stock: Option<i64>,
    pub max_stock: Option<i64>,
    pub last_buy_price: Option<i64>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct StockCreate {
    pub branch_id: i64,
    pub product_variant_id: i64,
    pub quantity: i64,
    pub min_stock: Option<i64>,
    pub max_stock: Option<i64>,
    pub last_buy_price: Option<i64>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct StockUpdate {
    pub min_stock: Update<i64>,
    pub max_stock: Update<i64>,
    pub last_buy_price: Update<i64>,
    pub metadata: Update<Value>,
}
