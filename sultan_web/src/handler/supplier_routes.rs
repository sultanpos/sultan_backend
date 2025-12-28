use axum::Extension;
use axum::extract::{Path, Query};
use axum::routing::get;
use axum::{
    Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::delete,
    routing::post, routing::put,
};
use std::sync::Arc;
use sultan_core::application::SupplierServiceTrait;
use sultan_core::domain::context::Context;
use sultan_core::domain::model::supplier::{SupplierCreate, SupplierUpdate};
use sultan_core::domain::{DomainResult, Error};
use tracing::instrument;
use utoipa::OpenApi;
use validator::Validate;

use crate::AppState;
use crate::dto::supplier::{SupplierQueryParams, SupplierResponse, SupplierUpdateRequest};
use crate::dto::{ErrorResponse, ListResponse, SupplierCreateRequest, SupplierCreateResponse};

// ============================================================================
// OpenAPI Documentation
// ============================================================================

#[derive(OpenApi)]
#[openapi(
    paths(create, update, delete_supplier, get_one, get_many),
    components(schemas(
        SupplierCreateRequest,
        SupplierCreateResponse,
        SupplierUpdateRequest,
        SupplierResponse,
        SupplierQueryParams,
        ListResponse<SupplierResponse>,
        ErrorResponse,
    )),
    tags(
        (name = "supplier", description = "Supplier management endpoints")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub struct SupplierApiDoc;

// ============================================================================
// HTTP Handlers
// ============================================================================

/// Create a new supplier
///
/// Creates a new supplier with the provided information. Requires authentication.
#[utoipa::path(
    post,
    path = "/api/supplier",
    tag = "supplier",
    request_body = SupplierCreateRequest,
    responses(
        (status = 201, description = "Supplier created successfully", body = SupplierCreateResponse),
        (status = 400, description = "Bad request - validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(supplier_service, payload, ctx))]
async fn create(
    State(supplier_service): State<Arc<dyn SupplierServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Json(payload): Json<SupplierCreateRequest>,
) -> DomainResult<impl IntoResponse> {
    // Validate input
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;

    let id = supplier_service
        .create(
            &ctx,
            &SupplierCreate {
                name: payload.name,
                code: payload.code,
                email: payload.email,
                address: payload.address,
                phone: payload.phone,
                npwp: payload.npwp,
                npwp_name: payload.npwp_name,
                metadata: payload.metadata,
            },
        )
        .await?;

    Ok((StatusCode::CREATED, Json(SupplierCreateResponse { id })))
}

/// Update an existing supplier
///
/// Updates a supplier's information by ID. Requires authentication.
#[utoipa::path(
    put,
    path = "/api/supplier/{id}",
    tag = "supplier",
    params(
        ("id" = i64, Path, description = "Supplier ID")
    ),
    request_body = SupplierUpdateRequest,
    responses(
        (status = 204, description = "Supplier updated successfully"),
        (status = 400, description = "Bad request - validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 404, description = "Supplier not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn update(
    State(supplier_service): State<Arc<dyn SupplierServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path(id): Path<i64>,
    Json(payload): Json<SupplierUpdateRequest>,
) -> DomainResult<impl IntoResponse> {
    // Validate input
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;

    supplier_service
        .update(
            &ctx,
            id,
            &SupplierUpdate {
                name: payload.name,
                code: payload.code,
                email: payload.email,
                address: payload.address,
                phone: payload.phone,
                npwp: payload.npwp,
                npwp_name: payload.npwp_name,
                metadata: payload.metadata,
            },
        )
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Delete a supplier
///
/// Deletes a supplier by ID. Requires authentication.
#[utoipa::path(
    delete,
    path = "/api/supplier/{id}",
    tag = "supplier",
    params(
        ("id" = i64, Path, description = "Supplier ID")
    ),
    responses(
        (status = 204, description = "Supplier deleted successfully"),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 404, description = "Supplier not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn delete_supplier(
    State(supplier_service): State<Arc<dyn SupplierServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path(id): Path<i64>,
) -> DomainResult<impl IntoResponse> {
    supplier_service.delete(&ctx, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Get a supplier by ID
///
/// Retrieves a single supplier by its ID. Requires authentication.
#[utoipa::path(
    get,
    path = "/api/supplier/{id}",
    tag = "supplier",
    params(
        ("id" = i64, Path, description = "Supplier ID")
    ),
    responses(
        (status = 200, description = "Supplier retrieved successfully", body = SupplierResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 404, description = "Supplier not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn get_one(
    State(supplier_service): State<Arc<dyn SupplierServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path(id): Path<i64>,
) -> DomainResult<impl IntoResponse> {
    let supplier = supplier_service
        .get_by_id(&ctx, id)
        .await?
        .ok_or(Error::NotFound(format!(
            "Supplier with id {} not found",
            id
        )))?;

    Ok((StatusCode::OK, Json(SupplierResponse::from(supplier))))
}

/// Get all suppliers
///
/// Retrieves a list of suppliers with optional filtering and pagination. Requires authentication.
#[utoipa::path(
    get,
    path = "/api/supplier",
    tag = "supplier",
    params(SupplierQueryParams),
    responses(
        (status = 200, description = "Suppliers retrieved successfully", body = ListResponse<SupplierResponse>),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn get_many(
    State(supplier_service): State<Arc<dyn SupplierServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Query(params): Query<SupplierQueryParams>,
) -> DomainResult<impl IntoResponse> {
    let supplier = supplier_service
        .get_all(&ctx, &params.to_filter(), &params.to_pagination())
        .await?;

    Ok((
        StatusCode::OK,
        Json(ListResponse {
            data: supplier.into_iter().map(SupplierResponse::from).collect(),
        }),
    ))
}

// ============================================================================
// Router
// ============================================================================

pub fn supplier_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create))
        .route("/{id}", put(update))
        .route("/{id}", delete(delete_supplier))
        .route("/{id}", get(get_one))
        .route("/", get(get_many))
}
