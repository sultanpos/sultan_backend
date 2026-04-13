use super::{default_page_size, i64_to_string};
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
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
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
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
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

#[derive(Debug, Serialize, ToSchema)]
pub struct SupplierListResponse {
    pub items: Vec<SupplierResponse>,
    /// Opaque cursor to fetch the next page. `null` when there are no more pages.
    #[schema(example = "eyJmaWVsZF92YWx1ZSI6IjEyMzQ1NiIsImlkIjoxMjM0NX0")]
    pub next_cursor: Option<String>,
}

impl SupplierListResponse {
    pub fn from_page(page: sultan_core::domain::model::supplier::SupplierPage) -> Self {
        use base64::Engine;

        let next_cursor = page.next_cursor.map(|c| {
            let json = serde_json::to_vec(&c).expect("cursor is always serializable");
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json)
        });

        Self {
            items: page.items.into_iter().map(SupplierResponse::from).collect(),
            next_cursor,
        }
    }
}

// ── Query Params ──────────────────────────────────────────────────────────────

/// Query parameters for listing suppliers with cursor-based pagination.
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

    /// Sort field: "id", "updated_at", or "name" (default: "id")
    #[serde(default = "default_supplier_sort_field")]
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

fn default_supplier_sort_field() -> String {
    "id".to_string()
}

fn default_sort_direction() -> String {
    "asc".to_string()
}

impl SupplierQueryParams {
    /// Convert to SupplierQuery for cursor-based pagination.
    pub fn to_query(
        &self,
    ) -> Result<sultan_core::domain::model::supplier::SupplierQuery, sultan_core::domain::Error>
    {
        use sultan_core::domain::model::product::SortDirection;
        use sultan_core::domain::model::supplier::{
            SupplierCursor, SupplierFilter, SupplierQuery, SupplierSortField,
        };

        let sort_field = match self.sort_field.as_str() {
            "id" => SupplierSortField::Id,
            "updated_at" => SupplierSortField::UpdatedAt,
            "name" => SupplierSortField::Name,
            other => {
                return Err(sultan_core::domain::Error::ValidationError(format!(
                    "Invalid sort_field '{}'. Must be one of: id, updated_at, name",
                    other
                )));
            }
        };

        let sort_direction = match self.sort_direction.as_str() {
            "asc" => SortDirection::Asc,
            "desc" => SortDirection::Desc,
            other => {
                return Err(sultan_core::domain::Error::ValidationError(format!(
                    "Invalid sort_direction '{}'. Must be 'asc' or 'desc'",
                    other
                )));
            }
        };

        let cursor = self
            .cursor
            .as_deref()
            .map(|encoded| {
                use base64::Engine;
                let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
                    .decode(encoded)
                    .map_err(|_| {
                        sultan_core::domain::Error::ValidationError(
                            "Invalid cursor encoding".to_string(),
                        )
                    })?;
                serde_json::from_slice::<SupplierCursor>(&bytes).map_err(|_| {
                    sultan_core::domain::Error::ValidationError("Invalid cursor format".to_string())
                })
            })
            .transpose()?;

        Ok(SupplierQuery {
            filter: SupplierFilter {
                code: self.code.clone(),
                name: self.name.clone(),
                phone: self.phone.clone(),
                email: self.email.clone(),
                npwp: self.npwp.clone(),
            },
            sort_field,
            sort_direction,
            cursor,
            limit: self.limit.clamp(1, 100) as u64,
        })
    }
}
