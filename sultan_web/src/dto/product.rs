use super::category::CategoryChildResponse;
use super::{
    default_page, default_page_size, i64_to_string, option_i64_to_string, option_string_to_i64,
    string_to_i64,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sultan_core::domain::model::{Update, product::Product};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

// ===== Product DTOs =====

/// Request to create a new product
#[derive(Debug, Deserialize, Validate, ToSchema)]
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

    /// Product type (e.g., "goods", "service")
    #[validate(length(
        min = 1,
        max = 50,
        message = "Product type must be between 1 and 50 characters"
    ))]
    #[schema(example = "goods")]
    pub product_type: String,

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

    /// Whether the product has variants
    #[schema(example = true)]
    pub has_variant: bool,

    /// Additional metadata (optional)
    #[schema(example = json!({"color": "black", "warranty": "2 years"}))]
    pub metadata: Option<Value>,

    /// Category IDs this product belongs to
    #[schema(example = json!(["1234567890", "9876543210"]))]
    #[serde(default)]
    pub category_ids: Vec<String>,
}

impl ProductCreateRequest {
    /// Convert category_ids from strings to i64
    pub fn to_domain(&self) -> sultan_core::domain::model::product::ProductCreate {
        let category_ids: Result<Vec<i64>, _> =
            self.category_ids.iter().map(|s| s.parse::<i64>()).collect();

        sultan_core::domain::model::product::ProductCreate {
            name: self.name.clone(),
            description: self.description.clone(),
            product_type: self.product_type.clone(),
            main_image: self.main_image.clone(),
            sellable: self.sellable,
            buyable: self.buyable,
            editable_price: self.editable_price,
            has_variant: self.has_variant,
            metadata: self.metadata.clone(),
            category_ids: category_ids
                .map_err(|e| format!("Invalid category ID: {}", e))
                .unwrap_or_default(),
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

/// Request to update a product
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ProductUpdateRequest {
    /// Product name
    #[validate(length(
        min = 1,
        max = 256,
        message = "Name must be between 1 and 256 characters"
    ))]
    #[schema(example = "Laptop ASUS ROG Updated")]
    pub name: Option<String>,

    /// Product description (optional)
    #[schema(example = "Updated description", value_type = Option<String>)]
    pub description: Update<String>,

    /// Product type
    #[schema(example = "goods")]
    pub product_type: Option<String>,

    /// Main product image URL
    #[schema(example = "https://example.com/images/laptop-new.jpg", value_type = Option<String>)]
    pub main_image: Update<String>,

    /// Whether the product can be sold
    #[schema(example = true)]
    pub sellable: Option<bool>,

    /// Whether the product can be purchased
    #[schema(example = true)]
    pub buyable: Option<bool>,

    /// Whether the price can be edited during sale
    #[schema(example = false)]
    pub editable_price: Option<bool>,

    /// Whether the product has variants
    #[schema(example = true)]
    pub has_variant: Option<bool>,

    /// Additional metadata
    #[schema(value_type = Option<Value>)]
    pub metadata: Update<Value>,

    /// Category IDs this product belongs to
    #[schema(example = json!(["1234567890", "9876543210"]))]
    pub category_ids: Option<Vec<String>>,
}

impl ProductUpdateRequest {
    /// Convert to domain ProductUpdate
    pub fn to_domain(&self) -> Result<sultan_core::domain::model::product::ProductUpdate, String> {
        let category_ids = if let Some(ref ids) = self.category_ids {
            let parsed: Result<Vec<i64>, _> = ids.iter().map(|s| s.parse::<i64>()).collect();
            Some(parsed.map_err(|e| format!("Invalid category ID: {}", e))?)
        } else {
            None
        };

        Ok(sultan_core::domain::model::product::ProductUpdate {
            name: self.name.clone(),
            description: self.description.clone(),
            product_type: self.product_type.clone(),
            main_image: self.main_image.clone(),
            sellable: self.sellable,
            buyable: self.buyable,
            editable_price: self.editable_price,
            has_variant: self.has_variant,
            metadata: self.metadata.clone(),
            category_ids,
        })
    }
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
    #[schema(example = "goods")]
    pub product_type: String,

