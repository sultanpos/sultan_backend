use axum::routing::delete;
use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use std::sync::Arc;
use sultan_core::application::AuthServiceTrait;
use sultan_core::domain::Error;
use sultan_core::domain::{DomainResult, context::BranchContext};
use tracing::instrument;
use validator::Validate;

use crate::domain::dto::{LoginRequest, LoginResponse, LogoutRequest, RefreshTokenRequest};
use crate::web::AppState;
use crate::with_branch_context;

// ============================================================================
// HTTP Handlers
// ============================================================================

/// Register a new user
#[instrument(skip(auth_service, payload))]
async fn login(
    State(auth_service): State<Arc<dyn AuthServiceTrait<BranchContext>>>,
    Json(payload): Json<LoginRequest>,
) -> DomainResult<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;

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
}

#[instrument(skip(auth_service, payload))]
async fn refresh(
    State(auth_service): State<Arc<dyn AuthServiceTrait<BranchContext>>>,
    Json(payload): Json<RefreshTokenRequest>,
) -> DomainResult<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;

    with_branch_context!(ctx => {
        let tokens = auth_service
            .refresh(&ctx, &payload.refresh_token)
            .await?;

        Ok((
            StatusCode::OK,
            Json(LoginResponse {
                access_token: tokens.access_token,
                refresh_token: tokens.refresh_token,
            }),
        ))
    })
}

#[instrument(skip(auth_service, payload))]
async fn logout(
    State(auth_service): State<Arc<dyn AuthServiceTrait<BranchContext>>>,
    Json(payload): Json<LogoutRequest>,
) -> DomainResult<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;

    with_branch_context!(ctx => {
        auth_service
            .logout(&ctx, &payload.refresh_token)
            .await?;

        Ok(StatusCode::NO_CONTENT)
    })
}

// ============================================================================
// Router
// ============================================================================

pub fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/", post(login))
        .route("/refresh", post(refresh))
        .route("/", delete(logout))
}
