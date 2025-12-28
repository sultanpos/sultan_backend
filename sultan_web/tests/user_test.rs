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
use tower::ServiceExt;

use common::{MockAppStateBuilder, MockUserService};

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
    assert_eq!(permissions[0].permission, resource::USER);
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
