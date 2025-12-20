use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use sultan_core::domain::DomainResult;
use tracing::{info, instrument};

use crate::web::{AppState, app_state::ConcreteAuthService};

// ============================================================================
// DTOs (Request/Response models)
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Clone, Serialize)]
struct LoginResponse {
    success: bool,
}

// ============================================================================
// HTTP Handlers
// ============================================================================

/// Register a new user
#[instrument(skip(auth_service, payload))]
async fn login(
    State(auth_service): State<Arc<ConcreteAuthService>>,
    Json(payload): Json<LoginRequest>,
) -> DomainResult<impl IntoResponse> {
    Ok((StatusCode::OK, Json(LoginResponse { success: true })))
}

// ============================================================================
// Router
// ============================================================================

pub fn auth_router() -> Router<AppState> {
    Router::new().route("/", post(login))
}
