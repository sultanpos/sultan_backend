use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use std::sync::Arc;
use sultan_core::domain::DomainResult;
use tracing::instrument;

use crate::domain::dto::{LoginRequest, LoginResponse};
use crate::web::{AppState, app_state::ConcreteAuthService};
use crate::with_branch_context;

// ============================================================================
// HTTP Handlers
// ============================================================================

/// Register a new user
#[instrument(skip(auth_service, payload))]
async fn login(
    State(auth_service): State<Arc<ConcreteAuthService>>,
    Json(payload): Json<LoginRequest>,
) -> DomainResult<impl IntoResponse> {
    with_branch_context!(ctx => {
        let tokens = auth_service
            .login(&ctx, &payload.username, &payload.password)
            .await?;

        Ok((
            StatusCode::OK,
            Json(LoginResponse {
                access_token: tokens.access_token,
                refresh_token: tokens.refresh_token,
            }),
        ))
    })
} // ============================================================================
// Router
// ============================================================================

pub fn auth_router() -> Router<AppState> {
    Router::new().route("/", post(login))
}
