use axum::Extension;
use axum::extract::{Path, Query};
use axum::routing::get;
use axum::{
    Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::delete,
    routing::post, routing::put,
};
use std::sync::Arc;
use sultan_core::application::MachineServiceTrait;
use sultan_core::domain::context::Context;
use sultan_core::domain::model::machine::{MachineCreate, MachineUpdate};
use sultan_core::domain::{DomainResult, Error};
use tracing::instrument;
use utoipa::OpenApi;
use validator::Validate;

use crate::AppState;
use crate::dto::ErrorResponse;
use crate::dto::machine::{
    MachineCreateRequest, MachineCreateResponse, MachineListResponse, MachineQueryParams,
    MachineResponse, MachineUpdateRequest,
};

// ============================================================================
// OpenAPI Documentation
// ============================================================================

#[derive(OpenApi)]
#[openapi(
    paths(create, update, delete_machine, get_one, get_many),
    components(schemas(
        MachineCreateRequest,
        MachineCreateResponse,
        MachineUpdateRequest,
        MachineResponse,
        MachineQueryParams,
        MachineListResponse,
        ErrorResponse,
    )),
    tags(
        (name = "machine", description = "POS machine management endpoints")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub struct MachineApiDoc;

// ============================================================================
// HTTP Handlers
// ============================================================================

/// Create a new machine
///
/// Registers a new POS machine for a branch. The `key` is unique per branch and immutable.
#[utoipa::path(
    post,
    path = "/api/machine",
    tag = "machine",
    request_body = MachineCreateRequest,
    responses(
        (status = 201, description = "Machine created successfully", body = MachineCreateResponse),
        (status = 400, description = "Bad request - validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 409, description = "Conflict - machine key already exists in this branch", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(machine_service, payload, ctx))]
async fn create(
    State(machine_service): State<Arc<dyn MachineServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Json(payload): Json<MachineCreateRequest>,
) -> DomainResult<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;

    let id = machine_service
        .create(
            &ctx,
            &MachineCreate {
                branch_id: payload.branch_id,
                key: payload.key,
                name: payload.name,
                description: payload.description,
                metadata: payload.metadata,
            },
        )
        .await?;

    Ok((StatusCode::CREATED, Json(MachineCreateResponse { id })))
}

/// Update an existing machine
///
/// Updates a machine's name, description, or metadata. The `key` field cannot be changed.
#[utoipa::path(
    put,
    path = "/api/machine/{id}",
    tag = "machine",
    params(
        ("id" = i64, Path, description = "Machine ID")
    ),
    request_body = MachineUpdateRequest,
    responses(
        (status = 204, description = "Machine updated successfully"),
        (status = 400, description = "Bad request - validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 404, description = "Machine not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn update(
    State(machine_service): State<Arc<dyn MachineServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path(id): Path<i64>,
    Json(payload): Json<MachineUpdateRequest>,
) -> DomainResult<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;

    machine_service
        .update(
            &ctx,
            id,
            &MachineUpdate {
                name: payload.name,
                description: payload.description,
                metadata: payload.metadata,
            },
        )
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Delete a machine
///
/// Soft-deletes a machine by ID.
#[utoipa::path(
    delete,
    path = "/api/machine/{id}",
    tag = "machine",
    params(
        ("id" = i64, Path, description = "Machine ID")
    ),
    responses(
        (status = 204, description = "Machine deleted successfully"),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 404, description = "Machine not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn delete_machine(
    State(machine_service): State<Arc<dyn MachineServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path(id): Path<i64>,
) -> DomainResult<impl IntoResponse> {
    machine_service.delete(&ctx, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Get a machine by ID
///
/// Retrieves a single machine by its ID.
#[utoipa::path(
    get,
    path = "/api/machine/{id}",
    tag = "machine",
    params(
        ("id" = i64, Path, description = "Machine ID")
    ),
    responses(
        (status = 200, description = "Machine retrieved successfully", body = MachineResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 404, description = "Machine not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn get_one(
    State(machine_service): State<Arc<dyn MachineServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path(id): Path<i64>,
) -> DomainResult<impl IntoResponse> {
    let machine = machine_service
        .get_by_id(&ctx, id)
        .await?
        .ok_or_else(|| Error::NotFound(format!("Machine with id {} not found", id)))?;

    Ok((StatusCode::OK, Json(MachineResponse::from(machine))))
}

/// List machines
///
/// Retrieves a paginated list of machines using cursor-based pagination.
#[utoipa::path(
    get,
    path = "/api/machine",
    tag = "machine",
    params(MachineQueryParams),
    responses(
        (status = 200, description = "Machines retrieved successfully", body = MachineListResponse),
        (status = 400, description = "Bad request - invalid query params", body = ErrorResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn get_many(
    State(machine_service): State<Arc<dyn MachineServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Query(params): Query<MachineQueryParams>,
) -> DomainResult<impl IntoResponse> {
    let query = params.to_query()?;
    let page = machine_service.get_all(&ctx, &query).await?;
    Ok((StatusCode::OK, Json(MachineListResponse::from_page(page))))
}

// ============================================================================
// Router
// ============================================================================

pub fn machine_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create))
        .route("/", get(get_many))
        .route("/{id}", put(update))
        .route("/{id}", delete(delete_machine))
        .route("/{id}", get(get_one))
}
