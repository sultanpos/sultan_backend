mod common;

use axum::{
    Router,
    body::Body,
    extract::Extension,
    http::{Request, StatusCode, header},
    middleware,
    response::IntoResponse,
    routing::post,
};
use serde_json::{Value, json};
use std::sync::Arc;
use sultan_core::application::UserServiceTrait;
use sultan_core::crypto::{DefaultJwtManager, JwtConfig, JwtManager};
use sultan_core::domain::Context;
use sultan_core::domain::model::permission::{action, resource};
use sultan_web::handler::middleware::verify_jwt;
use sultan_web::handler::user_routes::user_router;
use sultan_web::middleware::context_middleware;
use tower::ServiceExt;

use common::{MockAppStateBuilder, MockUserService, make_request};

// ============================================================================
// Test Utilities
// ============================================================================

/// Helper function to build a test router with the context middleware
fn build_test_router(app_state: MockAppStateBuilder) -> Router {
    Router::new()
        .nest("/api/user", user_router())
        .layer(middleware::from_fn(context_middleware))
        .with_state(app_state.build())
}

// Helper to extract JSON response
async fn get_json_response(response: axum::response::Response) -> (StatusCode, Value) {
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap_or(json!({}));
    (status, json)
}

// Test handler that checks permissions in context
async fn test_permissions_handler(Extension(ctx): Extension<Context>) -> impl IntoResponse {
    let user_id = ctx.user_id();

    // Check if user has USER READ permission
    let has_user_read = ctx.has_access(None, resource::USER, action::READ);
    let has_user_create = ctx.has_access(None, resource::USER, action::CREATE);
    let has_user_update = ctx.has_access(None, resource::USER, action::UPDATE);

    axum::Json(json!({
        "user_id": user_id,
        "has_user_read": has_user_read,
        "has_user_create": has_user_create,
        "has_user_update": has_user_update,
    }))
}

// ============================================================================
// POST /api/user - Create User Tests
// ============================================================================

#[tokio::test]
async fn test_create_user_success() {
    let app_state =
        MockAppStateBuilder::new().with_user_service(Arc::new(MockUserService::new_success()));
    let app = build_test_router(app_state);

    let body = json!({
        "username": "testuser",
        "password": "password123",
        "name": "Test User",
        "email": "test@example.com",
        "permissions": []
    });

    let (status, response) = make_request(app, "POST", "/api/user", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::CREATED);
    assert!(response.get("id").is_some());
    assert_eq!(response["id"].as_str().unwrap(), "1");
}

#[tokio::test]
async fn test_create_user_validation_error_empty_username() {
    let app_state =
        MockAppStateBuilder::new().with_user_service(Arc::new(MockUserService::new_success()));
    let app = build_test_router(app_state);

    let body = json!({
        "username": "",
        "password": "password123",
        "name": "Test User",
        "permissions": []
    });

    let (status, response) = make_request(app, "POST", "/api/user", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["error"].as_str().unwrap().contains("Username"));
}

#[tokio::test]
async fn test_create_user_validation_error_empty_password() {
    let app_state =
        MockAppStateBuilder::new().with_user_service(Arc::new(MockUserService::new_success()));
    let app = build_test_router(app_state);

    let body = json!({
        "username": "testuser",
        "password": "",
        "name": "Test User",
        "permissions": []
    });

    let (status, response) = make_request(app, "POST", "/api/user", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["error"].as_str().unwrap().contains("Password"));
}

#[tokio::test]
async fn test_create_user_service_error() {
    let app_state =
        MockAppStateBuilder::new().with_user_service(Arc::new(MockUserService::new_failure()));
    let app = build_test_router(app_state);

    let body = json!({
        "username": "testuser",
        "password": "password123",
        "name": "Test User",
        "permissions": []
    });

    let (status, response) = make_request(app, "POST", "/api/user", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response.get("error").is_some());
}

// ============================================================================
// PUT /api/user/:id - Update User Tests
// ============================================================================

