use axum::Extension;
use axum::extract::{Path, Query};
use axum::routing::get;
use axum::{
    Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::delete,
    routing::post, routing::put,
};
use std::sync::Arc;
use sultan_core::application::CashierSessionServiceTrait;
use sultan_core::domain::context::Context;
use sultan_core::domain::model::cashier_session::{CashierSessionClose, CashierSessionCreate};
use sultan_core::domain::{DomainResult, Error};
use tracing::instrument;
use utoipa::OpenApi;
use validator::Validate;

use crate::AppState;
use crate::dto::ErrorResponse;
use crate::dto::cashier_session::{
    CashierSessionListResponse, CashierSessionQueryParams, CashierSessionResponse,
    CloseSessionRequest, OpenSessionRequest, OpenSessionResponse,
};

// ============================================================================
// OpenAPI Documentation
// ============================================================================

#[derive(OpenApi)]
#[openapi(
    paths(open_session, close_session, delete_session, get_one, get_many, get_current),
    components(schemas(
        OpenSessionRequest,
        OpenSessionResponse,
        CloseSessionRequest,
        CashierSessionResponse,
        CashierSessionQueryParams,
        CashierSessionListResponse,
        ErrorResponse,
    )),
    tags(
        (name = "cashier_session", description = "Cashier session management endpoints")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub struct CashierSessionApiDoc;

// ============================================================================
// HTTP Handlers
// ============================================================================

/// Open a new cashier session
///
/// Opens a new cashier session for a user on a branch. Returns validation error if
/// the user already has an open session on that branch.
#[utoipa::path(
    post,
    path = "/api/cashier-session",
    tag = "cashier_session",
    request_body = OpenSessionRequest,
    responses(
        (status = 201, description = "Session opened successfully", body = OpenSessionResponse),
        (status = 400, description = "Bad request - validation error or session already open", body = ErrorResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(cashier_session_service, payload, ctx))]
async fn open_session(
    State(cashier_session_service): State<Arc<dyn CashierSessionServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Json(payload): Json<OpenSessionRequest>,
) -> DomainResult<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;

    let user_id = ctx
        .user_id()
        .ok_or_else(|| Error::Unauthorized("User not authenticated".to_string()))?;

    let id = cashier_session_service
        .open_session(
            &ctx,
            &CashierSessionCreate {
                branch_id: payload.branch_id,
                user_id,
                opening_cash: payload.opening_cash,
                notes: payload.notes,
            },
        )
        .await?;

    Ok((StatusCode::CREATED, Json(OpenSessionResponse { id })))
}

/// Close a cashier session
///
/// Closes an open cashier session by ID. Returns not found if session doesn't exist
/// or is already closed.
#[utoipa::path(
    put,
    path = "/api/cashier-session/{id}/close",
    tag = "cashier_session",
    params(
        ("id" = i64, Path, description = "Session ID")
    ),
    request_body = CloseSessionRequest,
    responses(
        (status = 204, description = "Session closed successfully"),
        (status = 400, description = "Bad request - validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 404, description = "Session not found or already closed", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(cashier_session_service, payload, ctx))]
async fn close_session(
    State(cashier_session_service): State<Arc<dyn CashierSessionServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path(id): Path<i64>,
    Json(payload): Json<CloseSessionRequest>,
) -> DomainResult<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;

    cashier_session_service
        .close_session(
            &ctx,
            id,
            &CashierSessionClose {
                closing_cash: payload.closing_cash,
                notes: payload.notes,
            },
        )
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Delete a cashier session
///
/// Soft-deletes a cashier session by ID.
#[utoipa::path(
    delete,
    path = "/api/cashier-session/{id}",
    tag = "cashier_session",
    params(
        ("id" = i64, Path, description = "Session ID")
    ),
    responses(
        (status = 204, description = "Session deleted successfully"),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 404, description = "Session not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn delete_session(
    State(cashier_session_service): State<Arc<dyn CashierSessionServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path(id): Path<i64>,
) -> DomainResult<impl IntoResponse> {
    cashier_session_service.delete(&ctx, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Get a cashier session by ID
#[utoipa::path(
    get,
    path = "/api/cashier-session/{id}",
    tag = "cashier_session",
    params(
        ("id" = i64, Path, description = "Session ID")
    ),
    responses(
        (status = 200, description = "Session retrieved successfully", body = CashierSessionResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 404, description = "Session not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn get_one(
    State(cashier_session_service): State<Arc<dyn CashierSessionServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path(id): Path<i64>,
) -> DomainResult<impl IntoResponse> {
    let session = cashier_session_service
        .get_by_id(&ctx, id)
        .await?
        .ok_or_else(|| Error::NotFound(format!("Cashier session with id {} not found", id)))?;

    Ok((StatusCode::OK, Json(CashierSessionResponse::from(session))))
}

/// Get the current open session for a user on a branch
///
/// Returns the currently open cashier session for the authenticated user on the given branch,
/// or 404 if none exists.
#[utoipa::path(
    get,
    path = "/api/cashier-session/current",
    tag = "cashier_session",
    params(
        ("branch_id" = String, Query, description = "Branch ID"),
    ),
    responses(
        (status = 200, description = "Current session retrieved", body = CashierSessionResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 404, description = "No open session found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn get_current(
    State(cashier_session_service): State<Arc<dyn CashierSessionServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Query(params): Query<CurrentSessionParams>,
) -> DomainResult<impl IntoResponse> {
    let user_id = ctx
        .user_id()
        .ok_or_else(|| Error::Unauthorized("User not authenticated".to_string()))?;
    let session = cashier_session_service
        .get_current_session(&ctx, params.branch_id, user_id)
        .await?
        .ok_or_else(|| {
            Error::NotFound(
                "No open cashier session found for this user on this branch".to_string(),
            )
        })?;

    Ok((StatusCode::OK, Json(CashierSessionResponse::from(session))))
}

#[derive(Debug, serde::Deserialize)]
struct CurrentSessionParams {
    #[serde(deserialize_with = "crate::dto::string_to_i64")]
    branch_id: i64,
}

/// List cashier sessions
///
/// Retrieves a paginated list of cashier sessions using cursor-based pagination.
#[utoipa::path(
    get,
    path = "/api/cashier-session",
    tag = "cashier_session",
    params(CashierSessionQueryParams),
    responses(
        (status = 200, description = "Sessions retrieved successfully", body = CashierSessionListResponse),
        (status = 400, description = "Bad request - invalid query params", body = ErrorResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn get_many(
    State(cashier_session_service): State<Arc<dyn CashierSessionServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Query(params): Query<CashierSessionQueryParams>,
) -> DomainResult<impl IntoResponse> {
    let query = params.to_query()?;
    let page = cashier_session_service.get_all(&ctx, &query).await?;
    Ok((
        StatusCode::OK,
        Json(CashierSessionListResponse::from_page(page)),
    ))
}

// ============================================================================
// Router
// ============================================================================

pub fn cashier_session_router() -> Router<AppState> {
    Router::new()
        .route("/", post(open_session))
        .route("/", get(get_many))
        .route("/current", get(get_current))
        .route("/{id}", get(get_one))
        .route("/{id}/close", put(close_session))
        .route("/{id}", delete(delete_session))
}
