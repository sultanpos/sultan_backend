mod common;

use axum::Router;
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::middleware::from_fn;
use axum::response::IntoResponse;
use serde_json::json;
use std::sync::Arc;

use common::{
    MockAppStateBuilder, make_request, mock_cashier_session_service::MockCashierSessionService,
};
use sultan_core::domain::Context;
use sultan_web::{
    handler::cashier_session_router::cashier_session_router, middleware::context_middleware,
};

// ============================================================================
// Helper Functions
// ============================================================================

/// Middleware that injects a context with user_id = 1 (simulates authenticated user)
async fn inject_user_context(
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
) -> impl IntoResponse {
    use std::collections::HashMap;
    let ctx = Context::new_with_all(Some(1), HashMap::new(), HashMap::new());
    req.extensions_mut().insert(ctx);
    next.run(req).await
}

fn build_test_router(app_state: MockAppStateBuilder) -> Router {
    Router::new()
        .nest("/api/cashier-session", cashier_session_router())
        .layer(from_fn(inject_user_context))
        .with_state(app_state.build())
}

// ============================================================================
// POST /api/cashier-session - Open Session Tests
// ============================================================================

#[tokio::test]
async fn test_open_session_success() {
    let app_state = MockAppStateBuilder::new()
        .with_cashier_session_service(Arc::new(MockCashierSessionService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "branch_id": "1",
        "opening_cash": 100000
    });

    let (status, response) = make_request(app, "POST", "/api/cashier-session", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::CREATED);
    assert!(response.get("id").is_some());
    assert_eq!(response["id"].as_str().unwrap(), "1");
}

