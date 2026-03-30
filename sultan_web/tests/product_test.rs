mod common;

use axum::Router;
use axum::http::StatusCode;
use axum::middleware::from_fn;
use serde_json::json;
use std::sync::Arc;

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
async fn test_create_product_validation_error_empty_name() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "product": {
            "name": "",
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

    assert_eq!(status, StatusCode::BAD_REQUEST);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Name must be between 1 and 256 characters"),
        "Expected name validation error, got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_create_product_validation_error_name_too_long() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "product": {
            "name": "a".repeat(257),
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

    assert_eq!(status, StatusCode::BAD_REQUEST);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Name must be between 1 and 256 characters"),
        "Expected name validation error, got: {}",
        error_msg
    );
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

    let (status, response) = make_request(app, "POST", "/api/product", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Name must be between 1 and 256 characters"),
        "Expected name validation error, got: {}",
        error_msg
    );
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

    let (status, response) = make_request(app, "POST", "/api/product", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Product type must be between 1 and 50 characters"),
        "Expected product_type validation error, got: {}",
        error_msg
    );
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

// ============================================================================
// PATCH /api/product/{id} - Update Product Tests
// ============================================================================

#[tokio::test]
async fn test_update_product_success() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "name": "Updated Laptop",
        "sellable": false,
        "category_ids": ["1234567890"]
    });

    let (status, response) = make_request(app, "PATCH", "/api/product/1", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(response.is_null() || response.as_object().is_none_or(|o| o.is_empty()));
}

#[tokio::test]
async fn test_update_product_clear_nullable_field() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    // Explicitly clear description by sending null
    let body = json!({ "description": null });

    let (status, _) = make_request(app, "PATCH", "/api/product/1", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_update_product_empty_body() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    // Empty body is valid — all fields Unchanged
    let body = json!({});

    let (status, _) = make_request(app, "PATCH", "/api/product/1", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_update_product_not_found() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({ "name": "Updated Name" });

    let (status, response) = make_request(app, "PATCH", "/api/product/999", Some(body))
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
async fn test_update_product_validation_error_empty_name() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({ "name": "" });

    let (status, response) = make_request(app, "PATCH", "/api/product/1", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Name must be between 1 and 256 characters"),
        "Expected name validation error, got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_update_product_service_error() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_failure()));

    let app = build_test_router(app_state);

    let body = json!({ "name": "Updated Name" });

    let (status, response) = make_request(app, "PATCH", "/api/product/1", Some(body))
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
// GET /api/product - List Products Tests
// ============================================================================

#[tokio::test]
async fn test_get_all_products_success() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/product", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert!(response.get("items").is_some());
    assert!(response["items"].is_array());
    assert!(response.get("next_cursor").is_some());
}

#[tokio::test]
async fn test_get_all_products_with_query_params() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(
        app,
        "GET",
        "/api/product?name=Laptop&sort_field=name&sort_direction=asc&limit=10",
        None,
    )
    .await
    .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert!(response["items"].is_array());
}

#[tokio::test]
async fn test_get_all_products_invalid_sort_field() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/product?sort_field=invalid", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Invalid sort_field"),
        "Expected sort_field validation error, got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_get_all_products_invalid_cursor() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) =
        make_request(app, "GET", "/api/product?cursor=not-valid-base64!!!", None)
            .await
            .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("cursor"),
        "Expected cursor validation error, got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_get_all_products_service_error() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_failure()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/product", None)
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
// POST /api/product/{id}/variant - Create Variant Tests
// ============================================================================

#[tokio::test]
async fn test_create_variant_success() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "variant": {
            "product_id": "1",
            "barcode": "1234567890",
            "name": "Red - L"
        },
        "sell_prices": [],
        "stocks": []
    });

    let (status, response) = make_request(app, "POST", "/api/product/1/variant", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::CREATED);
    assert!(response["id"].as_str().is_some(), "Expected id in response");
}

#[tokio::test]
async fn test_create_variant_path_id_overrides_body_product_id() {
    // Path id (1) should override whatever product_id is in the body
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "variant": {
            "product_id": "999",
            "barcode": null,
            "name": "Variant A"
        },
        "sell_prices": [],
        "stocks": []
    });

    let (status, _response) = make_request(app, "POST", "/api/product/1/variant", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn test_create_variant_service_error() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_failure()));

    let app = build_test_router(app_state);

    let body = json!({
        "variant": {
            "product_id": "1",
            "barcode": null,
            "name": null
        },
        "sell_prices": [],
        "stocks": []
    });

    let (status, response) = make_request(app, "POST", "/api/product/1/variant", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response["error"].as_str().is_some());
}

