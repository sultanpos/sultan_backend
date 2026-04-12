use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use std::sync::Arc;
use sultan_core::application::PaymentChannelServiceTrait;
use sultan_core::domain::context::Context;
use sultan_core::domain::model::payment_channel::{
    PaymentChannelCreate, PaymentChannelFilter, PaymentChannelPriorityUpdate, PaymentChannelUpdate,
};
use sultan_core::domain::{DomainResult, Error};
use tracing::instrument;
use utoipa::OpenApi;
use validator::Validate;

use crate::AppState;
use crate::dto::payment_channel::{
    BulkPriorityUpdateRequest, PaymentChannelCreateRequest, PaymentChannelCreateResponse,
    PaymentChannelQueryParams, PaymentChannelResponse, PaymentChannelUpdateRequest,
};
use crate::dto::{ErrorResponse, ListResponse};

// ============================================================================
// OpenAPI Documentation
// ============================================================================

#[derive(OpenApi)]
#[openapi(
    paths(create, update, delete_channel, get_by_id, get_all, update_priorities),
    components(schemas(
        PaymentChannelCreateRequest,
        PaymentChannelCreateResponse,
        PaymentChannelUpdateRequest,
        PaymentChannelResponse,
        PaymentChannelQueryParams,
        BulkPriorityUpdateRequest,
        ListResponse<PaymentChannelResponse>,
        ErrorResponse,
    )),
    tags(
        (name = "payment-channel", description = "Payment channel management endpoints")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub struct PaymentChannelApiDoc;

// ============================================================================
// HTTP Handlers
// ============================================================================

/// Create a new payment channel
#[utoipa::path(
    post,
    path = "/api/payment-channel",
    tag = "payment-channel",
    request_body = PaymentChannelCreateRequest,
    responses(
        (status = 201, description = "Payment channel created successfully", body = PaymentChannelCreateResponse),
        (status = 400, description = "Bad request - validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
#[instrument(skip(service, payload, ctx))]
async fn create(
    State(service): State<Arc<dyn PaymentChannelServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Json(payload): Json<PaymentChannelCreateRequest>,
) -> DomainResult<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;

    let id = service
        .create(
            &ctx,
            &PaymentChannelCreate {
                branch_id: payload.branch_id,
                name: payload.name,
                priority: payload.priority,
                metadata: payload.metadata,
            },
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(PaymentChannelCreateResponse { id }),
    ))
}

/// Update an existing payment channel
#[utoipa::path(
    put,
    path = "/api/payment-channel/{id}",
    tag = "payment-channel",
    request_body = PaymentChannelUpdateRequest,
    params(("id" = i64, Path, description = "Payment channel ID")),
    responses(
        (status = 204, description = "Updated successfully"),
        (status = 400, description = "Bad request - validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
#[instrument(skip(service, payload, ctx))]
async fn update(
    State(service): State<Arc<dyn PaymentChannelServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path(id): Path<i64>,
    Json(payload): Json<PaymentChannelUpdateRequest>,
) -> DomainResult<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;

    service
        .update(
            &ctx,
            id,
            &PaymentChannelUpdate {
                name: payload.name,
                priority: payload.priority,
                metadata: payload.metadata,
            },
        )
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Delete a payment channel
#[utoipa::path(
    delete,
    path = "/api/payment-channel/{id}",
    tag = "payment-channel",
    params(("id" = i64, Path, description = "Payment channel ID")),
    responses(
        (status = 204, description = "Deleted successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
#[instrument(skip(service, ctx))]
async fn delete_channel(
    State(service): State<Arc<dyn PaymentChannelServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path(id): Path<i64>,
) -> DomainResult<impl IntoResponse> {
    service.delete(&ctx, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Get a payment channel by ID
#[utoipa::path(
    get,
    path = "/api/payment-channel/{id}",
    tag = "payment-channel",
    params(("id" = i64, Path, description = "Payment channel ID")),
    responses(
        (status = 200, description = "Payment channel found", body = PaymentChannelResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
#[instrument(skip(service, ctx))]
async fn get_by_id(
    State(service): State<Arc<dyn PaymentChannelServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path(id): Path<i64>,
) -> DomainResult<impl IntoResponse> {
    match service.get_by_id(&ctx, id).await? {
        Some(channel) => Ok((StatusCode::OK, Json(PaymentChannelResponse::from(channel)))),
        None => Err(Error::NotFound(format!(
            "Payment channel with id {} not found",
            id
        ))),
    }
}

/// List all payment channels
#[utoipa::path(
    get,
    path = "/api/payment-channel",
    tag = "payment-channel",
    params(PaymentChannelQueryParams),
    responses(
        (status = 200, description = "List of payment channels", body = ListResponse<PaymentChannelResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
#[instrument(skip(service, ctx))]
async fn get_all(
    State(service): State<Arc<dyn PaymentChannelServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Query(params): Query<PaymentChannelQueryParams>,
) -> DomainResult<impl IntoResponse> {
    let channels = service
        .get_all(
            &ctx,
            &PaymentChannelFilter {
                branch_id: params.branch_id,
                name: params.name,
            },
        )
        .await?;

    Ok((
        StatusCode::OK,
        Json(ListResponse {
            data: channels
                .into_iter()
                .map(PaymentChannelResponse::from)
                .collect(),
        }),
    ))
}

/// Bulk-update display priorities
#[utoipa::path(
    put,
    path = "/api/payment-channel/priorities",
    tag = "payment-channel",
    request_body = BulkPriorityUpdateRequest,
    responses(
        (status = 204, description = "Priorities updated successfully"),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
#[instrument(skip(service, payload, ctx))]
async fn update_priorities(
    State(service): State<Arc<dyn PaymentChannelServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Json(payload): Json<BulkPriorityUpdateRequest>,
) -> DomainResult<impl IntoResponse> {
    let updates: Vec<PaymentChannelPriorityUpdate> = payload
        .channels
        .into_iter()
        .map(|item| PaymentChannelPriorityUpdate {
            id: item.id,
            priority: item.priority,
        })
        .collect();

    service.update_priorities(&ctx, &updates).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Router
// ============================================================================

pub fn payment_channel_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create))
        .route("/", get(get_all))
        .route("/priorities", put(update_priorities))
        .route("/{id}", get(get_by_id))
        .route("/{id}", put(update))
        .route("/{id}", delete(delete_channel))
}
