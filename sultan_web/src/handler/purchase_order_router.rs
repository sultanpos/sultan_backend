use axum::extract::Path;
use axum::{
    Extension, Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use std::sync::Arc;
use sultan_core::application::PurchaseOrderServiceTrait;
use sultan_core::domain::context::Context;
use sultan_core::domain::model::purchase_order::{PurchaseOrderCreate, PurchaseOrderUpdate};
use sultan_core::domain::{DomainResult, Error};
use tracing::instrument;
use utoipa::OpenApi;
use validator::Validate;

use crate::AppState;
use crate::dto::ErrorResponse;
use crate::dto::purchase_order::{
    PurchaseOrderCreateRequest, PurchaseOrderCreateResponse, PurchaseOrderResponse,
    PurchaseOrderUpdateRequest,
};

// ============================================================================
// OpenAPI Documentation
// ============================================================================

#[derive(OpenApi)]
#[openapi(
    paths(create, update, delete_purchase_order, get_purchase_order),
    components(schemas(
        PurchaseOrderCreateRequest,
        PurchaseOrderCreateResponse,
        PurchaseOrderUpdateRequest,
        PurchaseOrderResponse,
        ErrorResponse,
    )),
    tags(
        (name = "purchase_order", description = "Purchase order management endpoints")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub struct PurchaseOrderApiDoc;

// ============================================================================
// HTTP Handlers
// ============================================================================

/// Create a new purchase order
///
/// Creates a new purchase order (header only). Line items can be added separately.
#[utoipa::path(
    post,
    operation_id = "create_purchase_order",
    path = "/api/branch/{branch_id}/purchase-order",
    tag = "purchase_order",
    request_body = PurchaseOrderCreateRequest,
    responses(
        (status = 201, description = "Purchase order created successfully", body = PurchaseOrderCreateResponse),
        (status = 400, description = "Bad request - validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 403, description = "Forbidden - insufficient permissions", body = ErrorResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(purchase_order_service, payload, ctx))]
async fn create(
    State(purchase_order_service): State<Arc<dyn PurchaseOrderServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path(branch_id): Path<i64>,
    Json(payload): Json<PurchaseOrderCreateRequest>,
) -> DomainResult<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;

    let id = purchase_order_service
        .create(
            &ctx,
            &PurchaseOrderCreate {
                number: "".to_string(),
                branch_id,
                supplier_id: payload.supplier_id,
                reference_number: payload.reference_number,
                order_date: payload.order_date,
                expected_date: payload.expected_date,
                payment_due_date: payload.payment_due_date,
                discount_amount: payload.discount_amount.unwrap_or(0),
                notes: payload.notes,
                metadata: payload.metadata,
            },
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(PurchaseOrderCreateResponse { id }),
    ))
}

/// Update an existing purchase order (header fields only).
#[utoipa::path(
    put,
    operation_id = "update_purchase_order",
    path = "/api/branch/{branch_id}/purchase-order/{id}",
    tag = "purchase_order",
    request_body = PurchaseOrderUpdateRequest,
    params(
        ("branch_id" = i64, Path, description = "Branch ID"),
        ("id" = i64, Path, description = "Purchase order ID")
    ),
    responses(
        (status = 204, description = "Purchase order updated successfully"),
        (status = 400, description = "Bad request - validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 403, description = "Forbidden - insufficient permissions", body = ErrorResponse),
        (status = 404, description = "Purchase order not found", body = ErrorResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn update(
    State(purchase_order_service): State<Arc<dyn PurchaseOrderServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path((branch_id, id)): Path<(i64, i64)>,
    Json(payload): Json<PurchaseOrderUpdateRequest>,
) -> DomainResult<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format! {"{}", e}))?;

    purchase_order_service
        .update(
            &ctx,
            branch_id,
            id,
            &PurchaseOrderUpdate {
                supplier_id: payload.supplier_id,
                reference_number: payload.reference_number,

                ..Default::default()
            },
        )
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Soft-delete a purchase order by ID.
#[utoipa::path(
    delete,
    operation_id = "delete_purchase_order",
    path = "/api/branch/{branch_id}/purchase-order/{id}",
    tag = "purchase_order",
    params(
        ("branch_id" = i64, Path, description = "Branch ID"),
        ("id" = i64, Path, description = "Purchase order ID")
    ),
    responses(
        (status = 204, description = "Purchase order deleted successfully"),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 403, description = "Forbidden - insufficient permissions", body = ErrorResponse),
        (status = 404, description = "Purchase order not found", body = ErrorResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn delete_purchase_order(
    State(purchase_order_service): State<Arc<dyn PurchaseOrderServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path((branch_id, id)): Path<(i64, i64)>,
) -> DomainResult<impl IntoResponse> {
    purchase_order_service.delete(&ctx, branch_id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Get a purchase order by ID.
#[utoipa::path(
    get,
    operation_id = "get_purchase_order",
    path = "/api/branch/{branch_id}/purchase-order/{id}",
    tag = "purchase_order",
    params(
        ("branch_id" = i64, Path, description = "Branch ID"),
        ("id" = i64, Path, description = "Purchase order ID")
    ),
    responses(
        (status = 200, description = "Purchase order found", body = PurchaseOrderResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 403, description = "Forbidden - insufficient permissions", body = ErrorResponse),
        (status = 404, description = "Purchase order not found", body = ErrorResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn get_purchase_order(
    State(purchase_order_service): State<Arc<dyn PurchaseOrderServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path((branch_id, id)): Path<(i64, i64)>,
) -> DomainResult<impl IntoResponse> {
    let result = purchase_order_service
        .get(&ctx, branch_id, id)
        .await?
        .ok_or(Error::NotFound(format!(
            "Purchase order branch {} with id {} not found",
            branch_id, id
        )))?;
    Ok((StatusCode::OK, Json(PurchaseOrderResponse::from(result))))
}

// ============================================================================
// Router
// ============================================================================

pub fn purchase_order_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create))
        .route("/{id}", put(update))
        .route("/{id}", delete(delete_purchase_order))
        .route("/{id}", get(get_purchase_order))
}
