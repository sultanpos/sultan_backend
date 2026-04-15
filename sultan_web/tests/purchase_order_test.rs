mod common;

use axum::Router;
use axum::http::StatusCode;
use axum::middleware::from_fn;
use serde_json::json;
use std::sync::Arc;

use common::{
    MockAppStateBuilder, make_request, mock_purchase_order_service::MockPurchaseOrderService,
};
use sultan_web::handler::middleware::context_middleware;
use sultan_web::handler::purchase_order_router::purchase_order_router;

fn build_test_router(app_state: MockAppStateBuilder) -> Router {
    Router::new()
        .nest("/api/purchase-order", purchase_order_router())
        .layer(from_fn(context_middleware))
        .with_state(app_state.build())
}

// ============================================================================
// POST /api/purchase-order - Create Purchase Order Tests
// ============================================================================

#[tokio::test]
async fn test_create_purchase_order_success() {
    let app_state = MockAppStateBuilder::new()
        .with_purchase_order_service(Arc::new(MockPurchaseOrderService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "branch_id": 1,
        "supplier_id": null,
        "number": "PO-0001",
        "reference_number": null,
        "order_date": null,
        "expected_date": null,
        "payment_due_date": null,
        "discount_amount": 0,
        "notes": null,
        "metadata": null
    });

    let (status, response) = make_request(app, "POST", "/api/purchase-order", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::CREATED);
    assert!(response.get("id").is_some());
    assert_eq!(response["id"].as_str().unwrap(), "1");
}

#[tokio::test]
async fn test_create_purchase_order_validation_error_empty_number() {
    let app_state = MockAppStateBuilder::new()
        .with_purchase_order_service(Arc::new(MockPurchaseOrderService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "branch_id": 1,
        "number": ""
    });

    let (status, response) = make_request(app, "POST", "/api/purchase-order", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        response["error"]
            .as_str()
            .unwrap()
            .contains("Number must be")
    );
}

#[tokio::test]
async fn test_create_purchase_order_service_error() {
    let app_state = MockAppStateBuilder::new()
        .with_purchase_order_service(Arc::new(MockPurchaseOrderService::new_failure()));

    let app = build_test_router(app_state);

    let body = json!({
        "branch_id": 1,
        "number": "PO-0001"
    });

    let (status, _response) = make_request(app, "POST", "/api/purchase-order", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
}
