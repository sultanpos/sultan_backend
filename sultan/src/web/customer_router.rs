use axum::Extension;
use axum::extract::{Path, Query};
use axum::routing::get;
use axum::{
    Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::delete,
    routing::post, routing::put,
};
use std::sync::Arc;
use sultan_core::application::CustomerServiceTrait;
use sultan_core::domain::context::BranchContext;
use sultan_core::domain::model::customer::{CustomerCreate, CustomerUpdate};
use sultan_core::domain::{DomainResult, Error};
use tracing::instrument;
use utoipa::OpenApi;
use validator::Validate;

use crate::domain::dto::customer::{
    CustomerListResponse, CustomerQueryParams, CustomerResponse, CustomerUpdateRequest,
};
use crate::domain::dto::{CustomerCreateRequest, CustomerCreateResponse, ErrorResponse};
use crate::web::AppState;

// ============================================================================
// OpenAPI Documentation
// ============================================================================

#[derive(OpenApi)]
#[openapi(
    paths(create, update, delete_customer, get_by_id, get_all),
    components(schemas(
        CustomerCreateRequest,
        CustomerCreateResponse,
        CustomerUpdateRequest,
        CustomerResponse,
        CustomerListResponse,
        ErrorResponse,
    )),
    tags(
        (name = "customer", description = "Customer management endpoints")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub struct CustomerApiDoc;

#[utoipa::path(
    post,
    path = "/api/customer",
    tag = "customer",
    request_body = CustomerCreateRequest,
    responses(
        (status = 200, description = "Customer created successfully", body = CustomerCreateResponse),
        (status = 400, description = "Bad request - validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(customer_service, payload, ctx))]
async fn create(
    State(customer_service): State<Arc<dyn CustomerServiceTrait<BranchContext>>>,
    Extension(ctx): Extension<BranchContext>,
    Json(payload): Json<CustomerCreateRequest>,
) -> DomainResult<impl IntoResponse> {
    // Validate input
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;

    let id = customer_service
        .create(
            &ctx,
            &CustomerCreate {
                name: payload.name,
                number: payload.number.unwrap_or_default(),
                address: payload.address,
                email: payload.email,
                phone: payload.phone,
                level: payload.level,
                metadata: payload.metadata,
            },
        )
        .await?;

    Ok((StatusCode::CREATED, Json(CustomerCreateResponse { id })))
}

#[utoipa::path(
    put,
    path = "/api/customer/{id}",
    tag = "customer",
    request_body = CustomerUpdateRequest,
    params(
        ("id" = i64, Path, description = "Customer ID to update")
    ),
    responses(
        (status = 204, description = "Customer updated successfully"),
        (status = 400, description = "Bad request - validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 404, description = "Customer not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(customer_service, payload, ctx))]
async fn update(
    State(customer_service): State<Arc<dyn CustomerServiceTrait<BranchContext>>>,
    Extension(ctx): Extension<BranchContext>,
    Path(id): Path<i64>,
    Json(payload): Json<CustomerUpdateRequest>,
) -> DomainResult<impl IntoResponse> {
    // Validate input
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;

    customer_service
        .update(
            &ctx,
            id,
            &CustomerUpdate {
                name: payload.name,
                number: payload.number,
                address: payload.address,
                email: payload.email,
                phone: payload.phone,
                level: payload.level,
                metadata: payload.metadata,
            },
        )
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/customer/{id}",
    tag = "customer",
    params(
        ("id" = i64, Path, description = "Customer ID to delete")
    ),
    responses(
        (status = 204, description = "Customer deleted successfully"),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 404, description = "Customer not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(customer_service, ctx))]
async fn delete_customer(
    State(customer_service): State<Arc<dyn CustomerServiceTrait<BranchContext>>>,
    Extension(ctx): Extension<BranchContext>,
    Path(id): Path<i64>,
) -> DomainResult<impl IntoResponse> {
    customer_service.delete(&ctx, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/customer/{id}",
    tag = "customer",
    params(
        ("id" = i64, Path, description = "Customer ID to retrieve")
    ),
    responses(
        (status = 200, description = "Customer retrieved successfully", body = CustomerResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 404, description = "Customer not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(customer_service, ctx))]
async fn get_by_id(
    State(customer_service): State<Arc<dyn CustomerServiceTrait<BranchContext>>>,
    Extension(ctx): Extension<BranchContext>,
    Path(id): Path<i64>,
) -> DomainResult<impl IntoResponse> {
    let customer = customer_service
        .get_by_id(&ctx, id)
        .await?
        .ok_or(Error::NotFound(format!(
            "Customer with id {} not found",
            id
        )))?;
    Ok((StatusCode::OK, Json(CustomerResponse::from(customer))))
}

#[utoipa::path(
    get,
    path = "/api/customer",
    tag = "customer",
    params(
        ("number" = Option<String>, Query, description = "Filter by customer number"),
        ("name" = Option<String>, Query, description = "Filter by customer name (partial match)"),
        ("phone" = Option<String>, Query, description = "Filter by phone number"),
        ("email" = Option<String>, Query, description = "Filter by email"),
        ("level" = Option<i32>, Query, description = "Filter by customer level"),
        ("page" = u32, Query, description = "Page number (default: 1)"),
        ("page_size" = u32, Query, description = "Page size (default: 20, max: 100)"),
        ("order_by" = Option<String>, Query, description = "Order by field"),
        ("order_direction" = Option<String>, Query, description = "Order direction (asc/desc)")
    ),
    responses(
        (status = 200, description = "Customers retrieved successfully", body = CustomerListResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(customer_service, ctx))]
async fn get_all(
    State(customer_service): State<Arc<dyn CustomerServiceTrait<BranchContext>>>,
    Extension(ctx): Extension<BranchContext>,
    Query(query): Query<CustomerQueryParams>,
) -> DomainResult<impl IntoResponse> {
    let filter = query.to_filter();
    let pagination = query.to_pagination();
    let customer = customer_service.get_all(&ctx, &filter, &pagination).await?;
    Ok((
        StatusCode::OK,
        Json(CustomerListResponse {
            customers: customer.into_iter().map(CustomerResponse::from).collect(),
        }),
    ))
}

// ============================================================================
// Router
// ============================================================================

pub fn customer_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create))
        .route("/{id}", put(update))
        .route("/{id}", delete(delete_customer))
        .route("/{id}", get(get_by_id))
        .route("/", get(get_all))
}
