use axum::routing::delete;
use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use std::sync::Arc;
use sultan_core::application::AuthServiceTrait;
use sultan_core::domain::Error;
use sultan_core::domain::{DomainResult, context::Context};
use tracing::instrument;
use utoipa::OpenApi;
use validator::Validate;

use crate::domain::dto::{
    ErrorResponse, LoginRequest, LoginResponse, LogoutRequest, RefreshTokenRequest,
};
use crate::web::AppState;

// ============================================================================
// OpenAPI Documentation
// ============================================================================

#[derive(OpenApi)]
#[openapi(
    paths(login, refresh, logout),
    components(schemas(LoginRequest, LoginResponse, RefreshTokenRequest, LogoutRequest, ErrorResponse)),
    tags(
        (name = "auth", description = "Authentication endpoints")
    )
)]
pub struct AuthApiDoc;

// ============================================================================
// HTTP Handlers
// ============================================================================

/// Login with username and password
///
/// Authenticate a user with their credentials and receive access and refresh tokens.
#[utoipa::path(
    post,
    path = "/api/auth",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 400, description = "Bad request - validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized - invalid credentials", body = ErrorResponse)
    )
)]
#[instrument(skip(auth_service, payload))]
async fn login(
    State(auth_service): State<Arc<dyn AuthServiceTrait>>,
    Json(payload): Json<LoginRequest>,
) -> DomainResult<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;
    let ctx = Context::new();
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
}

/// Refresh access token
///
/// Use a refresh token to obtain a new access token and refresh token.
#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    tag = "auth",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Token refreshed successfully", body = LoginResponse),
        (status = 400, description = "Bad request - validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized - invalid refresh token", body = ErrorResponse)
    )
)]
#[instrument(skip(auth_service, payload))]
async fn refresh(
    State(auth_service): State<Arc<dyn AuthServiceTrait>>,
    Json(payload): Json<RefreshTokenRequest>,
) -> DomainResult<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;
    let ctx = Context::new();
    let tokens = auth_service.refresh(&ctx, &payload.refresh_token).await?;

    Ok((
        StatusCode::OK,
        Json(LoginResponse {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
        }),
    ))
}

/// Logout user
///
/// Invalidate a refresh token to log out the user.
#[utoipa::path(
    delete,
    path = "/api/auth",
    tag = "auth",
    request_body = LogoutRequest,
    responses(
        (status = 204, description = "Logout successful"),
        (status = 400, description = "Bad request - validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized - invalid refresh token", body = ErrorResponse)
    )
)]
#[instrument(skip(auth_service, payload))]
async fn logout(
    State(auth_service): State<Arc<dyn AuthServiceTrait>>,
    Json(payload): Json<LogoutRequest>,
) -> DomainResult<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;
    let ctx = Context::new();
    auth_service.logout(&ctx, &payload.refresh_token).await?;

    Ok(StatusCode::NO_CONTENT)
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
