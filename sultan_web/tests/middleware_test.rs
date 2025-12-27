mod common;

use axum::{
    Router,
    body::Body,
    extract::Extension,
    http::{Request, StatusCode, header},
    middleware,
    response::IntoResponse,
    routing::get,
};
use serde_json::{Value, json};
use std::sync::Arc;
use sultan_core::crypto::{DefaultJwtManager, JwtConfig, JwtManager};
use sultan_core::domain::Context;
use sultan_web::handler::middleware::{context_middleware, verify_jwt};
use tower::ServiceExt;

use common::{MockAppStateBuilder, mock_auth_service::MockAuthService};

// Helper to extract JSON response
async fn get_json_response(response: axum::response::Response) -> (StatusCode, Value) {
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap_or(json!({}));
    (status, json)
}

// Test handler that uses the context
async fn test_handler_with_context(Extension(ctx): Extension<Context>) -> impl IntoResponse {
    let user_id = ctx.user_id();
    axum::Json(json!({
        "user_id": user_id,
        "message": "success"
    }))
}

// Test handler without context
async fn test_handler_no_auth() -> impl IntoResponse {
    axum::Json(json!({
        "message": "no auth required"
    }))
}

#[tokio::test]
async fn test_context_middleware_adds_context() {
    // Create a simple router with context middleware
    let app = Router::new()
        .route("/test", get(test_handler_no_auth))
        .layer(middleware::from_fn(context_middleware));

    let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let (status, json) = get_json_response(response).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["message"], "no auth required");
}

#[tokio::test]
async fn test_verify_jwt_missing_authorization_header() {
    // Setup
    let app_state = MockAppStateBuilder::new()
        .with_auth_service(Arc::new(MockAuthService::new_success()))
        .build();

    let app = Router::new()
        .route("/test", get(test_handler_with_context))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            verify_jwt,
        ))
        .with_state(app_state);

    // Request without Authorization header
    let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

    let response = app.oneshot(request).await.unwrap();

    let (status, json) = get_json_response(response).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(json["error"], "Missing or invalid authorization header");
}

#[tokio::test]
async fn test_verify_jwt_invalid_authorization_format() {
    // Setup
    let app_state = MockAppStateBuilder::new()
        .with_auth_service(Arc::new(MockAuthService::new_success()))
        .build();

    let app = Router::new()
        .route("/test", get(test_handler_with_context))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            verify_jwt,
        ))
        .with_state(app_state);

    // Request with invalid Authorization header format (missing "Bearer ")
    let request = Request::builder()
        .uri("/test")
        .header(header::AUTHORIZATION, "InvalidToken")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let (status, json) = get_json_response(response).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(json["error"], "Missing or invalid authorization header");
}

#[tokio::test]
async fn test_verify_jwt_expired_token() {
    // Setup
    let app_state = MockAppStateBuilder::new()
        .with_auth_service(Arc::new(MockAuthService::new_success()))
        .build();

    let app = Router::new()
        .route("/test", get(test_handler_with_context))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            verify_jwt,
        ))
        .with_state(app_state);

    // Request with invalid/expired token
    let request = Request::builder()
        .uri("/test")
        .header(header::AUTHORIZATION, "Bearer invalid_or_expired_token")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let (status, json) = get_json_response(response).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(json["error"], "Invalid or expired token");
}

#[tokio::test]
async fn test_verify_jwt_valid_token() {
    // Setup - create a valid JWT token
    let jwt_manager = DefaultJwtManager::new(JwtConfig::new(
        "test_secret_key_which_is_long_enough".to_string(),
        3600,
    ));

    let token = jwt_manager.generate_token(123456, "testuser").unwrap();

    let app_state = MockAppStateBuilder::new()
        .with_auth_service(Arc::new(MockAuthService::new_success()))
        .build();

    let app = Router::new()
        .route("/test", get(test_handler_with_context))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            verify_jwt,
        ))
        .with_state(app_state);

    // Request with valid token
    let request = Request::builder()
        .uri("/test")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let (status, json) = get_json_response(response).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["user_id"], 123456);
    assert_eq!(json["message"], "success");
}

#[tokio::test]
async fn test_verify_jwt_empty_bearer_token() {
    // Setup
    let app_state = MockAppStateBuilder::new()
        .with_auth_service(Arc::new(MockAuthService::new_success()))
        .build();

    let app = Router::new()
        .route("/test", get(test_handler_with_context))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            verify_jwt,
        ))
        .with_state(app_state);

    // Request with "Bearer " but no token
    let request = Request::builder()
        .uri("/test")
        .header(header::AUTHORIZATION, "Bearer ")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let (status, json) = get_json_response(response).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(json["error"], "Invalid or expired token");
}

#[tokio::test]
async fn test_verify_jwt_bearer_case_sensitivity() {
    // Setup
    let app_state = MockAppStateBuilder::new()
        .with_auth_service(Arc::new(MockAuthService::new_success()))
        .build();

    let app = Router::new()
        .route("/test", get(test_handler_with_context))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            verify_jwt,
        ))
        .with_state(app_state);

    // Request with lowercase "bearer"
    let request = Request::builder()
        .uri("/test")
        .header(header::AUTHORIZATION, "bearer sometoken")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let (status, json) = get_json_response(response).await;
    // Should fail because we check for "Bearer " with capital B
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(json["error"], "Missing or invalid authorization header");
}

#[tokio::test]
async fn test_context_middleware_integration() {
    // Test that context middleware properly initializes context
    let app = Router::new()
        .route(
            "/test",
            get(|Extension(ctx): Extension<Context>| async move {
                // Context should exist but no user_id should be set
                assert!(ctx.user_id().is_none());
                axum::Json(json!({"status": "ok"}))
            }),
        )
        .layer(middleware::from_fn(context_middleware));

    let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
