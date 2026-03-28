use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::model::{
    category::Category,
    sell_price::{SellDiscountCreate, SellPrice, SellPriceCreate},
    stock::StockCreate,
};

use super::Update;

/// Sort fields available for product listing.
///
/// The cursor-based pagination always appends `id` as a tiebreaker,
/// so the effective ordering is `(field, id)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductSortField {
    Name,
    CreatedAt,
    UpdatedAt,
}

/// Direction for ordering results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortDirection {
    Asc,
    Desc,
}

/// Cursor for keyset (cursor-based) pagination over products.
///
/// Contains the last-seen values of the sort field and the tiebreaker `id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductCursor {
    /// Value of the primary sort field from the last item of the previous page.
    pub field_value: String,
    /// ID of the last item of the previous page (tiebreaker).
    pub id: i64,
}

/// Options for querying a list of products with cursor-based pagination.
#[derive(Debug, Clone)]
pub struct ProductQuery {
    pub filter: ProductFilter,
    pub sort_field: ProductSortField,
    pub sort_direction: SortDirection,
    pub cursor: Option<ProductCursor>,
    pub limit: u64,
}

/// A page of results with an optional cursor pointing to the next page.
#[derive(Debug, Clone)]
pub struct CursorPage<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<ProductCursor>,
}

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
    pub metadata: Option<Value>,
    pub category_ids: Vec<i64>,
}

#[derive(Debug, Clone, Default)]
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