// ============================================================================
// PATCH /api/product/{id}/variant/{variant_id} - Update Variant Tests
// ============================================================================

#[tokio::test]
async fn test_update_variant_success() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({ "name": "Blue - XL" });

    let (status, _) = make_request(app, "PATCH", "/api/product/1/variant/1", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_update_variant_clear_nullable_field() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    // Send null to clear barcode
    let body = json!({ "barcode": null });

    let (status, _) = make_request(app, "PATCH", "/api/product/1/variant/1", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_update_variant_not_found() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({ "name": "Red" });

    let (status, response) = make_request(app, "PATCH", "/api/product/1/variant/9999", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(response["error"].as_str().is_some());
}

#[tokio::test]
async fn test_update_variant_service_error() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_failure()));

    let app = build_test_router(app_state);

    let body = json!({ "name": "Red" });

    let (status, response) = make_request(app, "PATCH", "/api/product/1/variant/1", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response["error"].as_str().is_some());
}

// ============================================================================
// DELETE /api/product/{id}/variant/{variant_id} - Delete Variant Tests
// ============================================================================

#[tokio::test]
async fn test_delete_variant_success() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let (status, _) = make_request(app, "DELETE", "/api/product/1/variant/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_variant_not_found() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "DELETE", "/api/product/1/variant/9999", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(response["error"].as_str().is_some());
}

#[tokio::test]
async fn test_delete_variant_service_error() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_failure()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "DELETE", "/api/product/1/variant/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response["error"].as_str().is_some());
}

// ============================================================================
// POST /api/product/{id}/variant/{variant_id}/price - Create Sell Price Tests
// ============================================================================

#[tokio::test]
async fn test_create_sell_price_success() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "sell_price": {
            "branch_id": null,
            "product_variant_id": "1",
            "uom_id": "1",
            "quantity": 1,
            "price": 150000,
            "metadata": null
        },
        "discounts": []
    });

    let (status, response) =
        make_request(app, "POST", "/api/product/1/variant/1/price", Some(body))
            .await
            .expect("Request failed");

    assert_eq!(status, StatusCode::CREATED);
    assert!(response.get("id").is_some());
    assert_eq!(response["id"].as_str().unwrap(), "1");
}

#[tokio::test]
async fn test_create_sell_price_service_error() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_failure()));

    let app = build_test_router(app_state);

    let body = json!({
        "sell_price": {
            "product_variant_id": "1",
            "uom_id": "1",
            "quantity": 1,
            "price": 150000
        },
        "discounts": []
    });

    let (status, response) =
        make_request(app, "POST", "/api/product/1/variant/1/price", Some(body))
            .await
            .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response["error"].as_str().is_some());
}

#[tokio::test]
async fn test_create_sell_price_invalid_quantity_zero() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "sell_price": {
            "product_variant_id": "1",
            "uom_id": "1",
            "quantity": 0,
            "price": 150000
        }
    });

    let (status, response) =
        make_request(app, "POST", "/api/product/1/variant/1/price", Some(body))
            .await
            .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["error"].as_str().is_some());
}

#[tokio::test]
async fn test_create_sell_price_invalid_price_zero() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "sell_price": {
            "product_variant_id": "1",
            "uom_id": "1",
            "quantity": 1,
            "price": 0
        }
    });

    let (status, response) =
        make_request(app, "POST", "/api/product/1/variant/1/price", Some(body))
            .await
            .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["error"].as_str().is_some());
}

#[tokio::test]
async fn test_create_sell_price_invalid_discount_quantity_zero() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "sell_price": {
            "product_variant_id": "1",
            "uom_id": "1",
            "quantity": 1,
            "price": 150000
        },
        "discounts": [{
            "price_id": "1",
            "quantity": 0,
            "discount_formula": "price * 0.9"
        }]
    });

    let (status, response) =
        make_request(app, "POST", "/api/product/1/variant/1/price", Some(body))
            .await
            .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["error"].as_str().is_some());
}

