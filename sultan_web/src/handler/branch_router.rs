use axum::Extension;
use axum::extract::{Path, Query};
use axum::routing::get;
use axum::{
    Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::delete,
    routing::post, routing::put,
};
use std::sync::Arc;
use sultan_core::application::BranchServiceTrait;
use sultan_core::domain::context::Context;
use sultan_core::domain::model::branch::{BranchCreate, BranchUpdate};
use sultan_core::domain::{DomainResult, Error};
use tracing::instrument;
use utoipa::OpenApi;
use validator::Validate;

use crate::AppState;
use crate::dto::branch::{
    BranchCreateResponse, BranchListResponse, BranchQueryParams, BranchResponse,
    BranchUpdateRequest,
};
use crate::dto::{BranchCreateRequest, ErrorResponse};

// ============================================================================
// OpenAPI Documentation
// ============================================================================

#[derive(OpenApi)]
#[openapi(
    paths(create, update, delete_branch, get_by_id, get_all),
    components(schemas(
        BranchCreateRequest,
        BranchCreateResponse,
        BranchUpdateRequest,
        BranchResponse,
        BranchQueryParams,
        BranchListResponse,
        ErrorResponse,
    )),
    tags(
        (name = "branch", description = "Branch management endpoints")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub struct BranchApiDoc;

#[utoipa::path(
    post,
    operation_id = "create_branch",
    path = "/api/branch",
    tag = "branch",
    request_body = BranchCreateRequest,
    responses(
        (status = 201, description = "Branch created successfully", body = BranchCreateResponse),
        (status = 400, description = "Bad request - validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(branch_service, payload, ctx))]
async fn create(
    State(branch_service): State<Arc<dyn BranchServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Json(payload): Json<BranchCreateRequest>,
) -> DomainResult<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;

    let id = branch_service
        .create(
            &ctx,
            &BranchCreate {
                is_main: payload.is_main,
                name: payload.name,
                code: payload.code,
                address: payload.address,
                phone: payload.phone,
                npwp: payload.npwp,
                image: payload.image,
            },
        )
        .await?;

    Ok((StatusCode::CREATED, Json(BranchCreateResponse { id })))
}

#[utoipa::path(
    put,
    operation_id = "update_branch",
    path = "/api/branch/{id}",
    tag = "branch",
    request_body = BranchUpdateRequest,
    params(
        ("id" = i64, Path, description = "Branch ID to update")
    ),
    responses(
        (status = 204, description = "Branch updated successfully"),
        (status = 400, description = "Bad request - validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 404, description = "Branch not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(branch_service, payload, ctx))]
async fn update(
    State(branch_service): State<Arc<dyn BranchServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path(id): Path<i64>,
    Json(payload): Json<BranchUpdateRequest>,
) -> DomainResult<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;

    branch_service
        .update(
            &ctx,
            id,
            &BranchUpdate {
                is_main: payload.is_main,
                name: payload.name,
                code: payload.code,
                address: payload.address,
                phone: payload.phone,
                npwp: payload.npwp,
                image: payload.image,
            },
        )
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/branch/{id}",
    tag = "branch",
    params(
        ("id" = i64, Path, description = "Branch ID to delete")
    ),
    responses(
        (status = 204, description = "Branch deleted successfully"),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 404, description = "Branch not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(branch_service, ctx))]
async fn delete_branch(
    State(branch_service): State<Arc<dyn BranchServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path(id): Path<i64>,
) -> DomainResult<impl IntoResponse> {
    branch_service.delete(&ctx, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    operation_id = "get_branch",
    path = "/api/branch/{id}",
    tag = "branch",
    params(
        ("id" = i64, Path, description = "Branch ID to retrieve")
    ),
    responses(
        (status = 200, description = "Branch retrieved successfully", body = BranchResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 404, description = "Branch not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(branch_service, ctx))]
async fn get_by_id(
    State(branch_service): State<Arc<dyn BranchServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path(id): Path<i64>,
) -> DomainResult<impl IntoResponse> {
    let branch = branch_service
        .get_by_id(&ctx, id)
        .await?
        .ok_or(Error::NotFound(format!("Branch with id {} not found", id)))?;
    Ok((StatusCode::OK, Json(BranchResponse::from(branch))))
}

#[utoipa::path(
    get,
    operation_id = "list_branches",
    path = "/api/branch",
    tag = "branch",
    params(BranchQueryParams),
    responses(
        (status = 200, description = "Branches retrieved successfully", body = BranchListResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(branch_service, ctx, params))]
async fn get_all(
    State(branch_service): State<Arc<dyn BranchServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Query(params): Query<BranchQueryParams>,
) -> DomainResult<impl IntoResponse> {
    let query = params.to_query()?;
    let page = branch_service.get_all(&ctx, &query).await?;
    Ok((StatusCode::OK, Json(BranchListResponse::from_page(page))))
}

// ============================================================================
// Router
// ============================================================================

pub fn branch_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create))
        .route("/{id}", put(update))
        .route("/{id}", delete(delete_branch))
        .route("/{id}", get(get_by_id))
        .route("/", get(get_all))
}
