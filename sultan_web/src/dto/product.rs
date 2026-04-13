use super::category::CategoryChildResponse;
use super::{
    default_page_size, i64_to_string, option_i64_to_string, option_string_to_i64,
    option_vec_string_to_i64, string_to_i64, vec_string_to_i64,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sultan_core::domain::model::Update;
use sultan_core::domain::model::product::{Product, ProductType};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

// ===== Product DTOs =====

/// Request to create a new product
#[derive(Debug, Default, Deserialize, Validate, ToSchema)]
#[serde(default)]
pub struct ProductCreateRequest {
    /// Product name
    #[validate(length(
        min = 1,
        max = 256,
        message = "Name must be between 1 and 256 characters"
    ))]
    #[schema(example = "Laptop ASUS ROG")]
    pub name: String,

    /// Product description (optional)
    #[validate(length(max = 1000, message = "Description must not exceed 1000 characters"))]
    #[schema(example = "High-performance gaming laptop")]
    pub description: Option<String>,

    /// Product type: "product", "service", or "bundle"
    #[schema(example = "product", value_type = String)]
    pub product_type: ProductType,

    /// Main product image URL (optional)
    #[schema(example = "https://example.com/images/laptop.jpg")]
    pub main_image: Option<String>,

    /// Whether the product can be sold
    #[schema(example = true)]
    pub sellable: bool,

    /// Whether the product can be purchased
    #[schema(example = true)]
    pub buyable: bool,

    /// Whether the price can be edited during sale
    #[schema(example = false)]
    pub editable_price: bool,

    /// Additional metadata (optional)
    #[schema(example = json!({"color": "black", "warranty": "2 years"}))]
    pub metadata: Option<Value>,

    /// Category IDs this product belongs to
    #[schema(example = json!(["1234567890", "9876543210"]))]
    #[serde(default, deserialize_with = "vec_string_to_i64")]
    pub category_ids: Vec<i64>,
}

impl ProductCreateRequest {
    pub fn to_domain(&self) -> sultan_core::domain::model::product::ProductCreate {
        sultan_core::domain::model::product::ProductCreate {
            name: self.name.clone(),
            description: self.description.clone(),
            product_type: self.product_type,
            main_image: self.main_image.clone(),
            sellable: self.sellable,
            buyable: self.buyable,
            editable_price: self.editable_price,
            metadata: self.metadata.clone(),
            category_ids: self.category_ids.clone(),
        }
    }
}

/// Request to update an existing product (all fields optional)
///
/// Fields absent from the JSON are left unchanged.
/// Nullable fields (`description`, `main_image`, `metadata`) can be cleared by sending `null`.
#[derive(Debug, Default, Deserialize, Validate, ToSchema)]
#[serde(default)]
pub struct ProductUpdateRequest {
    /// Product name
    #[validate(length(
        min = 1,
        max = 256,
        message = "Name must be between 1 and 256 characters"
    ))]
    #[schema(example = "Laptop ASUS ROG")]
    pub name: Option<String>,

    /// Omit to leave unchanged, `null` to clear
    #[schema(value_type = Option<String>, example = "High-performance gaming laptop")]
    pub description: Update<String>,

    /// Product type: "product", "service", or "bundle"
    #[schema(example = "product", value_type = Option<String>)]
    pub product_type: Option<ProductType>,

    /// Omit to leave unchanged, `null` to clear
    #[schema(value_type = Option<String>, example = "https://example.com/images/laptop.jpg")]
    pub main_image: Update<String>,

    #[schema(example = true)]
    pub sellable: Option<bool>,

    #[schema(example = true)]
    pub buyable: Option<bool>,

    #[schema(example = false)]
    pub editable_price: Option<bool>,

    /// Omit to leave unchanged, `null` to clear
    #[schema(value_type = Option<Value>, example = json!({"color": "black"}))]
    pub metadata: Update<Value>,

    /// If provided, replaces all existing category associations
    #[schema(example = json!(["1234567890", "9876543210"]))]
    #[serde(default, deserialize_with = "option_vec_string_to_i64")]
    pub category_ids: Option<Vec<i64>>,
}