#[tokio::test]
async fn test_create_sell_price_invalid_discount_formula_empty() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "sell_price": {
            "product_variant_id": "1",
            "uom_id": "1",
            "quantity": 1,
            "price": 150000
        },
        "discounts": [{
            "price_id": "1",
            "quantity": 10,
            "discount_formula": ""
        }]
    });

    let (status, response) =
        make_request(app, "POST", "/api/product/1/variant/1/price", Some(body))
            .await
            .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["error"].as_str().is_some());
}

#[tokio::test]
async fn test_update_sell_price_success() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({ "price": 175000 });

    let (status, response) =
        make_request(app, "PATCH", "/api/product/1/variant/1/price/1", Some(body))
            .await
            .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(response.is_null() || response.as_object().is_none_or(|o| o.is_empty()));
}

#[tokio::test]
async fn test_update_sell_price_not_found() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({ "price": 175000 });

    let (status, response) = make_request(
        app,
        "PATCH",
        "/api/product/1/variant/1/price/9999",
        Some(body),
    )
    .await
    .expect("Request failed");

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(response["error"].as_str().is_some());
}

#[tokio::test]
async fn test_update_sell_price_service_error() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_failure()));

    let app = build_test_router(app_state);

    let body = json!({ "price": 175000 });

    let (status, response) =
        make_request(app, "PATCH", "/api/product/1/variant/1/price/1", Some(body))
            .await
            .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response["error"].as_str().is_some());
}

#[tokio::test]
async fn test_update_sell_price_clear_metadata() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({ "metadata": null });

    let (status, _) = make_request(app, "PATCH", "/api/product/1/variant/1/price/1", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_update_sell_price_invalid_quantity_zero() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({ "quantity": 0 });

    let (status, response) =
        make_request(app, "PATCH", "/api/product/1/variant/1/price/1", Some(body))
            .await
            .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["error"].as_str().is_some());
}

#[tokio::test]
async fn test_update_sell_price_invalid_price_zero() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({ "price": 0 });

    let (status, response) =
        make_request(app, "PATCH", "/api/product/1/variant/1/price/1", Some(body))
            .await
            .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["error"].as_str().is_some());
}

// ============================================================================
// DELETE /api/product/{id}/variant/{variant_id}/price/{price_id} - Delete Sell Price Tests
// ============================================================================

#[tokio::test]
async fn test_delete_sell_price_success() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "DELETE", "/api/product/1/variant/1/price/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(response.is_null() || response.as_object().is_none_or(|o| o.is_empty()));
}

#[tokio::test]
async fn test_delete_sell_price_not_found() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) =
        make_request(app, "DELETE", "/api/product/1/variant/1/price/9999", None)
            .await
            .expect("Request failed");

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(response["error"].as_str().is_some());
}

#[tokio::test]
async fn test_delete_sell_price_service_error() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_failure()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "DELETE", "/api/product/1/variant/1/price/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response["error"].as_str().is_some());
}

// ============================================================================
// POST /api/product/{id}/variant/{variant_id}/price/{price_id}/discount - Create Sell Discount Tests
// ============================================================================

#[tokio::test]
async fn test_create_sell_discount_success() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "price_id": "1",
        "quantity": 10,
        "discount_formula": "price * 0.9"
    });

    let (status, response) = make_request(
        app,
        "POST",
        "/api/product/1/variant/1/price/1/discount",
        Some(body),
    )
    .await
    .expect("Request failed");

    assert_eq!(status, StatusCode::CREATED);
    assert!(response.get("id").is_some());
    assert_eq!(response["id"].as_str().unwrap(), "1");
}

#[tokio::test]
async fn test_create_sell_discount_service_error() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_failure()));

    let app = build_test_router(app_state);

    let body = json!({
        "price_id": "1",
        "quantity": 10,
        "discount_formula": "price * 0.9"
    });

    let (status, response) = make_request(
        app,
        "POST",
        "/api/product/1/variant/1/price/1/discount",
        Some(body),
    )
    .await
    .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response["error"].as_str().is_some());
}

#[tokio::test]
async fn test_create_sell_discount_invalid_quantity_zero() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "price_id": "1",
        "quantity": 0,
        "discount_formula": "price * 0.9"
    });

    let (status, response) = make_request(
        app,
        "POST",
        "/api/product/1/variant/1/price/1/discount",
        Some(body),
    )
    .await
    .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["error"].as_str().is_some());
}

