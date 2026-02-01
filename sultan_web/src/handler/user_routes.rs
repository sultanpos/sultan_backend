use axum::Extension;
use axum::extract::{Path, Query};
use axum::routing::{get, patch};
use axum::{
    Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::delete,
    routing::post, routing::put,
};
use std::sync::Arc;
use sultan_core::application::UserServiceTrait;
use sultan_core::domain::context::Context;
use sultan_core::domain::model::permission::PermissionCreate;
use sultan_core::domain::model::user::{UserCreate, UserUpdate};
use sultan_core::domain::{DomainResult, Error};
use tracing::instrument;
use utoipa::OpenApi;
use validator::Validate;

use crate::AppState;
use crate::dto::ErrorResponse;
use crate::dto::user::{
    ChangeMyPasswordRequest, ResetPasswordRequest, UserCreateRequest, UserCreateResponse,
    UserListResponse, UserPermissionResponse, UserQueryParams, UserResponse, UserUpdateRequest,
};

// ============================================================================
// OpenAPI Documentation
// ============================================================================

#[derive(OpenApi)]
#[openapi(
    paths(create, update, delete_user, get_by_id, get_all, get_permission_by_user_id, reset_password, change_my_password),
    components(schemas(
        UserCreateRequest,
        UserCreateResponse,
        UserUpdateRequest,
        UserResponse,
        UserListResponse,
        UserPermissionResponse,
        ResetPasswordRequest,
        ChangeMyPasswordRequest,
        ErrorResponse
    )),
    tags(
        (name = "user", description = "User management endpoints")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub struct UserApiDoc;

// ============================================================================
// HTTP Handlers
// ============================================================================

/// Create a new user
///
/// Creates a new user with the provided information. Requires authentication.
#[utoipa::path(
    post,
    path = "/api/user",
    tag = "user",
    request_body = UserCreateRequest,
    responses(
        (status = 201, description = "User created successfully", body = UserCreateResponse),
        (status = 400, description = "Bad request - validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(user_service, payload, ctx))]
async fn create(
    State(user_service): State<Arc<dyn UserServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Json(payload): Json<UserCreateRequest>,
) -> DomainResult<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;

    let permissions: Vec<PermissionCreate> = payload
        .permissions
        .iter()
        .map(|p| PermissionCreate {
            branch_id: p.branch_id,
            resource: p.resource,
            action: p.action,
        })
        .collect();

    let id = user_service
        .create(
            &ctx,
            &UserCreate {
                username: payload.username,
                password: payload.password,
                name: payload.name,
                address: payload.address,
                email: payload.email,
                phone: payload.phone,
                photo: payload.photo,
                pin: payload.pin,
            },
            &permissions,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(UserCreateResponse { id })))
}

/// Update an existing user
///
/// Updates a user's information. All fields in the request body are optional.
/// Requires authentication.
#[utoipa::path(
    put,
    path = "/api/user/{id}",
    tag = "user",
    request_body = UserUpdateRequest,
    params(
        ("id" = i64, Path, description = "User ID to update")
    ),
    responses(
        (status = 204, description = "User updated successfully"),
        (status = 400, description = "Bad request - validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(user_service, payload, ctx))]
async fn update(
    State(user_service): State<Arc<dyn UserServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path(id): Path<i64>,
    Json(payload): Json<UserUpdateRequest>,
) -> DomainResult<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;

    let permissions: Option<Vec<PermissionCreate>> = payload.permissions.map(|v| {
        v.iter()
            .map(|p| PermissionCreate {
                branch_id: p.branch_id,
                resource: p.resource,
                action: p.action,
            })
            .collect()
    });

    user_service
        .update(
            &ctx,
            id,
            &UserUpdate {
                username: payload.username,
                name: payload.name,
                address: payload.address,
                email: payload.email,
                phone: payload.phone,
                photo: payload.photo,
                pin: payload.pin,
            },
            permissions,
        )
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Delete a user
///
/// Soft deletes a user by ID. Requires authentication.
#[utoipa::path(
    delete,
    path = "/api/user/{id}",
    tag = "user",
    params(
        ("id" = i64, Path, description = "User ID to delete")
    ),
    responses(
        (status = 204, description = "User deleted successfully"),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(user_service, ctx))]
async fn delete_user(
    State(user_service): State<Arc<dyn UserServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path(id): Path<i64>,
) -> DomainResult<impl IntoResponse> {
    user_service.delete(&ctx, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Get a user by ID
///
/// Retrieves a single user by their ID. Requires authentication.
#[utoipa::path(
    get,
    path = "/api/user/{id}",
    tag = "user",
    params(
        ("id" = i64, Path, description = "User ID to retrieve")
    ),
    responses(
        (status = 200, description = "User found", body = UserResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(user_service, ctx))]
async fn get_by_id(
    State(user_service): State<Arc<dyn UserServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path(id): Path<i64>,
) -> DomainResult<impl IntoResponse> {
    let user = user_service
        .get_by_id(&ctx, id)
        .await?
        .ok_or(Error::NotFound(format!("User with id {} not found", id)))?;
    Ok((StatusCode::OK, Json(UserResponse::from(user))))
}

/// Get all users
/// Get all users
///
/// Retrieves a list of all users with optional filtering and pagination. Requires authentication.
#[utoipa::path(
    get,
    path = "/api/user",
    tag = "user",
    params(
        UserQueryParams
    ),
    responses(
        (status = 200, description = "List of users", body = UserListResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(user_service, ctx))]
async fn get_all(
    State(user_service): State<Arc<dyn UserServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Query(query): Query<UserQueryParams>,
) -> DomainResult<impl IntoResponse> {
    let filter = query.to_filter();
    let pagination = query.to_pagination();
    let users = user_service.get_all(&ctx, &filter, &pagination).await?;
    Ok((
        StatusCode::OK,
        Json(UserListResponse {
            data: users.into_iter().map(UserResponse::from).collect(),
        }),
    ))
}

/// Get user permissions
///
/// Retrieves all permissions assigned to a specific user. Requires authentication.
#[utoipa::path(
    get,
    path = "/api/user/{id}/permissions",
    tag = "user",
    params(
        ("id" = i64, Path, description = "User ID to get permissions for")
    ),
    responses(
        (status = 200, description = "List of user permissions", body = Vec<UserPermissionResponse>),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(user_service, ctx))]
async fn get_permission_by_user_id(
    State(user_service): State<Arc<dyn UserServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path(id): Path<i64>,
) -> DomainResult<impl IntoResponse> {
    let permissions = user_service.get_user_permission(&ctx, id).await?;
    Ok((
        StatusCode::OK,
        Json(
            permissions
                .into_iter()
                .map(UserPermissionResponse::from)
                .collect::<Vec<UserPermissionResponse>>(),
        ),
    ))
}

/// Reset user password (admin)
///
/// Resets a user's password. This is an admin operation. Requires authentication.
#[utoipa::path(
    patch,
    path = "/api/user/{id}/password",
    tag = "user",
    request_body = ResetPasswordRequest,
    params(
        ("id" = i64, Path, description = "User ID to reset password for")
    ),
    responses(
        (status = 204, description = "Password reset successfully"),
        (status = 400, description = "Bad request - validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(user_service, ctx))]
async fn reset_password(
    State(user_service): State<Arc<dyn UserServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Path(id): Path<i64>,
    Json(payload): Json<ResetPasswordRequest>,
) -> DomainResult<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;

    user_service
        .reset_password(&ctx, id, payload.new_password)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Change my password (self-service)
///
/// Allows authenticated users to change their own password by providing the old password.
/// Requires authentication.
#[utoipa::path(
    patch,
    path = "/api/user/mypassword",
    tag = "user",
    request_body = ChangeMyPasswordRequest,
    params(
        ("id" = i64, Path, description = "User ID (must match authenticated user)")
    ),
    responses(
        (status = 204, description = "Password changed successfully"),
        (status = 400, description = "Bad request - validation error or wrong old password", body = ErrorResponse),
        (status = 401, description = "Unauthorized - missing or invalid token", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(user_service, ctx))]
async fn change_my_password(
    State(user_service): State<Arc<dyn UserServiceTrait>>,
    Extension(ctx): Extension<Context>,
    Json(payload): Json<ChangeMyPasswordRequest>,
) -> DomainResult<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;

    user_service
        .change_my_password(&ctx, payload.old_password, payload.new_password)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Router Configuration
// ============================================================================

pub fn user_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create))
        .route("/", get(get_all))
        .route("/{id}", get(get_by_id))
        .route("/{id}", put(update))
        .route("/{id}", delete(delete_user))
        .route("/{id}/permissions", get(get_permission_by_user_id))
        .route("/{id}/password", patch(reset_password))
        .route("/mypassword", patch(change_my_password))
}