#[tokio::test]
async fn test_update_user_success() {
    let app_state =
        MockAppStateBuilder::new().with_user_service(Arc::new(MockUserService::new_success()));
    let app = build_test_router(app_state);

    let body = json!({
        "name": "Updated User",
        "email": "updated@example.com"
    });

    let (status, _response) = make_request(app, "PUT", "/api/user/1", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_update_user_service_error() {
    let app_state =
        MockAppStateBuilder::new().with_user_service(Arc::new(MockUserService::new_failure()));
    let app = build_test_router(app_state);

    let body = json!({
        "name": "Updated User"
    });

    let (status, response) = make_request(app, "PUT", "/api/user/1", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response.get("error").is_some());
}

// ============================================================================
// DELETE /api/user/:id - Delete User Tests
// ============================================================================

#[tokio::test]
async fn test_delete_user_success() {
    let app_state =
        MockAppStateBuilder::new().with_user_service(Arc::new(MockUserService::new_success()));
    let app = build_test_router(app_state);

    let (status, _response) = make_request(app, "DELETE", "/api/user/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_user_service_error() {
    let app_state =
        MockAppStateBuilder::new().with_user_service(Arc::new(MockUserService::new_failure()));
    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "DELETE", "/api/user/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response.get("error").is_some());
}

// ============================================================================
// GET /api/user/:id - Get User by ID Tests
// ============================================================================

#[tokio::test]
async fn test_get_user_by_id_success() {
    let app_state =
        MockAppStateBuilder::new().with_user_service(Arc::new(MockUserService::new_success()));
    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/user/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["id"].as_str().unwrap(), "1");
    assert_eq!(response["username"].as_str().unwrap(), "testuser");
    assert_eq!(response["name"].as_str().unwrap(), "Test User");
}

#[tokio::test]
async fn test_get_user_by_id_not_found() {
    let app_state =
        MockAppStateBuilder::new().with_user_service(Arc::new(MockUserService::new_success()));
    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/user/999", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(response["error"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn test_get_user_by_id_service_error() {
    let app_state =
        MockAppStateBuilder::new().with_user_service(Arc::new(MockUserService::new_failure()));
    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/user/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response.get("error").is_some());
}

// ============================================================================
// GET /api/user - Get All Users Tests
// ============================================================================

#[tokio::test]
async fn test_get_all_users_service_error() {
    let app_state =
        MockAppStateBuilder::new().with_user_service(Arc::new(MockUserService::new_success()));
    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/user", None)
        .await
        .expect("Request failed");

    // MockUserService returns error for get_all
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response.get("error").is_some());
}

// ============================================================================
// GET /api/user/:id/permissions - Get User Permissions Tests
// ============================================================================

#[tokio::test]
async fn test_get_user_permissions_success() {
    let app_state =
        MockAppStateBuilder::new().with_user_service(Arc::new(MockUserService::new_success()));
    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/user/1/permissions", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert!(response.is_array());
    let permissions = response.as_array().unwrap();
    assert_eq!(permissions.len(), 1);
}

#[tokio::test]
async fn test_get_user_permissions_no_permissions() {
    let app_state =
        MockAppStateBuilder::new().with_user_service(Arc::new(MockUserService::new_success()));
    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/user/999/permissions", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert!(response.is_array());
    let permissions = response.as_array().unwrap();
    assert_eq!(permissions.len(), 0);
}

#[tokio::test]
async fn test_get_user_permissions_service_error() {
    let app_state =
        MockAppStateBuilder::new().with_user_service(Arc::new(MockUserService::new_failure()));
    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/user/1/permissions", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response.get("error").is_some());
}

// ============================================================================
// PATCH /api/user/:id/password - Reset Password Tests
// ============================================================================

