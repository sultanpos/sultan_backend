use axum::Extension;
use axum::extract::Path;
use axum::routing::get;
use axum::{
    Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::delete,
    routing::post, routing::put,
};
use std::sync::Arc;
use sultan_core::application::CategoryServiceTrait;
use sultan_core::domain::context::Context;
use sultan_core::domain::model::category::{CategoryCreate, CategoryUpdate};
use sultan_core::domain::{DomainResult, Error};
use tracing::instrument;
use utoipa::OpenApi;
use validator::Validate;

use crate::AppState;
use crate::dto::category::{CategoryChildResponse, CategoryResponse, CategoryUpdateRequest};
use crate::dto::{CategoryCreateRequest, CategoryCreateResponse, ErrorResponse};

// ============================================================================
// OpenAPI Documentation
// ============================================================================

#[derive(OpenApi)]
#[openapi(
    paths(create, update, delete_category, get_by_id, get_all),
    components(schemas(
        CategoryCreateRequest,
        CategoryCreateResponse,
        CategoryUpdateRequest,
        CategoryResponse,
        CategoryChildResponse,
        ErrorResponse
    )),
    tags(
        (name = "category", description = "Category management endpoints")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub struct CategoryApiDoc;

// ============================================================================
// HTTP Handlers
// ============================================================================

/// Create a new category
///
/// Creates a new category with the provided information. Requires authentication.
#[utoipa::path(
    post,
    path = "/api/category",
    tag = "category",
    request_body = CategoryCreateRequest,
    responses(
        (status = 200, description = "Category created successfully", body = CategoryCreateResponse),
        (status = 400, description = "Bad request - validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(category_service, payload, ctx))]
async fn create(
    State(category_service): State<Arc<dyn CategoryServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Json(payload): Json<CategoryCreateRequest>,
) -> DomainResult<impl IntoResponse> {
    // Validate input
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;

    let id = category_service
        .create(
            &ctx,
            &CategoryCreate {
                name: payload.name,
                description: payload.description,
                parent_id: payload.parent_id,
            },
        )
        .await?;

    Ok((StatusCode::CREATED, Json(CategoryCreateResponse { id })))
}

/// Update an existing category
///
/// Updates a category's information. All fields in the request body are optional.
/// Requires authentication.
#[utoipa::path(
    put,
    path = "/api/category/{id}",
    tag = "category",
    request_body = CategoryUpdateRequest,
    params(
        ("id" = i64, Path, description = "Category ID to update")
    ),
    responses(
        (status = 204, description = "Category updated successfully"),
        (status = 400, description = "Bad request - validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 404, description = "Category not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(category_service, payload, ctx))]
async fn update(
    State(category_service): State<Arc<dyn CategoryServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path(id): Path<i64>,
    Json(payload): Json<CategoryUpdateRequest>,
) -> DomainResult<impl IntoResponse> {
    // Validate input
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;

    category_service
        .update(
            &ctx,
            id,
            &CategoryUpdate {
                name: Some(payload.name),
                description: payload.description,
                parent_id: payload.parent_id,
            },
        )
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Delete a category
///
/// Soft deletes a category by marking it as deleted. Requires authentication.
#[utoipa::path(
    delete,
    path = "/api/category/{id}",
    tag = "category",
    params(
        ("id" = i64, Path, description = "Category ID to delete")
    ),
    responses(
        (status = 204, description = "Category deleted successfully"),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 404, description = "Category not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(category_service, ctx))]
async fn delete_category(
    State(category_service): State<Arc<dyn CategoryServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path(id): Path<i64>,
) -> DomainResult<impl IntoResponse> {
    category_service.delete(&ctx, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Get a category by ID
///
/// Retrieves detailed information about a specific category, including its children.
/// Requires authentication.
#[utoipa::path(
    get,
    path = "/api/category/{id}",
    tag = "category",
    params(
        ("id" = i64, Path, description = "Category ID to retrieve")
    ),
    responses(
        (status = 200, description = "Category retrieved successfully", body = CategoryResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 404, description = "Category not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(category_service, ctx))]
async fn get_by_id(
    State(category_service): State<Arc<dyn CategoryServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path(id): Path<i64>,
) -> DomainResult<impl IntoResponse> {
    let result = category_service.get_by_id(&ctx, id).await?;
    match result {
        Some(category) => Ok((
            StatusCode::OK,
            Json(CategoryResponse {
                id: category.id,
                name: category.name,
                description: category.description,
                children: category.children.map(|children| {
                    children
                        .into_iter()
                        .map(|child| CategoryChildResponse {
                            id: child.id,
                            name: child.name,
                            description: child.description,
                        })
                        .collect()
                }),
            }),
        )),
        None => Err(Error::NotFound("Category not found".to_string())),
    }
}

/// Get all categories
///
/// Retrieves a list of all categories with their hierarchical structure.
/// Each category includes its immediate children. Requires authentication.
#[utoipa::path(
    get,
    path = "/api/category",
    tag = "category",
    responses(
        (status = 200, description = "Categories retrieved successfully", body = Vec<CategoryResponse>),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(category_service, ctx))]
async fn get_all(
    State(category_service): State<Arc<dyn CategoryServiceTrait>>,
    Extension(ctx): Extension<Context>,
) -> DomainResult<impl IntoResponse> {
    let result = category_service.get_all(&ctx).await?;
    Ok((
        StatusCode::OK,
        Json(
            result
                .into_iter()
                .map(|category| CategoryResponse {
                    id: category.id,
                    name: category.name,
                    description: category.description,
                    children: category.children.map(|children| {
                        children
                            .into_iter()
                            .map(|child| CategoryChildResponse {
                                id: child.id,
                                name: child.name,
                                description: child.description,
                            })
                            .collect()
                    }),
                })
                .collect::<Vec<_>>(),
        ),
    ))
}

// ============================================================================
// Router
// ============================================================================

pub fn category_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create))
        .route("/{id}", put(update))
        .route("/{id}", delete(delete_category))
        .route("/{id}", get(get_by_id))
        .route("/", get(get_all))
}