impl ProductUpdateRequest {
    pub fn to_domain(&self) -> sultan_core::domain::model::product::ProductUpdate {
        sultan_core::domain::model::product::ProductUpdate {
            name: self.name.clone(),
            description: self.description.clone(),
            product_type: self.product_type,
            main_image: self.main_image.clone(),
            sellable: self.sellable,
            buyable: self.buyable,
            editable_price: self.editable_price,
            metadata: self.metadata.clone(),
            category_ids: self.category_ids.clone(),
        }
    }
}

/// Response after creating a product
#[derive(Debug, Serialize, ToSchema)]
pub struct ProductCreateResponse {
    /// Product ID
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub id: i64,
}

/// Product response
#[derive(Debug, Serialize, ToSchema)]
pub struct ProductResponse {
    /// Product ID
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub id: i64,

    /// Creation timestamp
    pub created_at: chrono::DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: chrono::DateTime<Utc>,

    /// Product name
    #[schema(example = "Laptop ASUS ROG")]
    pub name: String,

    /// Product description
    #[schema(example = "High-performance gaming laptop")]
    pub description: Option<String>,

    /// Product type
    #[schema(example = "product", value_type = String)]
    pub product_type: ProductType,

    /// Main product image URL
    #[schema(example = "https://example.com/images/laptop.jpg")]
    pub main_image: Option<String>,

    /// Whether the product can be sold
    #[schema(example = true)]
    pub sellable: bool,

    /// Whether the product can be purchased
    #[schema(example = true)]
    pub buyable: bool,

    #[schema(example = 1)]
    pub variant_count: i32,

    /// Whether the price can be edited during sale
    #[schema(example = false)]
    pub editable_price: bool,

    /// Additional metadata
    pub metadata: Option<Value>,

    /// Categories this product belongs to
    pub categories: Vec<CategoryChildResponse>,

    /// Product variants
    pub variants: Vec<ProductVariantResponse>,
}

impl From<Product> for ProductResponse {
    fn from(product: Product) -> Self {
        Self {
            id: product.id,
            created_at: product.created_at,
            updated_at: product.updated_at,
            name: product.name,
            description: product.description,
            product_type: product.product_type,
            main_image: product.main_image,
            sellable: product.sellable,
            buyable: product.buyable,
            editable_price: product.editable_price,
            variant_count: product.variant_count,
            metadata: product.metadata,
            categories: product
                .categories
                .into_iter()
                .map(|c| CategoryChildResponse {
                    id: c.id,
                    name: c.name,
                    description: c.description,
                })
                .collect(),
            variants: product
                .variants
                .into_iter()
                .map(ProductVariantResponse::from)
                .collect(),
        }
    }
}

/// Query parameters for filtering and paginating products (cursor-based).
///
/// Products are always ordered by `(sort_field, id)` to guarantee stable ordering.
/// To fetch the next page, pass the `cursor` value returned in the previous response.
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct ProductQueryParams {
    /// Product name filter (partial match)
    #[schema(example = "Laptop")]
    pub name: Option<String>,

    /// Product type filter
    #[schema(example = "product")]
    pub product_type: Option<String>,

    /// Category ID filter
    #[schema(example = "1234567890")]
    #[serde(default, deserialize_with = "option_string_to_i64")]
    pub category_id: Option<i64>,

    /// Sort field: "name", "created_at", or "updated_at" (default: "created_at")
    #[serde(default = "default_sort_field")]
    #[schema(example = "created_at")]
    pub sort_field: String,

    /// Sort direction: "asc" or "desc" (default: "desc")
    #[serde(default = "default_sort_direction")]
    #[schema(example = "desc")]
    pub sort_direction: String,

    /// Opaque cursor from the previous page's `next_cursor` (omit for the first page)
    #[schema(example = "eyJmaWVsZF92YWx1ZSI6IjIwMjUtMDEtMDEiLCJpZCI6MTIzfQ==")]
    pub cursor: Option<String>,

    /// Maximum number of items per page (default: 20, max: 100)
    #[serde(default = "default_page_size")]
    #[schema(example = 20)]
    pub limit: u32,
}

