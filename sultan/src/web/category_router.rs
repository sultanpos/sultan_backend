use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use std::sync::Arc;
use sultan_core::application::AuthServiceTrait;
use sultan_core::domain::context::BranchContext;
use tracing::instrument;
use utoipa::OpenApi;
use validator::Validate;

use crate::domain::dto::{CategoryResponse, CreateCategoryRequest, ErrorResponse};
use crate::web::AppState;

// ============================================================================
// OpenAPI Documentation
// ============================================================================

#[derive(OpenApi)]
#[openapi(
    paths(create),
    components(schemas(CreateCategoryRequest, CategoryResponse, ErrorResponse)),
    tags(
        (name = "category", description = "Category management endpoints")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub struct CategoryApiDoc;

// ============================================================================
// HTTP Handlers
// ============================================================================

/// Create a new category
///
/// Creates a new category with the provided information. Requires authentication.
#[utoipa::path(
    post,
    path = "/api/category",
    tag = "category",
    request_body = CreateCategoryRequest,
    responses(
        (status = 200, description = "Category created successfully", body = CategoryResponse),
        (status = 400, description = "Bad request - validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(_auth_service, payload))]
async fn create(
    State(_auth_service): State<Arc<dyn AuthServiceTrait<BranchContext>>>,
    Json(payload): Json<CreateCategoryRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // Validate input
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("{}", e),
            }),
        )
    })?;

    // TODO: Implement actual category creation logic
    // For now, return a mock response
    Ok((
        StatusCode::OK,
        Json(CategoryResponse {
            id: 1,
            name: payload.name,
            description: payload.description,
            parent_id: payload.parent_id,
            created_at: chrono::Utc::now().to_rfc3339(),
        }),
    ))
}

// ============================================================================
// Router
// ============================================================================

pub fn category_router() -> Router<AppState> {
    Router::new().route("/", post(create))
}
