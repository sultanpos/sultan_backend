use chrono::Utc;
use serde_json::Value;

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
    pub has_variant: bool,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct ProductVariant {
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub deleted_at: Option<chrono::DateTime<Utc>>,
    pub is_deleted: bool,
    pub product: Product,
    pub barcode: Option<String>,
    pub name: Option<String>,
    pub metadata: Option<Value>,
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
    pub has_variant: bool,
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
    pub has_variant: Option<bool>,
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