#[tokio::test]
async fn test_open_session_service_failure() {
    let app_state = MockAppStateBuilder::new()
        .with_cashier_session_service(Arc::new(MockCashierSessionService::new_failure()));

    let app = build_test_router(app_state);

    let body = json!({
        "branch_id": "1",
        "opening_cash": 100000
    });

    let (status, _) = make_request(app, "POST", "/api/cashier-session", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_open_session_negative_cash() {
    let app_state = MockAppStateBuilder::new();
    let app = build_test_router(app_state);

    // Negative opening_cash fails validation
    let body = json!({
        "branch_id": "1",
        "opening_cash": -1
    });

    let (status, response) = make_request(app, "POST", "/api/cashier-session", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["error"].as_str().unwrap().contains("Opening cash"));
}

// ============================================================================
// PUT /api/cashier-session/{id}/close - Close Session Tests
// ============================================================================

#[tokio::test]
async fn test_close_session_success() {
    let app_state = MockAppStateBuilder::new()
        .with_cashier_session_service(Arc::new(MockCashierSessionService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "closing_cash": 200000
    });

    let (status, _) = make_request(app, "PUT", "/api/cashier-session/1/close", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_close_session_not_found() {
    let app_state = MockAppStateBuilder::new()
        .with_cashier_session_service(Arc::new(MockCashierSessionService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "closing_cash": 200000
    });

    let (status, response) = make_request(app, "PUT", "/api/cashier-session/999/close", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(response.get("error").is_some());
}

#[tokio::test]
async fn test_close_session_service_failure() {
    let app_state = MockAppStateBuilder::new()
        .with_cashier_session_service(Arc::new(MockCashierSessionService::new_failure()));

    let app = build_test_router(app_state);

    let body = json!({
        "closing_cash": 200000
    });

    let (status, _) = make_request(app, "PUT", "/api/cashier-session/1/close", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
}

// ============================================================================
// GET /api/cashier-session/{id} - Get Session Tests
// ============================================================================

#[tokio::test]
async fn test_get_session_success() {
    let app_state = MockAppStateBuilder::new()
        .with_cashier_session_service(Arc::new(MockCashierSessionService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/cashier-session/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["id"].as_str().unwrap(), "1");
    assert_eq!(response["status"].as_str().unwrap(), "open");
    assert_eq!(response["opening_cash"].as_i64().unwrap(), 100_000);
}

#[tokio::test]
async fn test_get_session_not_found() {
    let app_state = MockAppStateBuilder::new()
        .with_cashier_session_service(Arc::new(MockCashierSessionService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/cashier-session/999", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(response.get("error").is_some());
}

#[tokio::test]
async fn test_get_session_service_failure() {
    let app_state = MockAppStateBuilder::new()
        .with_cashier_session_service(Arc::new(MockCashierSessionService::new_failure()));

    let app = build_test_router(app_state);

    let (status, _) = make_request(app, "GET", "/api/cashier-session/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
}

// ============================================================================
// GET /api/cashier-session/current - Get Current Session Tests
// ============================================================================

#[tokio::test]
async fn test_get_current_session_success() {
    let app_state = MockAppStateBuilder::new()
        .with_cashier_session_service(Arc::new(MockCashierSessionService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(
        app,
        "GET",
        "/api/cashier-session/current?branch_id=1",
        None,
    )
    .await
    .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["id"].as_str().unwrap(), "1");
    assert_eq!(response["status"].as_str().unwrap(), "open");
}

#[tokio::test]
async fn test_get_current_session_service_failure() {
    let app_state = MockAppStateBuilder::new()
        .with_cashier_session_service(Arc::new(MockCashierSessionService::new_failure()));

    let app = build_test_router(app_state);

    let (status, _) = make_request(
        app,
        "GET",
        "/api/cashier-session/current?branch_id=1",
        None,
    )
    .await
    .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
}

// ============================================================================
// GET /api/cashier-session - List Sessions Tests
// ============================================================================

#[tokio::test]
async fn test_get_many_sessions_success() {
    let app_state = MockAppStateBuilder::new()
        .with_cashier_session_service(Arc::new(MockCashierSessionService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/cashier-session", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["items"].as_array().unwrap().len(), 2);
    assert!(response.get("next_cursor").is_some());
}

#[tokio::test]
async fn test_get_many_sessions_with_filters() {
    let app_state = MockAppStateBuilder::new()
        .with_cashier_session_service(Arc::new(MockCashierSessionService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(
        app,
        "GET",
        "/api/cashier-session?branch_id=1&status=open&sort_direction=asc&limit=10",
        None,
    )
    .await
    .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert!(response["items"].as_array().is_some());
}

#[tokio::test]
async fn test_get_many_sessions_invalid_sort_direction() {
    let app_state = MockAppStateBuilder::new();
    let app = build_test_router(app_state);

    let (status, response) = make_request(
        app,
        "GET",
        "/api/cashier-session?sort_direction=invalid",
        None,
    )
    .await
    .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        response["error"]
            .as_str()
            .unwrap()
            .contains("sort_direction")
    );
}

#[tokio::test]
async fn test_get_many_sessions_service_failure() {
    let app_state = MockAppStateBuilder::new()
        .with_cashier_session_service(Arc::new(MockCashierSessionService::new_failure()));

    let app = build_test_router(app_state);

    let (status, _) = make_request(app, "GET", "/api/cashier-session", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
}

// ============================================================================
// DELETE /api/cashier-session/{id} - Delete Session Tests
// ============================================================================

#[tokio::test]
async fn test_delete_session_success() {
    let app_state = MockAppStateBuilder::new()
        .with_cashier_session_service(Arc::new(MockCashierSessionService::new_success()));

    let app = build_test_router(app_state);

    let (status, _) = make_request(app, "DELETE", "/api/cashier-session/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_session_not_found() {
    let app_state = MockAppStateBuilder::new()
        .with_cashier_session_service(Arc::new(MockCashierSessionService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "DELETE", "/api/cashier-session/999", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(response.get("error").is_some());
}

#[tokio::test]
async fn test_delete_session_service_failure() {
    let app_state = MockAppStateBuilder::new()
        .with_cashier_session_service(Arc::new(MockCashierSessionService::new_failure()));

    let app = build_test_router(app_state);

    let (status, _) = make_request(app, "DELETE", "/api/cashier-session/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
}