fn default_sort_field() -> String {
    "created_at".to_string()
}

fn default_sort_direction() -> String {
    "desc".to_string()
}

fn parse_sort_field(
    s: &str,
) -> Result<sultan_core::domain::model::product::ProductSortField, sultan_core::domain::Error> {
    use sultan_core::domain::model::product::ProductSortField;
    match s {
        "name" => Ok(ProductSortField::Name),
        "created_at" => Ok(ProductSortField::CreatedAt),
        "updated_at" => Ok(ProductSortField::UpdatedAt),
        other => Err(sultan_core::domain::Error::ValidationError(format!(
            "Invalid sort_field '{}'. Must be one of: name, created_at, updated_at",
            other
        ))),
    }
}

impl ProductQueryParams {
    /// Convert query params into the domain `ProductQuery`.
    pub fn to_query(
        &self,
    ) -> Result<sultan_core::domain::model::product::ProductQuery, sultan_core::domain::Error> {
        use sultan_core::domain::model::product::{ProductCursor, ProductFilter, ProductQuery};

        let sort_field = parse_sort_field(&self.sort_field)?;
        let sort_direction = super::parse_sort_direction(&self.sort_direction)?;
        let cursor = self
            .cursor
            .as_deref()
            .map(super::decode_cursor::<ProductCursor>)
            .transpose()?;
        let limit = self.limit.clamp(1, 100) as u64;

        Ok(ProductQuery {
            filter: ProductFilter {
                name: self.name.clone(),
                product_type: self
                    .product_type
                    .as_deref()
                    .map(|pt| pt.parse())
                    .transpose()?,
                category_id: self.category_id,
            },
            sort_field,
            sort_direction,
            cursor,
            limit,
        })
    }
}

/// Response for a paginated list of products (cursor-based).
#[derive(Debug, Serialize, ToSchema)]
pub struct ProductListResponse {
    /// The items in this page
    pub items: Vec<ProductResponse>,

    /// Opaque cursor to fetch the next page. `null` when there are no more pages.
    #[schema(example = "eyJmaWVsZF92YWx1ZSI6IjIwMjUtMDEtMDEiLCJpZCI6MTIzfQ==")]
    pub next_cursor: Option<String>,
}

impl ProductListResponse {
    pub fn from_cursor_page(
        page: sultan_core::domain::model::product::CursorPage<
            sultan_core::domain::model::product::Product,
        >,
    ) -> Self {
        use base64::Engine;

        let next_cursor = page.next_cursor.map(|c| {
            let json = serde_json::to_vec(&c).expect("cursor is always serializable");
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json)
        });

        Self {
            items: page.items.into_iter().map(ProductResponse::from).collect(),
            next_cursor,
        }
    }
}

// ===== Product Variant DTOs =====

/// Request to create a new product variant
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct ProductVariantCreateRequest {
    /// Product ID this variant belongs to
    #[schema(example = "1234567890")]
    #[serde(deserialize_with = "super::string_to_i64")]
    pub product_id: i64,

    /// Variant barcode (optional)
    #[schema(example = "8901234567890")]
    pub barcode: Option<String>,

    /// Variant name (optional, e.g., "Red - Large")
    #[schema(example = "Black - 16GB RAM")]
    pub name: Option<String>,

    /// Additional metadata (optional)
    #[schema(example = json!({"ram": "16GB", "storage": "512GB SSD"}))]
    pub metadata: Option<Value>,
}

