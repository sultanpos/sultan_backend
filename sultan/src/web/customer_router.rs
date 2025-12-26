use axum::Extension;
use axum::extract::Path;
use axum::routing::get;
use axum::{
    Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::delete,
    routing::post, routing::put,
};
use std::sync::Arc;
use sultan_core::application::{CategoryServiceTrait, CustomerServiceTrait};
use sultan_core::domain::context::BranchContext;
use sultan_core::domain::model::category::{CategoryCreate, CategoryUpdate};
use sultan_core::domain::model::customer::CustomerCreate;
use sultan_core::domain::{DomainResult, Error};
use tracing::instrument;
use utoipa::OpenApi;
use validator::Validate;

use crate::domain::dto::{CustomerCreateRequest, CustomerCreateResponse, ErrorResponse};
use crate::web::AppState;

// ============================================================================
// OpenAPI Documentation
// ============================================================================

#[derive(OpenApi)]
#[openapi(
    paths(create),
    components(schemas(
        CustomerCreateRequest,
        CustomerCreateResponse,
        ErrorResponse,
    )),
    tags(
        (name = "Customer", description = "Customer management endpoints")
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

// ============================================================================
// Router
// ============================================================================

pub fn customer_router() -> Router<AppState> {
    Router::new().route("/", post(create))
}
