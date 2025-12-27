use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sultan_core::domain::model::Update;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CustomerCreateRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Name must be between 1 and 100 characters"
    ))]
    #[schema(example = "CV. Sultan Pos")]
    pub name: String,
    pub number: Option<String>,
    pub address: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub level: i32,
    pub metadata: Option<Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CustomerCreateResponse {
    #[schema(example = 1)]
    pub id: i64,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CustomerUpdateRequest {
    #[schema(example = "CV. Sultan Pos")]
    pub name: Option<String>,
    pub number: Option<String>,
    #[schema(value_type = Option<String>)]
    pub address: Update<String>,
    #[schema(value_type = Option<String>)]
    pub email: Update<String>,
    #[schema(value_type = Option<String>)]
    pub phone: Update<String>,
    pub level: Option<i32>,
    #[schema(value_type = Option<Value>)]
    pub metadata: Update<Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CustomerResponse {
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub number: String,
    pub name: String,
    pub address: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub level: i32,
    pub metadata: Option<Value>,
}

impl From<sultan_core::domain::model::customer::Customer> for CustomerResponse {
    fn from(customer: sultan_core::domain::model::customer::Customer) -> Self {
        Self {
            id: customer.id,
            name: customer.name,
            number: customer.number,
            address: customer.address,
            email: customer.email,
            phone: customer.phone,
            level: customer.level,
            metadata: customer.metadata,
            created_at: customer.created_at,
            updated_at: customer.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CustomerQueryParams {
    /// Customer number filter
    pub number: Option<String>,
    /// Customer name filter (partial match)
    pub name: Option<String>,
    /// Phone number filter
    pub phone: Option<String>,
    /// Email filter
    pub email: Option<String>,
    /// Customer level filter
    pub level: Option<i32>,
    /// Page number (default: 1)
    #[serde(default = "default_page")]
    pub page: u32,
    /// Page size (default: 20, max: 100)
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    /// Order by field
    pub order_by: Option<String>,
    /// Order direction (asc/desc)
    pub order_direction: Option<String>,
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    20
}

impl CustomerQueryParams {
    /// Convert to CustomerFilter
    pub fn to_filter(&self) -> sultan_core::domain::model::customer::CustomerFilter {
        sultan_core::domain::model::customer::CustomerFilter {
            number: self.number.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            email: self.email.clone(),
            level: self.level,
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

#[derive(Debug, Serialize, ToSchema)]
pub struct CustomerListResponse {
    pub customers: Vec<CustomerResponse>,
}
