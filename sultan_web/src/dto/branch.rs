use super::i64_to_string;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sultan_core::domain::model::Update;
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct BranchCreateRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Name must be between 1 and 100 characters"
    ))]
    #[schema(example = "Sultan Branch")]
    pub name: String,
    #[validate(length(
        min = 1,
        max = 50,
        message = "Code must be between 1 and 50 characters"
    ))]
    #[schema(example = "SULTAN")]
    pub code: String,
    #[schema(example = false)]
    #[serde(default)]
    pub is_main: bool,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub npwp: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BranchCreateResponse {
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub id: i64,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct BranchUpdateRequest {
    #[schema(example = "Sultan Branch")]
    pub name: Option<String>,
    #[schema(example = "SULTAN")]
    pub code: Option<String>,
    pub is_main: Option<bool>,
    #[schema(value_type = Option<String>)]
    pub address: Update<String>,
    #[schema(value_type = Option<String>)]
    pub phone: Update<String>,
    #[schema(value_type = Option<String>)]
    pub npwp: Update<String>,
    #[schema(value_type = Option<String>)]
    pub image: Update<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BranchResponse {
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub is_main: bool,
    pub name: String,
    pub code: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub npwp: Option<String>,
    pub image: Option<String>,
}

impl From<sultan_core::domain::model::branch::Branch> for BranchResponse {
    fn from(branch: sultan_core::domain::model::branch::Branch) -> Self {
        Self {
            id: branch.id,
            created_at: branch.created_at,
            updated_at: branch.updated_at,
            is_main: branch.is_main,
            name: branch.name,
            code: branch.code,
            address: branch.address,
            phone: branch.phone,
            npwp: branch.npwp,
            image: branch.image,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BranchListResponse {
    pub items: Vec<BranchResponse>,
    /// Opaque cursor to fetch the next page. `null` when there are no more pages.
    #[schema(example = "eyJmaWVsZF92YWx1ZSI6IjIwMjUtMDEtMDEiLCJpZCI6MTIzfQ")]
    pub next_cursor: Option<String>,
}

impl BranchListResponse {
    pub fn from_page(page: sultan_core::domain::model::branch::BranchPage) -> Self {
        use base64::Engine;

        let next_cursor = page.next_cursor.map(|c| {
            let json = serde_json::to_vec(&c).expect("cursor is always serializable");
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json)
        });

        Self {
            items: page.items.into_iter().map(BranchResponse::from).collect(),
            next_cursor,
        }
    }
}

// ── Query Params ──────────────────────────────────────────────────────────────

/// Query parameters for listing branches with cursor-based pagination.
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct BranchQueryParams {
    /// Filter by name (partial match)
    #[schema(example = "Main")]
    pub name: Option<String>,

    /// Sort field: "name" or "created_at" (default: "created_at")
    #[serde(default = "default_sort_field")]
    #[schema(example = "created_at")]
    pub sort_field: String,

    /// Sort direction: "asc" or "desc" (default: "desc")
    #[serde(default = "default_sort_direction")]
    #[schema(example = "desc")]
    pub sort_direction: String,

    /// Opaque cursor from the previous page's `next_cursor` (omit for the first page)
    #[schema(example = "eyJmaWVsZF92YWx1ZSI6IjIwMjUtMDEtMDEiLCJpZCI6MTIzfQ")]
    pub cursor: Option<String>,

    /// Maximum number of items per page (default: 20, max: 100)
    #[serde(default = "super::default_page_size")]
    #[schema(example = 20)]
    pub limit: u32,
}

fn default_sort_field() -> String {
    "created_at".to_string()
}

fn default_sort_direction() -> String {
    "desc".to_string()
}

impl BranchQueryParams {
    pub fn to_query(
        &self,
    ) -> Result<sultan_core::domain::model::branch::BranchQuery, sultan_core::domain::Error> {
        use sultan_core::domain::model::branch::{
            BranchCursor, BranchFilter, BranchQuery, BranchSortField,
        };

        let sort_field = match self.sort_field.as_str() {
            "name" => BranchSortField::Name,
            "created_at" => BranchSortField::CreatedAt,
            other => {
                return Err(sultan_core::domain::Error::ValidationError(format!(
                    "Invalid sort_field '{}'. Must be one of: name, created_at",
                    other
                )));
            }
        };

        let sort_direction = super::parse_sort_direction(&self.sort_direction)?;
        let cursor = self
            .cursor
            .as_deref()
            .map(super::decode_cursor::<BranchCursor>)
            .transpose()?;

        Ok(BranchQuery {
            filter: BranchFilter {
                name: self.name.clone(),
            },
            sort_field,
            sort_direction,
            cursor,
            limit: self.limit.clamp(1, 100) as u64,
        })
    }
}