impl From<ProductVariantCreateRequest>
    for sultan_core::domain::model::product::ProductVariantCreate
{
    fn from(req: ProductVariantCreateRequest) -> Self {
        Self {
            product_id: req.product_id,
            barcode: req.barcode,
            name: req.name,
            metadata: req.metadata,
        }
    }
}

/// Response after creating a product variant
#[derive(Debug, Serialize, ToSchema)]
pub struct ProductVariantCreateResponse {
    /// Variant ID
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub id: i64,
}

/// Request to update a product variant
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ProductVariantUpdateRequest {
    /// Omit to leave unchanged, `null` to clear
    #[schema(value_type = Option<String>, example = "8901234567890")]
    pub barcode: Update<String>,

    /// Omit to leave unchanged, `null` to clear
    #[schema(value_type = Option<String>, example = "Red - Large")]
    pub name: Update<String>,

    /// Omit to leave unchanged, `null` to clear
    #[schema(value_type = Option<Value>, example = json!({"color": "red"}))]
    pub metadata: Update<Value>,
}

impl ProductVariantUpdateRequest {
    pub fn to_domain(&self) -> sultan_core::domain::model::product::ProductVariantUpdate {
        sultan_core::domain::model::product::ProductVariantUpdate {
            barcode: self.barcode.clone(),
            name: self.name.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

/// Product variant response
#[derive(Debug, Serialize, ToSchema)]
pub struct ProductVariantResponse {
    /// Variant ID
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub id: i64,

    /// Creation timestamp
    pub created_at: chrono::DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: chrono::DateTime<Utc>,

    /// Variant barcode
    #[schema(example = "8901234567890")]
    pub barcode: Option<String>,

    /// Variant name
    #[schema(example = "Black - 16GB RAM")]
    pub name: Option<String>,

    /// Additional metadata
    pub metadata: Option<Value>,

    /// Sell prices for this variant
    pub sell_prices: Vec<SellPriceResponse>,
}

impl From<sultan_core::domain::model::product::ProductVariant> for ProductVariantResponse {
    fn from(variant: sultan_core::domain::model::product::ProductVariant) -> Self {
        Self {
            id: variant.id,
            created_at: variant.created_at,
            updated_at: variant.updated_at,
            barcode: variant.barcode,
            name: variant.name,
            metadata: variant.metadata,
            sell_prices: variant
                .sell_prices
                .into_iter()
                .map(SellPriceResponse::from)
                .collect(),
        }
    }
}

// ===== Response DTOs for nested resources =====

/// Sell discount response
#[derive(Debug, Serialize, ToSchema)]
pub struct SellDiscountResponse {
    /// Discount ID
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub id: i64,

    /// Creation timestamp
    pub created_at: chrono::DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: chrono::DateTime<Utc>,

    /// Sell price ID this discount belongs to
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub sell_price_id: i64,

    /// Minimum quantity for this discount
    pub quantity: i64,

    /// Discount formula (e.g., "price * 0.9" for 10% off)
    #[schema(example = "price * 0.9")]
    pub discount_formula: Option<String>,

    /// Calculated price after applying the discount
    pub calculated_price: i64,

    /// Customer level this discount applies to
    pub customer_level: Option<i64>,

    /// Additional metadata
    pub metadata: Option<Value>,
}

impl From<sultan_core::domain::model::sell_price::SellDiscount> for SellDiscountResponse {
    fn from(d: sultan_core::domain::model::sell_price::SellDiscount) -> Self {
        Self {
            id: d.id,
            created_at: d.created_at,
            updated_at: d.updated_at,
            sell_price_id: d.sell_price_id,
            quantity: d.quantity,
            discount_formula: d.discount_formula,
            calculated_price: d.calculated_price,
            customer_level: d.customer_level,
            metadata: d.metadata,
        }
    }
}

/// Sell price response
#[derive(Debug, Serialize, ToSchema)]
pub struct SellPriceResponse {
    /// Price ID
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub id: i64,

    /// Creation timestamp
    pub created_at: chrono::DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: chrono::DateTime<Utc>,

    /// Branch ID (null means all branches)
    #[schema(example = "1234567890", value_type = Option<String>)]
    #[serde(serialize_with = "option_i64_to_string")]
    pub branch_id: Option<i64>,

    /// Unit of measure ID
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub uom_id: i64,

    /// Quantity for this price point
    pub quantity: i64,

    /// Price in cents/smallest currency unit
    pub price: i64,

    /// Additional metadata
    pub metadata: Option<Value>,

    /// Discounts for this price
    pub discounts: Vec<SellDiscountResponse>,
}

impl From<sultan_core::domain::model::sell_price::SellPrice> for SellPriceResponse {
    fn from(sp: sultan_core::domain::model::sell_price::SellPrice) -> Self {
        Self {
            id: sp.id,
            created_at: sp.created_at,
            updated_at: sp.updated_at,
            branch_id: sp.branch_id,
            uom_id: sp.uom_id,
            quantity: sp.quantity,
            price: sp.price,
            metadata: sp.metadata,
            discounts: sp
                .discounts
                .into_iter()
                .map(SellDiscountResponse::from)
                .collect(),
        }
    }
}

// ===== Advanced Create DTOs (for creating product with all related data) =====

/// Request to create a sell discount
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct SellDiscountCreateRequest {
    /// Price ID this discount belongs to
    #[schema(example = "1234567890")]
    #[serde(deserialize_with = "string_to_i64")]
    pub price_id: i64,

    /// Minimum quantity for this discount
    #[schema(example = 10)]
    #[validate(range(min = 1, message = "Quantity must be greater than 0"))]
    pub quantity: i64,

    /// Discount formula (e.g., "price * 0.9" for 10% off)
    #[schema(example = "price * 0.9")]
    #[validate(length(min = 1, message = "Discount formula cannot be empty"))]
    pub discount_formula: String,

    /// Customer level this discount applies to (optional)
    #[schema(example = 1)]
    pub customer_level: Option<i64>,

    /// Additional metadata
    pub metadata: Option<Value>,
}

impl From<SellDiscountCreateRequest>
    for sultan_core::domain::model::sell_price::SellDiscountCreate
{
    fn from(req: SellDiscountCreateRequest) -> Self {
        Self {
            price_id: req.price_id,
            quantity: req.quantity,
            discount_formula: req.discount_formula,
            customer_level: req.customer_level,
            metadata: req.metadata,
        }
    }
}

/// Request to create a sell price
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct SellPriceCreateRequest {
    /// Branch ID (optional, null means all branches)
    #[schema(example = "1234567890")]
    #[serde(default, deserialize_with = "option_string_to_i64")]
    pub branch_id: Option<i64>,

    /// Product variant ID
    #[schema(example = "1234567890")]
    #[serde(deserialize_with = "string_to_i64")]
    pub product_variant_id: i64,

    /// Unit of measure ID
    #[schema(example = "1234567890")]
    #[serde(deserialize_with = "string_to_i64")]
    pub uom_id: i64,

    /// Quantity for this price point
    #[schema(example = 1)]
    #[validate(range(min = 1, message = "Quantity must be greater than 0"))]
    pub quantity: i64,

    /// Price in cents/smallest currency unit
    #[schema(example = 150000)]
    #[validate(range(min = 1, message = "Price must be greater than 0"))]
    pub price: i64,

    /// Additional metadata
    pub metadata: Option<Value>,
}

impl From<SellPriceCreateRequest> for sultan_core::domain::model::sell_price::SellPriceCreate {
    fn from(req: SellPriceCreateRequest) -> Self {
        Self {
            branch_id: req.branch_id,
            product_variant_id: req.product_variant_id,
            uom_id: req.uom_id,
            quantity: req.quantity,
            price: req.price,
            metadata: req.metadata,
        }
    }
}

/// Request to create a sell price with discounts
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SellPriceFullCreateRequest {
    /// Sell price details
    #[validate(nested)]
    pub sell_price: SellPriceCreateRequest,

    /// List of discounts for this price
    #[serde(default)]
    #[validate(nested)]
    pub discounts: Vec<SellDiscountCreateRequest>,
}

impl SellPriceFullCreateRequest {
    pub fn to_domain(&self) -> sultan_core::domain::model::product::SellPriceFullCreate {
        sultan_core::domain::model::product::SellPriceFullCreate {
            sell_price: self.sell_price.clone().into(),
            discounts: self.discounts.iter().cloned().map(|d| d.into()).collect(),
        }
    }
}

/// Response after creating a sell price
#[derive(Debug, Serialize, ToSchema)]
pub struct SellPriceCreateResponse {
    /// Sell price ID
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub id: i64,
}

/// Request to update a sell price
#[derive(Debug, Default, Deserialize, Validate, ToSchema)]
#[serde(default)]
pub struct SellPriceUpdateRequest {
    /// Unit of measure ID (omit to leave unchanged)
    #[schema(value_type = Option<String>, example = "1234567890")]
    #[serde(deserialize_with = "option_string_to_i64")]
    pub uom_id: Option<i64>,

    /// Quantity for this price point (omit to leave unchanged)
    #[schema(example = 1)]
    #[validate(range(min = 1, message = "Quantity must be greater than 0"))]
    pub quantity: Option<i64>,

    /// Price in cents/smallest currency unit (omit to leave unchanged)
    #[schema(example = 150000)]
    #[validate(range(min = 1, message = "Price must be greater than 0"))]
    pub price: Option<i64>,

    /// Omit to leave unchanged, `null` to clear
    #[schema(value_type = Option<Value>)]
    pub metadata: Update<Value>,
}

impl SellPriceUpdateRequest {
    pub fn to_domain(&self) -> sultan_core::domain::model::sell_price::SellPriceUpdate {
        sultan_core::domain::model::sell_price::SellPriceUpdate {
            uom_id: self.uom_id,
            quantity: self.quantity,
            price: self.price,
            metadata: self.metadata.clone(),
        }
    }
}

/// Response after creating a sell discount
#[derive(Debug, Serialize, ToSchema)]
pub struct SellDiscountCreateResponse {
    /// Sell discount ID
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub id: i64,
}

/// Request to update a sell discount
#[derive(Debug, Default, Deserialize, Validate, ToSchema)]
#[serde(default)]
pub struct SellDiscountUpdateRequest {
    /// Minimum quantity for this discount (omit to leave unchanged)
    #[schema(example = 10)]
    #[validate(range(min = 1, message = "Quantity must be greater than 0"))]
    pub quantity: Option<i64>,

    /// Discount formula (omit to leave unchanged)
    #[schema(example = "price * 0.9")]
    #[validate(length(min = 1, message = "Discount formula cannot be empty"))]
    pub discount_formula: Option<String>,

    /// Customer level this discount applies to. Omit to leave unchanged, `null` to clear
    #[schema(value_type = Option<i64>, example = 1)]
    pub customer_level: Update<i64>,

    /// Additional metadata. Omit to leave unchanged, `null` to clear
    #[schema(value_type = Option<Value>)]
    pub metadata: Update<Value>,
}

impl SellDiscountUpdateRequest {
    pub fn to_domain(&self) -> sultan_core::domain::model::sell_price::SellDiscountUpdate {
        sultan_core::domain::model::sell_price::SellDiscountUpdate {
            quantity: self.quantity,
            discount_formula: self.discount_formula.clone(),
            customer_level: self.customer_level.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

/// Request to create stock for a product variant
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct StockCreateRequest {
    /// Branch ID where this stock is located
    #[schema(example = "1234567890")]
    #[serde(deserialize_with = "string_to_i64")]
    pub branch_id: i64,

    /// Product variant ID
    #[schema(example = "1234567890")]
    #[serde(deserialize_with = "string_to_i64")]
    pub product_variant_id: i64,

    /// Current quantity
    #[schema(example = 100)]
    pub quantity: i64,

    /// Minimum stock level (optional)
    #[schema(example = 10)]
    pub min_stock: Option<i64>,

    /// Maximum stock level (optional)
    #[schema(example = 1000)]
    pub max_stock: Option<i64>,

    /// Last purchase price in cents (optional)
    #[schema(example = 120000)]
    pub last_buy_price: Option<i64>,

    /// Additional metadata
    pub metadata: Option<Value>,
}

impl From<StockCreateRequest> for sultan_core::domain::model::stock::StockCreate {
    fn from(req: StockCreateRequest) -> Self {
        Self {
            branch_id: req.branch_id,
            product_variant_id: req.product_variant_id,
            quantity: req.quantity,
            min_stock: req.min_stock,
            max_stock: req.max_stock,
            last_buy_price: req.last_buy_price,
            metadata: req.metadata,
        }
    }
}

/// Request to create a product variant with sell prices and stocks
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ProductVariantFullCreateRequest {
    /// Variant details
    pub variant: ProductVariantCreateRequest,

    /// List of sell prices with discounts
    #[serde(default)]
    pub sell_prices: Vec<SellPriceFullCreateRequest>,

    /// List of stock records
    #[serde(default)]
    pub stocks: Vec<StockCreateRequest>,
}

impl ProductVariantFullCreateRequest {
    pub fn to_domain(&self) -> sultan_core::domain::model::product::ProductVariantFullCreate {
        sultan_core::domain::model::product::ProductVariantFullCreate {
            variant: self.variant.clone().into(),
            sell_prices: self.sell_prices.iter().map(|sp| sp.to_domain()).collect(),
            stocks: self.stocks.iter().cloned().map(|s| s.into()).collect(),
        }
    }
}

/// Request to create a complete product with variants, prices, and stocks
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ProductFullCreateRequest {
    /// Product details
    #[validate(nested)]
    pub product: ProductCreateRequest,

    /// List of product variants with prices and stocks
    #[serde(default)]
    pub variants: Vec<ProductVariantFullCreateRequest>,

    /// Category IDs (can also be specified in product.category_ids)
    #[schema(example = json!(["1234567890", "9876543210"]))]
    #[serde(default, deserialize_with = "vec_string_to_i64")]
    pub categories: Vec<i64>,
}

impl ProductFullCreateRequest {
    pub fn to_domain(&self) -> sultan_core::domain::model::product::ProductFullCreate {
        sultan_core::domain::model::product::ProductFullCreate {
            product: self.product.to_domain(),
            variants: self.variants.iter().map(|v| v.to_domain()).collect(),
            categories: self.categories.clone(),
        }
    }
}

// ===== Variant Search DTOs =====

/// Query parameters for searching product variants with cursor-based pagination.
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct VariantSearchQueryParams {
    /// Filter by product name (partial match)
    #[schema(example = "Macbook Pro M5")]
    pub name: Option<String>,

    /// Filter by product type
    #[schema(example = "product")]
    pub product_type: Option<String>,

    /// Filter by category ID
    #[schema(example = "1234567890")]
    #[serde(default, deserialize_with = "option_string_to_i64")]
    pub category_id: Option<i64>,

    /// Filter by barcode (exact match)
    #[schema(example = "8901234567890")]
    pub barcode: Option<String>,

    /// Sort field: "name", "created_at", or "updated_at" (default: "created_at")
    #[serde(default = "default_sort_field")]
    #[schema(example = "created_at")]
    pub sort_field: String,

    /// Sort direction: "asc" or "desc" (default: "desc")
    #[serde(default = "default_sort_direction")]
    #[schema(example = "desc")]
    pub sort_direction: String,

    /// Opaque cursor from the previous page's `next_cursor` (omit for the first page)
    #[schema(example = "eyJmaWVsZF92YWx1ZSI6IjIwMjUtMDEtMDEiLCJpZCI6MTIzfQ==")]
    pub cursor: Option<String>,

    /// Maximum number of items per page (default: 20, max: 100)
    #[serde(default = "default_page_size")]
    #[schema(example = 20)]
    pub limit: u32,
}

impl VariantSearchQueryParams {
    pub fn to_query(
        &self,
    ) -> Result<sultan_core::domain::model::product::VariantSearchQuery, sultan_core::domain::Error>
    {
        use sultan_core::domain::model::product::{
            ProductCursor, VariantSearchFilter, VariantSearchQuery,
        };

        let sort_field = parse_sort_field(&self.sort_field)?;
        let sort_direction = super::parse_sort_direction(&self.sort_direction)?;
        let cursor = self
            .cursor
            .as_deref()
            .map(super::decode_cursor::<ProductCursor>)
            .transpose()?;
        let limit = self.limit.clamp(1, 100) as u64;

        Ok(VariantSearchQuery {
            filter: VariantSearchFilter {
                name: self.name.clone(),
                product_type: self
                    .product_type
                    .as_deref()
                    .map(|pt| pt.parse())
                    .transpose()?,
                category_id: self.category_id,
                barcode: self.barcode.clone(),
            },
            sort_field,
            sort_direction,
            cursor,
            limit,
        })
    }
}

/// A single variant in the search result, including its parent product and categories.
#[derive(Debug, Serialize, ToSchema)]
pub struct VariantSearchItemResponse {
    /// Variant ID
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub id: i64,

    /// Variant barcode
    #[schema(example = "8901234567890")]
    pub barcode: Option<String>,

    /// Variant name
    #[schema(example = "Black - 16GB RAM")]
    pub name: Option<String>,

    /// Additional metadata
    pub metadata: Option<Value>,

    /// Parent product
    pub product: ProductResponse,

    /// Sell prices for this variant
    pub sell_prices: Vec<SellPriceResponse>,

    /// Categories (from the parent product)
    pub categories: Vec<CategoryChildResponse>,
}

impl From<sultan_core::domain::model::product::ProductVariantRead> for VariantSearchItemResponse {
    fn from(v: sultan_core::domain::model::product::ProductVariantRead) -> Self {
        Self {
            id: v.id,
            barcode: v.barcode,
            name: v.name,
            metadata: v.metadata,
            product: ProductResponse::from(v.product),
            sell_prices: v
                .sell_prices
                .into_iter()
                .map(SellPriceResponse::from)
                .collect(),
            categories: v
                .categories
                .into_iter()
                .map(|c| CategoryChildResponse {
                    id: c.id,
                    name: c.name,
                    description: c.description,
                })
                .collect(),
        }
    }
}

/// Response for a paginated variant search (cursor-based).
#[derive(Debug, Serialize, ToSchema)]
pub struct VariantSearchListResponse {
    /// The items in this page
    pub items: Vec<VariantSearchItemResponse>,

    /// Opaque cursor to fetch the next page. `null` when there are no more pages.
    #[schema(example = "eyJmaWVsZF92YWx1ZSI6IjIwMjUtMDEtMDEiLCJpZCI6MTIzfQ==")]
    pub next_cursor: Option<String>,
}

impl VariantSearchListResponse {
    pub fn from_cursor_page(
        page: sultan_core::domain::model::product::CursorPage<
            sultan_core::domain::model::product::ProductVariantRead,
        >,
    ) -> Self {
        use base64::Engine;

        let next_cursor = page.next_cursor.map(|c| {
            let json = serde_json::to_vec(&c).expect("cursor is always serializable");
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json)
        });

        Self {
            items: page
                .items
                .into_iter()
                .map(VariantSearchItemResponse::from)
                .collect(),
            next_cursor,
        }
    }
}
