mod common;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::middleware::from_fn;
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;

use common::{MockAppStateBuilder, make_request, mock_product_service::MockProductService};
use sultan_web::{handler::product_router::product_router, middleware::context_middleware};

// ============================================================================
// Helper
// ============================================================================

/// Helper function to build a test router with the context middleware
fn build_test_router(app_state: MockAppStateBuilder) -> Router {
    Router::new()
        .nest("/api/product", product_router())
        .layer(from_fn(context_middleware))
        .with_state(app_state.build())
}

// ============================================================================
// POST /api/product - Create Product Tests
// ============================================================================

#[tokio::test]
async fn test_create_product_success() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "product": {
            "name": "Laptop ASUS ROG",
            "description": "High-performance gaming laptop",
            "product_type": "goods",
            "main_image": null,
            "sellable": true,
            "buyable": true,
            "editable_price": false,
            "has_variant": false,
            "metadata": null,
            "category_ids": []
        },
        "variants": [],
        "categories": []
    });

    let (status, response) = make_request(app, "POST", "/api/product", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::CREATED);
    assert!(response.get("id").is_some());
    assert_eq!(response["id"].as_str().unwrap(), "1");
}

#[tokio::test]
async fn test_create_product_validation_error_missing_name() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    // Missing required "name" field in product
    let body = json!({
        "product": {
            "description": null,
            "product_type": "goods",
            "main_image": null,
            "sellable": true,
            "buyable": true,
            "editable_price": false,
            "has_variant": false,
            "metadata": null,
            "category_ids": []
        },
        "variants": [],
        "categories": []
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/product")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_create_product_validation_error_missing_product_type() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    // Missing required "product_type" field in product
    let body = json!({
        "product": {
            "name": "Laptop",
            "description": null,
            "main_image": null,
            "sellable": true,
            "buyable": true,
            "editable_price": false,
            "has_variant": false,
            "metadata": null,
            "category_ids": []
        },
        "variants": [],
        "categories": []
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/product")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_create_product_service_error() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_failure()));

    let app = build_test_router(app_state);

    let body = json!({
        "product": {
            "name": "Laptop ASUS ROG",
            "description": null,
            "product_type": "goods",
            "main_image": null,
            "sellable": true,
            "buyable": true,
            "editable_price": false,
            "has_variant": false,
            "metadata": null,
            "category_ids": []
        },
        "variants": [],
        "categories": []
    });

    let (status, response) = make_request(app, "POST", "/api/product", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Internal error"),
        "Expected internal error, got: {}",
        error_msg
    );
}

// ============================================================================
// GET /api/product/{id} - Get Product By ID Tests
// ============================================================================

#[tokio::test]
async fn test_get_product_by_id_success() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/product/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["id"].as_str().unwrap(), "1");
    assert_eq!(response["name"].as_str().unwrap(), "Test Product");
    assert_eq!(
        response["description"].as_str().unwrap(),
        "Test Description"
    );
    assert_eq!(response["product_type"].as_str().unwrap(), "goods");
    assert!(response["sellable"].as_bool().unwrap());
    assert!(response["buyable"].as_bool().unwrap());
    assert!(!response["editable_price"].as_bool().unwrap());
    assert!(!response["has_variant"].as_bool().unwrap());
    assert!(response["categories"].as_array().unwrap().is_empty());
    assert!(response["variants"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_get_product_by_id_not_found() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/product/999", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NOT_FOUND);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Product with id 999 not found"),
        "Expected not found error, got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_get_product_by_id_service_error() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_failure()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/product/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Internal error"),
        "Expected internal error, got: {}",
        error_msg
    );
}

// ============================================================================
// DELETE /api/product/{id} - Delete Product Tests
// ============================================================================

#[tokio::test]
async fn test_delete_product_success() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "DELETE", "/api/product/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(response.is_null() || response.as_object().is_none_or(|o| o.is_empty()));
}

#[tokio::test]
async fn test_delete_product_not_found() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "DELETE", "/api/product/999", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NOT_FOUND);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Product with id 999 not found"),
        "Expected not found error, got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_delete_product_service_error() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_failure()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "DELETE", "/api/product/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Internal error"),
        "Expected internal error, got: {}",
        error_msg
    );
}
