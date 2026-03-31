use super::i64_to_string;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sultan_core::domain::model::{Update, machine::Machine};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

// ── Create ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct MachineCreateRequest {
    /// Branch the machine belongs to
    #[schema(example = "1234567890", value_type = String)]
    #[serde(deserialize_with = "super::string_to_i64")]
    pub branch_id: i64,

    /// Unique key identifying this machine within the branch (immutable after creation)
    #[validate(length(
        min = 1,
        max = 100,
        message = "Key must be between 1 and 100 characters"
    ))]
    #[schema(example = "POS-01")]
    pub key: String,

    /// Human-readable name
    #[validate(length(
        min = 1,
        max = 255,
        message = "Name must be between 1 and 255 characters"
    ))]
    #[schema(example = "Counter 1")]
    pub name: String,

    pub description: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MachineCreateResponse {
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub id: i64,
}

// ── Update ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct MachineUpdateRequest {
    pub name: Option<String>,
    #[schema(value_type = Option<String>)]
    pub description: Update<String>,
    #[schema(value_type = Option<Value>)]
    pub metadata: Update<Value>,
}

// ── Response ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct MachineResponse {
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub branch_id: i64,
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub metadata: Option<Value>,
}

impl From<Machine> for MachineResponse {
    fn from(m: Machine) -> Self {
        Self {
            id: m.id,
            created_at: m.created_at,
            updated_at: m.updated_at,
            branch_id: m.branch_id,
            key: m.key,
            name: m.name,
            description: m.description,
            metadata: m.metadata,
        }
    }
}

// ── Query Params ──────────────────────────────────────────────────────────────

/// Query parameters for listing machines with cursor-based pagination.
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct MachineQueryParams {
    /// Filter by branch ID
    #[schema(example = "1234567890")]
    #[serde(default, deserialize_with = "super::option_string_to_i64")]
    pub branch_id: Option<i64>,

    /// Filter by name (partial match)
    #[schema(example = "Counter")]
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
    #[schema(example = "eyJmaWVsZF92YWx1ZSI6IkNvdW50ZXIiLCJpZCI6MX0")]
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

impl MachineQueryParams {
    pub fn to_query(
        &self,
    ) -> Result<sultan_core::domain::model::machine::MachineQuery, sultan_core::domain::Error> {
        use sultan_core::domain::model::machine::{
            MachineCursor, MachineFilter, MachineQuery, MachineSortField,
        };
        use sultan_core::domain::model::product::SortDirection;

        let sort_field = match self.sort_field.as_str() {
            "name" => MachineSortField::Name,
            "created_at" => MachineSortField::CreatedAt,
            other => {
                return Err(sultan_core::domain::Error::ValidationError(format!(
                    "Invalid sort_field '{}'. Must be one of: name, created_at",
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
                serde_json::from_slice::<MachineCursor>(&bytes).map_err(|_| {
                    sultan_core::domain::Error::ValidationError("Invalid cursor format".to_string())
                })
            })
            .transpose()?;

        Ok(MachineQuery {
            filter: MachineFilter {
                branch_id: self.branch_id,
                name: self.name.clone(),
            },
            sort_field,
            sort_direction,
            cursor,
            limit: self.limit.clamp(1, 100) as u64,
        })
    }
}

/// Paginated list of machines with an optional next cursor.
#[derive(Debug, Serialize, ToSchema)]
pub struct MachineListResponse {
    pub items: Vec<MachineResponse>,
    /// Opaque cursor to fetch the next page. `null` when there are no more pages.
    #[schema(example = "eyJmaWVsZF92YWx1ZSI6IkNvdW50ZXIiLCJpZCI6MX0")]
    pub next_cursor: Option<String>,
}

impl MachineListResponse {
    pub fn from_page(page: sultan_core::domain::model::machine::MachinePage) -> Self {
        use base64::Engine;

        let next_cursor = page.next_cursor.map(|c| {
            let json = serde_json::to_vec(&c).expect("cursor is always serializable");
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json)
        });

        Self {
            items: page.items.into_iter().map(MachineResponse::from).collect(),
            next_cursor,
        }
    }
}