#[tokio::test]
async fn test_create_sell_discount_invalid_empty_formula() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "price_id": "1",
        "quantity": 10,
        "discount_formula": ""
    });

    let (status, response) = make_request(
        app,
        "POST",
        "/api/product/1/variant/1/price/1/discount",
        Some(body),
    )
    .await
    .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["error"].as_str().is_some());
}

// ============================================================================
// PATCH /api/product/{id}/variant/{variant_id}/price/{price_id}/discount/{discount_id} - Update Sell Discount Tests
// ============================================================================

#[tokio::test]
async fn test_update_sell_discount_success() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "quantity": 5,
        "discount_formula": "price * 0.8"
    });

    let (status, response) = make_request(
        app,
        "PATCH",
        "/api/product/1/variant/1/price/1/discount/1",
        Some(body),
    )
    .await
    .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(response.is_null() || response.as_object().is_none_or(|o| o.is_empty()));
}

#[tokio::test]
async fn test_update_sell_discount_not_found() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({ "quantity": 5 });

    let (status, response) = make_request(
        app,
        "PATCH",
        "/api/product/1/variant/1/price/1/discount/9999",
        Some(body),
    )
    .await
    .expect("Request failed");

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(response["error"].as_str().is_some());
}

#[tokio::test]
async fn test_update_sell_discount_service_error() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_failure()));

    let app = build_test_router(app_state);

    let body = json!({ "quantity": 5 });

    let (status, response) = make_request(
        app,
        "PATCH",
        "/api/product/1/variant/1/price/1/discount/1",
        Some(body),
    )
    .await
    .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response["error"].as_str().is_some());
}

#[tokio::test]
async fn test_update_sell_discount_invalid_quantity_zero() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({ "quantity": 0 });

    let (status, response) = make_request(
        app,
        "PATCH",
        "/api/product/1/variant/1/price/1/discount/1",
        Some(body),
    )
    .await
    .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["error"].as_str().is_some());
}

#[tokio::test]
async fn test_update_sell_discount_invalid_empty_formula() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({ "discount_formula": "" });

    let (status, response) = make_request(
        app,
        "PATCH",
        "/api/product/1/variant/1/price/1/discount/1",
        Some(body),
    )
    .await
    .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["error"].as_str().is_some());
}

// ============================================================================
// DELETE /api/product/{id}/variant/{variant_id}/price/{price_id}/discount/{discount_id} - Delete Sell Discount Tests
// ============================================================================

#[tokio::test]
async fn test_delete_sell_discount_success() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(
        app,
        "DELETE",
        "/api/product/1/variant/1/price/1/discount/1",
        None,
    )
    .await
    .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(response.is_null() || response.as_object().is_none_or(|o| o.is_empty()));
}

#[tokio::test]
async fn test_delete_sell_discount_not_found() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(
        app,
        "DELETE",
        "/api/product/1/variant/1/price/1/discount/9999",
        None,
    )
    .await
    .expect("Request failed");

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(response["error"].as_str().is_some());
}

#[tokio::test]
async fn test_delete_sell_discount_service_error() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_failure()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(
        app,
        "DELETE",
        "/api/product/1/variant/1/price/1/discount/1",
        None,
    )
    .await
    .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response["error"].as_str().is_some());
}

// ============================================================================
// GET /api/product/search-variant - Search Variants Tests
// ============================================================================

#[tokio::test]
async fn test_search_variants_success() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/product/search-variant", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert!(response["items"].is_array());
    assert!(response["next_cursor"].is_null() || response["next_cursor"].is_string());
}

#[tokio::test]
async fn test_search_variants_with_filters() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(
        app,
        "GET",
        "/api/product/search-variant?name=Test&product_type=goods&sort_field=name&sort_direction=asc&limit=10",
        None,
    )
    .await
    .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert!(response["items"].is_array());
}

#[tokio::test]
async fn test_search_variants_invalid_sort_field() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(
        app,
        "GET",
        "/api/product/search-variant?sort_field=invalid",
        None,
    )
    .await
    .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["error"].as_str().is_some());
}

#[tokio::test]
async fn test_search_variants_invalid_sort_direction() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(
        app,
        "GET",
        "/api/product/search-variant?sort_direction=invalid",
        None,
    )
    .await
    .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["error"].as_str().is_some());
}

#[tokio::test]
async fn test_search_variants_service_error() {
    let app_state = MockAppStateBuilder::new()
        .with_product_service(Arc::new(MockProductService::new_failure()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/product/search-variant", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response["error"].as_str().is_some());
}
