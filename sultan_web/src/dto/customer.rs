use super::i64_to_string;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sultan_core::domain::model::Update;
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use super::default_page_size;

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
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
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
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
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

#[derive(Debug, Serialize, ToSchema)]
pub struct CustomerListResponse {
    pub items: Vec<CustomerResponse>,
    /// Opaque cursor to fetch the next page. `null` when there are no more pages.
    #[schema(example = "eyJmaWVsZF92YWx1ZSI6IjEyMzQ1NiIsImlkIjoxMjM0NX0")]
    pub next_cursor: Option<String>,
}

impl CustomerListResponse {
    pub fn from_page(page: sultan_core::domain::model::customer::CustomerPage) -> Self {
        use base64::Engine;

        let next_cursor = page.next_cursor.map(|c| {
            let json = serde_json::to_vec(&c).expect("cursor is always serializable");
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json)
        });

        Self {
            items: page.items.into_iter().map(CustomerResponse::from).collect(),
            next_cursor,
        }
    }
}

// ── Query Params ──────────────────────────────────────────────────────────────

/// Query parameters for listing customers with cursor-based pagination.
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct CustomerQueryParams {
    /// Customer number filter (partial match)
    pub number: Option<String>,
    /// Customer name filter (partial match)
    pub name: Option<String>,
    /// Phone number filter (partial match)
    pub phone: Option<String>,
    /// Email filter (partial match)
    pub email: Option<String>,
    /// Customer level filter (exact match)
    pub level: Option<i32>,

    /// Sort field: "id", "updated_at", or "name" (default: "id")
    #[serde(default = "default_customer_sort_field")]
    #[schema(example = "id")]
    pub sort_field: String,

    /// Sort direction: "asc" or "desc" (default: "asc")
    #[serde(default = "default_sort_direction")]
    #[schema(example = "asc")]
    pub sort_direction: String,

    /// Opaque cursor from the previous page's `next_cursor` (omit for the first page)
    #[schema(example = "eyJmaWVsZF92YWx1ZSI6IjEyMzQ1NiIsImlkIjoxMjM0NX0")]
    pub cursor: Option<String>,

    /// Maximum number of items per page (default: 20, max: 100)
    #[serde(default = "default_page_size")]
    #[schema(example = 20)]
    pub limit: u32,
}

fn default_customer_sort_field() -> String {
    "id".to_string()
}

fn default_sort_direction() -> String {
    "asc".to_string()
}

impl CustomerQueryParams {
    /// Convert to CustomerQuery for cursor-based pagination.
    pub fn to_query(
        &self,
    ) -> Result<sultan_core::domain::model::customer::CustomerQuery, sultan_core::domain::Error>
    {
        use sultan_core::domain::model::customer::{
            CustomerCursor, CustomerFilter, CustomerQuery, CustomerSortField,
        };

        let sort_field = match self.sort_field.as_str() {
            "id" => CustomerSortField::Id,
            "updated_at" => CustomerSortField::UpdatedAt,
            "name" => CustomerSortField::Name,
            other => {
                return Err(sultan_core::domain::Error::ValidationError(format!(
                    "Invalid sort_field '{}'. Must be one of: id, updated_at, name",
                    other
                )));
            }
        };

        let sort_direction = super::parse_sort_direction(&self.sort_direction)?;
        let cursor = self
            .cursor
            .as_deref()
            .map(super::decode_cursor::<CustomerCursor>)
            .transpose()?;

        Ok(CustomerQuery {
            filter: CustomerFilter {
                number: self.number.clone(),
                name: self.name.clone(),
                phone: self.phone.clone(),
                email: self.email.clone(),
                level: self.level,
            },
            sort_field,
            sort_direction,
            cursor,
            limit: self.limit.clamp(1, 100) as u64,
        })
    }
}