    /// Main product image URL
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

    /// Whether the product has variants
    #[schema(example = true)]
    pub has_variant: bool,

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
            has_variant: product.has_variant,
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

/// Query parameters for filtering and paginating products
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct ProductQueryParams {
    /// Product name filter (partial match)
    #[schema(example = "Laptop")]
    pub name: Option<String>,

    /// Product type filter
    #[schema(example = "goods")]
    pub product_type: Option<String>,

    /// Category ID filter
    #[schema(example = "1234567890")]
    #[serde(default, deserialize_with = "option_string_to_i64")]
    pub category_id: Option<i64>,

    /// Page number (default: 1)
    #[serde(default = "default_page")]
    #[schema(example = 1)]
    pub page: u32,

    /// Page size (default: 20, max: 100)
    #[serde(default = "default_page_size")]
    #[schema(example = 20)]
    pub page_size: u32,

    /// Order by field
    #[schema(example = "name")]
    pub order_by: Option<String>,

    /// Order direction (asc/desc)
    #[schema(example = "asc")]
    pub order_direction: Option<String>,
}

impl ProductQueryParams {
    /// Convert to ProductFilter
    pub fn to_filter(&self) -> sultan_core::domain::model::product::ProductFilter {
        sultan_core::domain::model::product::ProductFilter {
            name: self.name.clone(),
            product_type: self.product_type.clone(),
            category_id: self.category_id,
        }
    }

    /// Convert to PaginationOptions
    pub fn to_pagination(&self) -> sultan_core::domain::model::pagination::PaginationOptions {
        use sultan_core::domain::model::pagination::{PaginationOptions, PaginationOrder};

        let page_size = self.page_size.min(100); // Cap at 100
        let order = match (self.order_by.as_ref(), self.order_direction.as_ref()) {
            (Some(field), direction) => Some(PaginationOrder {
                field: field.clone(),
                direction: direction.cloned().unwrap_or_else(|| "asc".to_string()),
            }),
            _ => None,
        };

        PaginationOptions::new(self.page, page_size, order)
    }
}

/// List of products response
#[derive(Debug, Serialize, ToSchema)]
pub struct ProductListResponse {
    pub products: Vec<ProductResponse>,
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
    /// Variant barcode
    #[schema(example = "8901234567890", value_type = Option<String>)]
    pub barcode: Update<String>,

    /// Variant name
    #[schema(example = "Black - 16GB RAM", value_type = Option<String>)]
    pub name: Update<String>,

    /// Additional metadata
    #[schema(value_type = Option<Value>)]
    pub metadata: Update<Value>,
}

impl From<ProductVariantUpdateRequest>
    for sultan_core::domain::model::product::ProductVariantUpdate
{
    fn from(req: ProductVariantUpdateRequest) -> Self {
        Self {
            barcode: req.barcode,
            name: req.name,
            metadata: req.metadata,
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
    pub quantity: i64,

    /// Discount formula (e.g., "price * 0.9" for 10% off)
    #[schema(example = "price * 0.9")]
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
    pub quantity: i64,

    /// Price in cents/smallest currency unit
    #[schema(example = 150000)]
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
    pub sell_price: SellPriceCreateRequest,

    /// List of discounts for this price
    #[serde(default)]
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
    #[serde(default)]
    pub categories: Vec<String>,
}

impl ProductFullCreateRequest {
    pub fn to_domain(&self) -> sultan_core::domain::model::product::ProductFullCreate {
        let product = self.product.to_domain();

        let categories: Result<Vec<i64>, _> =
            self.categories.iter().map(|s| s.parse::<i64>()).collect();

        let categories = categories
            .map_err(|e| format!("Invalid category ID: {}", e))
            .unwrap_or_default();

        sultan_core::domain::model::product::ProductFullCreate {
            product,
            variants: self.variants.iter().map(|v| v.to_domain()).collect(),
            categories,
        }
    }
}
