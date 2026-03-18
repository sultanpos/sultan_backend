use axum::Extension;
use axum::extract::{Path, Query};
use axum::routing::get;
use axum::{
    Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::delete,
    routing::post, routing::put,
};
use std::sync::Arc;
use sultan_core::application::ProductServiceTrait;
use sultan_core::domain::context::Context;
use sultan_core::domain::model::product::Product;
use sultan_core::domain::{DomainResult, Error};
use tracing::instrument;
use utoipa::OpenApi;
use validator::Validate;

use crate::AppState;
use crate::dto::product::ProductFullCreateRequest;
use crate::dto::{
    ErrorResponse, ListResponse, ProductCreateResponse, SupplierCreateRequest,
    SupplierCreateResponse,
};

// ============================================================================
// OpenAPI Documentation
// ============================================================================

#[derive(OpenApi)]
#[openapi(
    paths(create),
    components(schemas(
        ProductFullCreateRequest,
        ProductCreateResponse,
        ErrorResponse,
    )),
    tags(
        (name = "product", description = "Product management endpoints")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub struct ProductApiDoc;

// ============================================================================
// HTTP Handlers
// ============================================================================

/// Create a new supplier
///
/// Creates a new supplier with the provided information. Requires authentication.
#[utoipa::path(
    post,
    path = "/api/product",
    tag = "product",
    request_body = ProductFullCreateRequest,
    responses(
        (status = 201, description = "Product created successfully", body = ProductCreateResponse),
        (status = 400, description = "Bad request - validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(product_service, payload, ctx))]
async fn create(
    State(product_service): State<Arc<dyn ProductServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Json(payload): Json<ProductFullCreateRequest>,
) -> DomainResult<impl IntoResponse> {
    // Validate input
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;

    let id = product_service
        .create_product(&ctx, &payload.to_domain())
        .await?;

    Ok((StatusCode::CREATED, Json(ProductCreateResponse { id })))
}
