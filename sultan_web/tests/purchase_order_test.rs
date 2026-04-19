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
        .nest(
            "/api/branch/{branch_id}/purchase-order",
            purchase_order_router(),
        )
        .layer(from_fn(context_middleware))
        .with_state(app_state.build())
}

// ============================================================================
// POST /api/branch/{branch_id}/purchase-order - Create Purchase Order Tests
// ============================================================================

#[tokio::test]
async fn test_create_purchase_order_success() {
    let app_state = MockAppStateBuilder::new()
        .with_purchase_order_service(Arc::new(MockPurchaseOrderService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "supplier_id": null,
        "reference_number": null,
        "order_date": null,
        "expected_date": null,
        "payment_due_date": null,
        "discount_amount": 0,
        "notes": null,
        "metadata": null
    });

    let (status, response) = make_request(app, "POST", "/api/branch/1/purchase-order", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::CREATED);
    assert!(response.get("id").is_some());
    assert_eq!(response["id"].as_str().unwrap(), "1");
}

#[tokio::test]
async fn test_create_purchase_order_success_with_all_fields() {
    let app_state = MockAppStateBuilder::new()
        .with_purchase_order_service(Arc::new(MockPurchaseOrderService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "supplier_id": "5",
        "reference_number": "PO-0001",
        "order_date": "2026-04-01",
        "expected_date": "2026-04-15",
        "payment_due_date": "2026-04-30",
        "discount_amount": 500,
        "notes": "Urgent order",
        "metadata": { "department": "warehouse" }
    });

    let (status, response) = make_request(app, "POST", "/api/branch/1/purchase-order", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(response["id"].as_str().unwrap(), "1");
}

#[tokio::test]
async fn test_create_purchase_order_service_error() {
    let app_state = MockAppStateBuilder::new()
        .with_purchase_order_service(Arc::new(MockPurchaseOrderService::new_failure()));

    let app = build_test_router(app_state);

    let body = json!({});

    let (status, response) = make_request(app, "POST", "/api/branch/1/purchase-order", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response.get("error").is_some());
}

#[tokio::test]
async fn test_create_purchase_order_validation_reference_number_empty() {
    let app_state = MockAppStateBuilder::new()
        .with_purchase_order_service(Arc::new(MockPurchaseOrderService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({ "reference_number": "" });

    let (status, response) = make_request(app, "POST", "/api/branch/1/purchase-order", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        response["error"]
            .as_str()
            .unwrap()
            .contains("Number must be between 1 and 100 characters")
    );
}

#[tokio::test]
async fn test_create_purchase_order_validation_reference_number_too_long() {
    let app_state = MockAppStateBuilder::new()
        .with_purchase_order_service(Arc::new(MockPurchaseOrderService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "reference_number": "a".repeat(101)
    });

    let (status, response) = make_request(app, "POST", "/api/branch/1/purchase-order", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        response["error"]
            .as_str()
            .unwrap()
            .contains("Number must be between 1 and 100 characters")
    );
}

#[tokio::test]
async fn test_create_purchase_order_validation_discount_negative() {
    let app_state = MockAppStateBuilder::new()
        .with_purchase_order_service(Arc::new(MockPurchaseOrderService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "discount_amount": -1
    });

    let (status, response) = make_request(app, "POST", "/api/branch/1/purchase-order", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        response["error"]
            .as_str()
            .unwrap()
            .contains("Discount amount must be non-negative")
    );
}

// ============================================================================
// PUT /api/branch/{branch_id}/purchase-order/{id} - Update Purchase Order Tests
// ============================================================================

#[tokio::test]
async fn test_update_purchase_order_success() {
    let app_state = MockAppStateBuilder::new()
        .with_purchase_order_service(Arc::new(MockPurchaseOrderService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "supplier_id": "5",
        "reference_number": "PO-0002"
    });

    let (status, _response) =
        make_request(app, "PUT", "/api/branch/1/purchase-order/1", Some(body))
            .await
            .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_update_purchase_order_success_empty_body() {
    let app_state = MockAppStateBuilder::new()
        .with_purchase_order_service(Arc::new(MockPurchaseOrderService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({});

    let (status, _response) =
        make_request(app, "PUT", "/api/branch/1/purchase-order/42", Some(body))
            .await
            .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_update_purchase_order_success_with_all_fields() {
    let app_state = MockAppStateBuilder::new()
        .with_purchase_order_service(Arc::new(MockPurchaseOrderService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "supplier_id": "3",
        "reference_number": "PO-9999",
        "order_date": "2026-05-01",
        "expected_date": "2026-05-15",
        "payment_due_date": "2026-05-31",
        "notes": "Updated notes",
        "metadata": { "updated": true }
    });

    let (status, _response) =
        make_request(app, "PUT", "/api/branch/1/purchase-order/7", Some(body))
            .await
            .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_update_purchase_order_success_clear_nullable_fields() {
    let app_state = MockAppStateBuilder::new()
        .with_purchase_order_service(Arc::new(MockPurchaseOrderService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "reference_number": null,
        "order_date": null,
        "expected_date": null,
        "payment_due_date": null,
        "notes": null,
        "metadata": null
    });

    let (status, _response) =
        make_request(app, "PUT", "/api/branch/1/purchase-order/7", Some(body))
            .await
            .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_update_purchase_order_service_error() {
    let app_state = MockAppStateBuilder::new()
        .with_purchase_order_service(Arc::new(MockPurchaseOrderService::new_failure()));

    let app = build_test_router(app_state);

    let body = json!({});

    let (status, response) = make_request(app, "PUT", "/api/branch/1/purchase-order/1", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response.get("error").is_some());
}

#[tokio::test]
async fn test_update_purchase_order_validation_supplier_id_zero() {
    let app_state = MockAppStateBuilder::new()
        .with_purchase_order_service(Arc::new(MockPurchaseOrderService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "supplier_id": "0"
    });

    let (status, response) = make_request(app, "PUT", "/api/branch/1/purchase-order/1", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        response["error"]
            .as_str()
            .unwrap()
            .contains("Supplier ID must be greater than 0")
    );
}

#[tokio::test]
async fn test_update_purchase_order_validation_supplier_id_negative() {
    let app_state = MockAppStateBuilder::new()
        .with_purchase_order_service(Arc::new(MockPurchaseOrderService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "supplier_id": "-1"
    });

    let (status, response) = make_request(app, "PUT", "/api/branch/1/purchase-order/1", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        response["error"]
            .as_str()
            .unwrap()
            .contains("Supplier ID must be greater than 0")
    );
}
