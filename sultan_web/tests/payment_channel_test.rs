mod common;

use axum::Router;
use axum::http::StatusCode;
use axum::middleware::from_fn;
use serde_json::json;
use std::sync::Arc;

use common::{
    MockAppStateBuilder, make_request, mock_payment_channel_service::MockPaymentChannelService,
};
use sultan_web::{
    handler::payment_channel_router::payment_channel_router, middleware::context_middleware,
};

fn build_test_router(app_state: MockAppStateBuilder) -> Router {
    Router::new()
        .nest("/api/payment-channel", payment_channel_router())
        .layer(from_fn(context_middleware))
        .with_state(app_state.build())
}

// ============================================================================
// POST /api/payment-channel
// ============================================================================

#[tokio::test]
async fn test_create_success() {
    let app_state = MockAppStateBuilder::new()
        .with_payment_channel_service(Arc::new(MockPaymentChannelService::new_success()));
    let app = build_test_router(app_state);

    let body = json!({ "name": "Cash", "priority": 1 });
    let (status, response) = make_request(app, "POST", "/api/payment-channel", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(response["id"].as_str().unwrap(), "1");
}

#[tokio::test]
async fn test_create_validation_error_empty_name() {
    let app_state = MockAppStateBuilder::new()
        .with_payment_channel_service(Arc::new(MockPaymentChannelService::new_success()));
    let app = build_test_router(app_state);

    let body = json!({ "name": "", "priority": 1 });
    let (status, _) = make_request(app, "POST", "/api/payment-channel", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ============================================================================
// GET /api/payment-channel
// ============================================================================

#[tokio::test]
async fn test_get_all_success() {
    let app_state = MockAppStateBuilder::new()
        .with_payment_channel_service(Arc::new(MockPaymentChannelService::new_success()));
    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/payment-channel", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert!(response["data"].as_array().unwrap().len() == 1);
    assert_eq!(response["data"][0]["name"].as_str().unwrap(), "Cash");
}

// ============================================================================
// GET /api/payment-channel/:id
// ============================================================================

#[tokio::test]
async fn test_get_by_id_found() {
    let app_state = MockAppStateBuilder::new()
        .with_payment_channel_service(Arc::new(MockPaymentChannelService::new_success()));
    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/payment-channel/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["id"].as_str().unwrap(), "1");
    assert_eq!(response["name"].as_str().unwrap(), "Cash");
}

#[tokio::test]
async fn test_get_by_id_not_found() {
    let app_state = MockAppStateBuilder::new()
        .with_payment_channel_service(Arc::new(MockPaymentChannelService::new_success()));
    let app = build_test_router(app_state);

    let (status, _) = make_request(app, "GET", "/api/payment-channel/999", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ============================================================================
// PUT /api/payment-channel/:id
// ============================================================================

#[tokio::test]
async fn test_update_success() {
    let app_state = MockAppStateBuilder::new()
        .with_payment_channel_service(Arc::new(MockPaymentChannelService::new_success()));
    let app = build_test_router(app_state);

    let body = json!({ "name": "QRIS", "priority": 2 });
    let (status, _) = make_request(app, "PUT", "/api/payment-channel/1", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_update_not_found() {
    let app_state = MockAppStateBuilder::new()
        .with_payment_channel_service(Arc::new(MockPaymentChannelService::new_success()));
    let app = build_test_router(app_state);

    let body = json!({ "name": "QRIS" });
    let (status, _) = make_request(app, "PUT", "/api/payment-channel/999", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ============================================================================
// DELETE /api/payment-channel/:id
// ============================================================================

#[tokio::test]
async fn test_delete_success() {
    let app_state = MockAppStateBuilder::new()
        .with_payment_channel_service(Arc::new(MockPaymentChannelService::new_success()));
    let app = build_test_router(app_state);

    let (status, _) = make_request(app, "DELETE", "/api/payment-channel/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_not_found() {
    let app_state = MockAppStateBuilder::new()
        .with_payment_channel_service(Arc::new(MockPaymentChannelService::new_success()));
    let app = build_test_router(app_state);

    let (status, _) = make_request(app, "DELETE", "/api/payment-channel/999", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ============================================================================
// PUT /api/payment-channel/priorities
// ============================================================================

#[tokio::test]
async fn test_update_priorities_success() {
    let app_state = MockAppStateBuilder::new()
        .with_payment_channel_service(Arc::new(MockPaymentChannelService::new_success()));
    let app = build_test_router(app_state);

    let body = json!({
        "channels": [
            { "id": "1", "priority": 2 },
            { "id": "2", "priority": 1 }
        ]
    });
    let (status, _) = make_request(app, "PUT", "/api/payment-channel/priorities", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
}
