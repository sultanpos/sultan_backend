use chrono::Utc;
use serde_json::Value;

use crate::domain::model::{
    category::Category,
    sell_price::{SellDiscountCreate, SellPrice, SellPriceCreate},
    stock::StockCreate,
};

use super::Update;

#[derive(Debug, Clone)]
pub struct UnitOfMeasure {
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub deleted_at: Option<chrono::DateTime<Utc>>,
    pub is_deleted: bool,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UnitOfMeasureCreate {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct UnitOfMeasureUpdate {
    pub name: Option<String>,
    pub description: Update<String>,
}

#[derive(Debug, Clone)]
pub struct Product {
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub deleted_at: Option<chrono::DateTime<Utc>>,
    pub is_deleted: bool,
    pub name: String,
    pub description: Option<String>,
    pub product_type: String,
    pub main_image: Option<String>,
    pub sellable: bool,
    pub buyable: bool,
    pub editable_price: bool,
    pub variant_count: i32,
    pub metadata: Option<Value>,
    pub categories: Vec<Category>,
    pub variants: Vec<ProductVariant>,
}

#[derive(Debug, Clone)]
pub struct ProductVariant {
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub deleted_at: Option<chrono::DateTime<Utc>>,
    pub is_deleted: bool,
    pub barcode: Option<String>,
    pub name: Option<String>,
    pub metadata: Option<Value>,
    pub sell_prices: Vec<SellPrice>,
}

#[derive(Debug, Clone)]
pub struct ProductVariantCreate {
    pub product_id: i64,
    pub barcode: Option<String>,
    pub name: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct ProductCreate {
    pub name: String,
    pub description: Option<String>,
    pub product_type: String,
    pub main_image: Option<String>,
    pub sellable: bool,
    pub buyable: bool,
    pub editable_price: bool,
    pub variant_count: i32,
    pub metadata: Option<Value>,
    pub category_ids: Vec<i64>,
}

#[derive(Debug, Clone)]
pub struct ProductUpdate {
    pub name: Option<String>,
    pub description: Update<String>,
    pub product_type: Option<String>,
    pub main_image: Update<String>,
    pub sellable: Option<bool>,
    pub buyable: Option<bool>,
    pub editable_price: Option<bool>,
    pub metadata: Update<Value>,
    pub category_ids: Option<Vec<i64>>,
}

#[derive(Debug, Clone)]
pub struct ProductVariantUpdate {
    pub barcode: Update<String>,
    pub name: Update<String>,
    pub metadata: Update<Value>,
}

#[derive(Debug, Clone)]
pub struct ProductCategory {
    pub product_id: i64,
    pub category_id: i64,
}

#[derive(Debug, Clone)]
pub struct ProductFilter {
    pub name: Option<String>,
    pub product_type: Option<String>,
    pub category_id: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct SellPriceFullCreate {
    pub sell_price: SellPriceCreate,
    pub discounts: Vec<SellDiscountCreate>,
}

#[derive(Debug, Clone)]
pub struct ProductVariantFullCreate {
    pub variant: ProductVariantCreate,
    pub sell_prices: Vec<SellPriceFullCreate>,
    pub stocks: Vec<StockCreate>,
}

#[derive(Debug, Clone)]
pub struct ProductFullCreate {
    pub product: ProductCreate,
    pub variants: Vec<ProductVariantFullCreate>,
    pub categories: Vec<i64>,
}
