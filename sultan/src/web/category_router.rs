use axum::Extension;
use axum::extract::Path;
use axum::routing::get;
use axum::{
    Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::delete,
    routing::post, routing::put,
};
use std::sync::Arc;
use sultan_core::application::CategoryServiceTrait;
use sultan_core::domain::context::BranchContext;
use sultan_core::domain::model::category::{CategoryCreate, CategoryUpdate};
use sultan_core::domain::{DomainResult, Error};
use tracing::instrument;
use utoipa::OpenApi;
use validator::Validate;

use crate::domain::dto::category::{CategoryResponse, CategoryUpdateRequest};
use crate::domain::dto::{CategoryCreateRequest, CategoryCreateResponse, ErrorResponse};
use crate::web::AppState;

// ============================================================================
// OpenAPI Documentation
// ============================================================================

#[derive(OpenApi)]
#[openapi(
    paths(create),
    components(schemas(CategoryCreateRequest, CategoryCreateResponse, ErrorResponse)),
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
    State(category_service): State<Arc<dyn CategoryServiceTrait<BranchContext>>>,
    Extension(ctx): Extension<BranchContext>,
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

#[instrument(skip(category_service, payload, ctx))]
async fn update(
    State(category_service): State<Arc<dyn CategoryServiceTrait<BranchContext>>>,
    Extension(ctx): Extension<BranchContext>,
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

#[instrument(skip(category_service, ctx))]
async fn delete_category(
    State(category_service): State<Arc<dyn CategoryServiceTrait<BranchContext>>>,
    Extension(ctx): Extension<BranchContext>,
    Path(id): Path<i64>,
) -> DomainResult<impl IntoResponse> {
    category_service.delete(&ctx, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[instrument(skip(category_service, ctx))]
async fn get_by_id(
    State(category_service): State<Arc<dyn CategoryServiceTrait<BranchContext>>>,
    Extension(ctx): Extension<BranchContext>,
    Path(id): Path<i64>,
) -> DomainResult<impl IntoResponse> {
    let result = category_service.get_by_id(&ctx, id).await?;
    match result {
        Some(category) => Ok((
            StatusCode::OK,
            Json(CategoryResponse {
                name: category.name,
                description: category.description,
                children: category.children.map(|children| {
                    children
                        .into_iter()
                        .map(|child| CategoryResponse {
                            name: child.name,
                            description: child.description,
                            children: None, // For simplicity, not including grandchildren
                        })
                        .collect()
                }),
            }),
        )),
        None => Err(Error::NotFound("Category not found".to_string())),
    }
}

#[instrument(skip(category_service, ctx))]
async fn get_all(
    State(category_service): State<Arc<dyn CategoryServiceTrait<BranchContext>>>,
    Extension(ctx): Extension<BranchContext>,
) -> DomainResult<impl IntoResponse> {
    let result = category_service.get_all(&ctx).await?;
    Ok((
        StatusCode::OK,
        Json(
            result
                .into_iter()
                .map(|category| CategoryResponse {
                    name: category.name,
                    description: category.description,
                    children: category.children.map(|children| {
                        children
                            .into_iter()
                            .map(|child| CategoryResponse {
                                name: child.name,
                                description: child.description,
                                children: None,
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
        .route("/:id", put(update))
        .route("/:id", delete(delete_category))
        .route("/:id", get(get_by_id))
        .route("/", get(get_all))
}
