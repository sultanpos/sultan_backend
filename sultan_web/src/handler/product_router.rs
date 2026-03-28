use axum::Extension;
use axum::extract::{Path, Query};
use axum::routing::get;
use axum::{
    Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::delete,
    routing::patch, routing::post,
};
use std::sync::Arc;
use sultan_core::application::ProductServiceTrait;
use sultan_core::domain::context::Context;
use sultan_core::domain::{DomainResult, Error};
use tracing::instrument;
use utoipa::OpenApi;
use validator::Validate;

use crate::AppState;
use crate::dto::category::CategoryChildResponse;
use crate::dto::product::{
    ProductFullCreateRequest, ProductListResponse, ProductQueryParams, ProductResponse,
    ProductUpdateRequest, ProductVariantCreateRequest, ProductVariantCreateResponse,
    ProductVariantResponse, SellDiscountResponse, SellPriceResponse,
};
use crate::dto::{ErrorResponse, ProductCreateResponse};

// ============================================================================
// OpenAPI Documentation
// ============================================================================

#[derive(OpenApi)]
#[openapi(
    paths(
        create,
        update_product,
        delete_product,
        get_by_id,
        get_all
    ),
    components(schemas(
        ProductFullCreateRequest,
        ProductUpdateRequest,
        ProductCreateResponse,
        ProductResponse,
        ProductListResponse,
        ProductQueryParams,
        ProductVariantCreateRequest,
        ProductVariantCreateResponse,
        ProductVariantResponse,
        SellPriceResponse,
        SellDiscountResponse,
        CategoryChildResponse,
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
// Product Handlers
// ============================================================================

/// Create a new product
///
/// Creates a new product with optional variants, sell prices, and stocks. Requires authentication.
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
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;

    let id = product_service
        .create_product(&ctx, &payload.to_domain())
        .await?;

    Ok((StatusCode::CREATED, Json(ProductCreateResponse { id })))
}

/// Update a product
///
/// Partially updates a product by ID. Fields absent from the JSON are left unchanged.
/// Nullable fields (`description`, `main_image`, `metadata`) can be cleared by sending `null`.
/// Providing `category_ids` replaces all existing category associations. Requires authentication.
#[utoipa::path(
    patch,
    path = "/api/product/{id}",
    tag = "product",
    params(
        ("id" = i64, Path, description = "Product ID")
    ),
    request_body = ProductUpdateRequest,
    responses(
        (status = 204, description = "Product updated successfully"),
        (status = 400, description = "Bad request - validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 404, description = "Product not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(product_service, payload, ctx))]
async fn update_product(
    State(product_service): State<Arc<dyn ProductServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path(id): Path<i64>,
    Json(payload): Json<ProductUpdateRequest>,
) -> DomainResult<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;

    product_service
        .update_product(&ctx, id, &payload.to_domain())
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Delete a product
///
/// Soft deletes a product and all its variants by ID. Requires authentication.
#[utoipa::path(
    delete,
    path = "/api/product/{id}",
    tag = "product",
    params(
        ("id" = i64, Path, description = "Product ID")
    ),
    responses(
        (status = 204, description = "Product deleted successfully"),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 404, description = "Product not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(product_service, ctx))]
async fn delete_product(
    State(product_service): State<Arc<dyn ProductServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path(id): Path<i64>,
) -> DomainResult<impl IntoResponse> {
    product_service.delete_product(&ctx, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Get a product by ID
///
/// Retrieves a single product by its ID. Requires authentication.
#[utoipa::path(
    get,
    path = "/api/product/{id}",
    tag = "product",
    params(
        ("id" = i64, Path, description = "Product ID")
    ),
    responses(
        (status = 200, description = "Product retrieved successfully", body = ProductResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 404, description = "Product not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(product_service, ctx))]
async fn get_by_id(
    State(product_service): State<Arc<dyn ProductServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path(id): Path<i64>,
) -> DomainResult<impl IntoResponse> {
    let product = product_service
        .get_by_id(&ctx, id)
        .await?
        .ok_or(Error::NotFound(format!("Product with id {} not found", id)))?;

    Ok((StatusCode::OK, Json(ProductResponse::from(product))))
}

/// List products
///
/// Returns a paginated list of products using cursor-based pagination.
/// Results are ordered by `(sort_field, id)` to guarantee stable ordering.
/// Pass `next_cursor` from the previous response as `cursor` to fetch the next page.
#[utoipa::path(
    get,
    path = "/api/product",
    tag = "product",
    params(ProductQueryParams),
    responses(
        (status = 200, description = "Products retrieved successfully", body = ProductListResponse),
        (status = 400, description = "Bad request - invalid query parameters", body = ErrorResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(product_service, ctx, params))]
async fn get_all(
    State(product_service): State<Arc<dyn ProductServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Query(params): Query<ProductQueryParams>,
) -> DomainResult<impl IntoResponse> {
    let query = params.to_query()?;

    let page = product_service.get_all_products(&ctx, &query).await?;

    Ok((
        StatusCode::OK,
        Json(ProductListResponse::from_cursor_page(page)),
    ))
}

// ============================================================================
// Router
// ============================================================================

pub fn product_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create))
        .route("/", get(get_all))
        .route("/{id}", patch(update_product))
        .route("/{id}", delete(delete_product))
        .route("/{id}", get(get_by_id))
}