#[tokio::test]
async fn test_reset_password_success() {
    let app_state =
        MockAppStateBuilder::new().with_user_service(Arc::new(MockUserService::new_success()));
    let app = build_test_router(app_state);

    let body = json!({
        "new_password": "newpassword123"
    });

    let (status, _response) = make_request(app, "PATCH", "/api/user/1/password", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_reset_password_validation_error() {
    let app_state =
        MockAppStateBuilder::new().with_user_service(Arc::new(MockUserService::new_success()));
    let app = build_test_router(app_state);

    let body = json!({
        "new_password": ""
    });

    let (status, response) = make_request(app, "PATCH", "/api/user/1/password", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["error"].as_str().unwrap().contains("password"));
}

#[tokio::test]
async fn test_reset_password_service_error() {
    let app_state =
        MockAppStateBuilder::new().with_user_service(Arc::new(MockUserService::new_failure()));
    let app = build_test_router(app_state);

    let body = json!({
        "new_password": "newpassword123"
    });

    let (status, response) = make_request(app, "PATCH", "/api/user/1/password", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response.get("error").is_some());
}

// ============================================================================
// PATCH /api/user/:id/mypassword - Change My Password Tests
// ============================================================================

#[tokio::test]
async fn test_change_my_password_service_error() {
    let app_state =
        MockAppStateBuilder::new().with_user_service(Arc::new(MockUserService::new_success()));
    let app = build_test_router(app_state);

    let body = json!({
        "old_password": "oldpassword123",
        "new_password": "newpassword123"
    });

    let (status, response) = make_request(app, "PATCH", "/api/user/mypassword", Some(body))
        .await
        .expect("Request failed");

    // MockUserService returns error for change_my_password
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response.get("error").is_some());
}

#[tokio::test]
async fn test_change_my_password_validation_error_empty_old_password() {
    let app_state =
        MockAppStateBuilder::new().with_user_service(Arc::new(MockUserService::new_success()));
    let app = build_test_router(app_state);

    let body = json!({
        "old_password": "",
        "new_password": "newpassword123"
    });

    let (status, response) = make_request(app, "PATCH", "/api/user/mypassword", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["error"].as_str().unwrap().contains("password"));
}

#[tokio::test]
async fn test_change_my_password_validation_error_empty_new_password() {
    let app_state =
        MockAppStateBuilder::new().with_user_service(Arc::new(MockUserService::new_success()));
    let app = build_test_router(app_state);

    let body = json!({
        "old_password": "oldpassword123",
        "new_password": ""
    });

    let (status, response) = make_request(app, "PATCH", "/api/user/mypassword", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["error"].as_str().unwrap().contains("password"));
}

// ============================================================================
// Service-Level Tests
// ============================================================================

#[tokio::test]
async fn test_user_service_get_user_permission_success() {
    // Setup mock user service
    let mock_user_service = Arc::new(MockUserService::new_success());

    // Test the service directly
    let ctx = Context::new_internal();
    let permissions = mock_user_service
        .get_user_permission(&ctx, 1)
        .await
        .unwrap();

    // Assert
    assert_eq!(permissions.len(), 1);
    assert_eq!(permissions[0].user_id, 1);
    assert_eq!(permissions[0].resource, resource::USER);
    assert_eq!(permissions[0].action, action::READ | action::CREATE);
    assert_eq!(permissions[0].branch_id, None);
}

#[tokio::test]
async fn test_user_service_get_user_permission_not_found() {
    // Setup mock user service
    let mock_user_service = Arc::new(MockUserService::new_success());

    // Test the service directly for non-existent user
    let ctx = Context::new_internal();
    let permissions = mock_user_service
        .get_user_permission(&ctx, 999)
        .await
        .unwrap();

    // Assert - should return empty vec for non-existent user
    assert_eq!(permissions.len(), 0);
}

#[tokio::test]
async fn test_user_service_get_user_permission_failure() {
    // Setup mock user service that returns errors
    let mock_user_service = Arc::new(MockUserService::new_failure());

    // Test the service directly
    let ctx = Context::new_internal();
    let result = mock_user_service.get_user_permission(&ctx, 1).await;

    // Assert - should return error
    assert!(result.is_err());
}

// ============================================================================
// JWT Middleware Tests
// ============================================================================

#[tokio::test]
async fn test_verify_jwt_middleware_sets_correct_permissions() {
    // Setup - create a valid JWT token for user ID 1
    let jwt_manager = DefaultJwtManager::new(JwtConfig::new(
        "test_secret_key_which_is_long_enough".to_string(),
        3600,
    ));

    let token = jwt_manager.generate_token(1, "testuser").unwrap();

    // Setup app state with mock user service
    let mock_user_service = Arc::new(MockUserService::new_success());
    let app_state = MockAppStateBuilder::new()
        .with_user_service(mock_user_service)
        .build();

    let app = Router::new()
        .route("/test", post(test_permissions_handler))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            verify_jwt,
        ))
        .with_state(app_state);

    // Request with valid token
    let request = Request::builder()
        .method("POST")
        .uri("/test")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let (status, json) = get_json_response(response).await;

    // Assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["user_id"], 1);

    // Verify permissions are correctly set in context
    assert_eq!(
        json["has_user_read"], true,
        "Should have USER READ permission"
    );
    assert_eq!(
        json["has_user_create"], true,
        "Should have USER CREATE permission"
    );
    assert_eq!(
        json["has_user_update"], false,
        "Should NOT have USER UPDATE permission"
    );
}

