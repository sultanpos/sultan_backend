use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use std::sync::Arc;
use sultan_core::application::AuthServiceTrait;
use sultan_core::domain::context::BranchContext;
use tracing::instrument;

use crate::domain::dto::LoginRequest;
use crate::web::AppState;

// ============================================================================
// HTTP Handlers
// ============================================================================

/// Register a new user
#[instrument(skip(_auth_service, _payload))]
async fn create(
    State(_auth_service): State<Arc<dyn AuthServiceTrait<BranchContext>>>,
    Json(_payload): Json<LoginRequest>,
) -> impl IntoResponse {
    StatusCode::OK
}

// ============================================================================
// Router
// ============================================================================

pub fn category_router() -> Router<AppState> {
    Router::new().route("/", post(create))
}
