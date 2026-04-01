use super::i64_to_string;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sultan_core::domain::model::cashier_session::{
    CashierSession, CashierSessionCursor, SessionStatus,
};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

// ── Open (Create) ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct OpenSessionRequest {
    /// Branch the session belongs to
    #[schema(example = "1234567890", value_type = String)]
    #[serde(deserialize_with = "super::string_to_i64")]
    pub branch_id: i64,

    /// Opening cash amount (in smallest currency unit, e.g. cents)
    #[validate(range(min = 0, message = "Opening cash must be non-negative"))]
    #[schema(example = 100000)]
    pub opening_cash: i64,

    pub notes: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OpenSessionResponse {
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub id: i64,
}

// ── Close ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CloseSessionRequest {
    /// Closing cash amount (in smallest currency unit)
    #[schema(example = 200000)]
    pub closing_cash: i64,

    pub notes: Option<String>,
}

// ── Response ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct CashierSessionResponse {
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub branch_id: i64,
    #[schema(example = "9876543210", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub user_id: i64,
    pub opened_at: chrono::DateTime<Utc>,
    pub closed_at: Option<chrono::DateTime<Utc>>,
    #[schema(value_type = String, example = "open")]
    pub status: SessionStatus,
    pub opening_cash: i64,
    pub closing_cash: Option<i64>,
    pub notes: Option<String>,
}

impl From<CashierSession> for CashierSessionResponse {
    fn from(s: CashierSession) -> Self {
        Self {
            id: s.id,
            created_at: s.created_at,
            updated_at: s.updated_at,
            branch_id: s.branch_id,
            user_id: s.user_id,
            opened_at: s.opened_at,
            closed_at: s.closed_at,
            status: s.status,
            opening_cash: s.opening_cash,
            closing_cash: s.closing_cash,
            notes: s.notes,
        }
    }
}

// ── Query Params ──────────────────────────────────────────────────────────────

/// Query parameters for listing cashier sessions with cursor-based pagination.
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct CashierSessionQueryParams {
    /// Filter by branch ID
    #[schema(example = "1234567890")]
    #[serde(default, deserialize_with = "super::option_string_to_i64")]
    pub branch_id: Option<i64>,

    /// Filter by user ID
    #[schema(example = "9876543210")]
    #[serde(default, deserialize_with = "super::option_string_to_i64")]
    pub user_id: Option<i64>,

    /// Filter by status: "open" or "closed"
    #[schema(example = "open")]
    pub status: Option<String>,

    /// Sort direction: "asc" or "desc" (default: "desc")
    #[serde(default = "default_sort_direction")]
    #[schema(example = "desc")]
    pub sort_direction: String,

    /// Opaque cursor from the previous page's `next_cursor` (omit for the first page)
    pub cursor: Option<String>,

    /// Maximum number of items per page (default: 20, max: 100)
    #[serde(default = "super::default_page_size")]
    #[schema(example = 20)]
    pub limit: u32,
}

fn default_sort_direction() -> String {
    "desc".to_string()
}

impl CashierSessionQueryParams {
    pub fn to_query(
        &self,
    ) -> Result<
        sultan_core::domain::model::cashier_session::CashierSessionQuery,
        sultan_core::domain::Error,
    > {
        use std::str::FromStr;
        use sultan_core::domain::model::cashier_session::{
            CashierSessionFilter, CashierSessionQuery, CashierSessionSortField,
        };
        use sultan_core::domain::model::product::SortDirection;

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

        let status = self
            .status
            .as_deref()
            .map(SessionStatus::from_str)
            .transpose()?;

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
                serde_json::from_slice::<CashierSessionCursor>(&bytes).map_err(|_| {
                    sultan_core::domain::Error::ValidationError("Invalid cursor format".to_string())
                })
            })
            .transpose()?;

        Ok(CashierSessionQuery {
            filter: CashierSessionFilter {
                branch_id: self.branch_id,
                user_id: self.user_id,
                status,
            },
            sort_field: CashierSessionSortField::OpenedAt,
            sort_direction,
            cursor,
            limit: self.limit.clamp(1, 100) as u64,
        })
    }
}

/// Paginated list of cashier sessions with an optional next cursor.
#[derive(Debug, Serialize, ToSchema)]
pub struct CashierSessionListResponse {
    pub items: Vec<CashierSessionResponse>,
    /// Opaque cursor to fetch the next page. `null` when there are no more pages.
    pub next_cursor: Option<String>,
}

impl CashierSessionListResponse {
    pub fn from_page(
        page: sultan_core::domain::model::cashier_session::CashierSessionPage,
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
                .map(CashierSessionResponse::from)
                .collect(),
            next_cursor,
        }
    }
}
