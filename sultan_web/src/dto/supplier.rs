use super::{default_page, default_page_size};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sultan_core::domain::model::{Update, supplier::Supplier};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SupplierCreateRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Name must be between 1 and 256 characters"
    ))]
    #[schema(example = "CV. Sultan Pos")]
    pub name: String,
    pub code: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub npwp: Option<String>,
    pub npwp_name: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SupplierCreateResponse {
    #[schema(example = 1)]
    pub id: i64,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SupplierUpdateRequest {
    pub name: Option<String>,
    #[schema(value_type = Option<String>)]
    pub code: Update<String>,
    #[schema(value_type = Option<String>)]
    pub email: Update<String>,
    #[schema(value_type = Option<String>)]
    pub address: Update<String>,
    #[schema(value_type = Option<String>)]
    pub phone: Update<String>,
    #[schema(value_type = Option<String>)]
    pub npwp: Update<String>,
    #[schema(value_type = Option<String>)]
    pub npwp_name: Update<String>,
    #[schema(value_type = Option<Value>)]
    pub metadata: Update<Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SupplierResponse {
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub name: String,
    pub code: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub npwp: Option<String>,
    pub npwp_name: Option<String>,
    pub metadata: Option<Value>,
}

impl From<Supplier> for SupplierResponse {
    fn from(supplier: Supplier) -> Self {
        Self {
            id: supplier.id,
            created_at: supplier.created_at,
            updated_at: supplier.updated_at,
            name: supplier.name,
            code: supplier.code,
            email: supplier.email,
            address: supplier.address,
            phone: supplier.phone,
            npwp: supplier.npwp,
            npwp_name: supplier.npwp_name,
            metadata: supplier.metadata,
        }
    }
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct SupplierQueryParams {
    #[schema(example = "CV. Sultan")]
    pub name: Option<String>,
    #[schema(example = "SUP001")]
    pub code: Option<String>,
    #[schema(example = "081234567890")]
    pub phone: Option<String>,
    #[schema(example = "12.345.678.9-012.345")]
    pub npwp: Option<String>,
    #[schema(example = "supplier@example.com")]
    pub email: Option<String>,
    /// Page number (default: 1)
    #[serde(default = "default_page")]
    #[schema(example = 1, default = 1)]
    pub page: u32,
    /// Page size (default: 20, max: 100)
    #[serde(default = "default_page_size")]
    #[schema(example = 20, default = 20)]
    pub page_size: u32,
    /// Order by field
    #[schema(example = "name")]
    pub order_by: Option<String>,
    /// Order direction (asc/desc)
    #[schema(example = "asc")]
    pub order_direction: Option<String>,
}

impl SupplierQueryParams {
    /// Convert to SupplierFilter
    pub fn to_filter(&self) -> sultan_core::domain::model::supplier::SupplierFilter {
        sultan_core::domain::model::supplier::SupplierFilter {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            email: self.email.clone(),
            npwp: self.npwp.clone(),
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