#[tokio::test]
async fn test_verify_jwt_middleware_different_user_no_permissions() {
    // Setup - create a valid JWT token for user ID 999 (not in mock data)
    let jwt_manager = DefaultJwtManager::new(JwtConfig::new(
        "test_secret_key_which_is_long_enough".to_string(),
        3600,
    ));

    let token = jwt_manager.generate_token(999, "otheruser").unwrap();

    // Setup app state with mock user service
    let mock_user_service = Arc::new(MockUserService::new_success());
    let app_state = MockAppStateBuilder::new()
        .with_user_service(mock_user_service)
        .build();

    let app = Router::new()
        .route("/test", post(test_permissions_handler))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            verify_jwt,
        ))
        .with_state(app_state);

    // Request with valid token for user without permissions
    let request = Request::builder()
        .method("POST")
        .uri("/test")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let (status, json) = get_json_response(response).await;

    // Assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["user_id"], 999);

    // Verify no permissions are set in context for this user
    assert_eq!(
        json["has_user_read"], false,
        "Should NOT have USER READ permission"
    );
    assert_eq!(
        json["has_user_create"], false,
        "Should NOT have USER CREATE permission"
    );
    assert_eq!(
        json["has_user_update"], false,
        "Should NOT have USER UPDATE permission"
    );
}

#[tokio::test]
async fn test_verify_jwt_middleware_permission_caching() {
    // Setup - create a valid JWT token
    let jwt_manager = DefaultJwtManager::new(JwtConfig::new(
        "test_secret_key_which_is_long_enough".to_string(),
        3600,
    ));

    let token = jwt_manager.generate_token(1, "testuser").unwrap();

    // Setup app state with mock user service
    let mock_user_service = Arc::new(MockUserService::new_success());
    let app_state = MockAppStateBuilder::new()
        .with_user_service(mock_user_service)
        .build();

    let app = Router::new()
        .route("/test", post(test_permissions_handler))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            verify_jwt,
        ))
        .with_state(app_state);

    // First request
    let request1 = Request::builder()
        .method("POST")
        .uri("/test")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response1 = app.clone().oneshot(request1).await.unwrap();
    let (status1, json1) = get_json_response(response1).await;

    assert_eq!(status1, StatusCode::OK);
    assert_eq!(json1["has_user_read"], true);

    // Second request with same token - should get same permissions
    let token2 = jwt_manager.generate_token(1, "testuser").unwrap();
    let request2 = Request::builder()
        .method("POST")
        .uri("/test")
        .header(header::AUTHORIZATION, format!("Bearer {}", token2))
        .body(Body::empty())
        .unwrap();

    let response2 = app.oneshot(request2).await.unwrap();
    let (status2, json2) = get_json_response(response2).await;

    assert_eq!(status2, StatusCode::OK);
    assert_eq!(json2["has_user_read"], true);
    assert_eq!(json2["has_user_create"], true);
}
