mod common;

use axum::Router;
use axum::http::StatusCode;
use axum::middleware::from_fn;
use serde_json::json;
use std::sync::Arc;

use common::{MockAppStateBuilder, make_request, mock_supplier_service::MockSupplierService};
use sultan_web::{handler::supplier_routes::supplier_router, middleware::context_middleware};

// ============================================================================
// Helper Functions
// ============================================================================

/// Helper function to build a test router with the context middleware
fn build_test_router(app_state: MockAppStateBuilder) -> Router {
    Router::new()
        .nest("/api/supplier", supplier_router())
        .layer(from_fn(context_middleware))
        .with_state(app_state.build())
}

// ============================================================================
// POST /api/supplier - Create Supplier Tests
// ============================================================================

#[tokio::test]
async fn test_create_supplier_success() {
    // Setup
    let app_state = MockAppStateBuilder::new()
        .with_supplier_service(Arc::new(MockSupplierService::new_success()));

    let app = build_test_router(app_state);

    // Make create request
    let body = json!({
        "name": "PT. Test Supplier",
        "code": "SUP001",
        "email": "supplier@test.com",
        "address": "Jakarta",
        "phone": "08123456789",
        "npwp": "12.345.678.9-012.000",
        "npwp_name": "PT Test Supplier",
        "metadata": null
    });

    let (status, response) = make_request(app, "POST", "/api/supplier", Some(body))
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::CREATED);
    assert!(response.get("id").is_some());
    assert_eq!(response["id"].as_i64().unwrap(), 1);
}

#[tokio::test]
async fn test_create_supplier_validation_error_empty_name() {
    // Setup
    let app_state = MockAppStateBuilder::new()
        .with_supplier_service(Arc::new(MockSupplierService::new_success()));

    let app = build_test_router(app_state);

    // Make create request with empty name
    let body = json!({
        "name": "",
        "code": "SUP001"
    });

    let (status, response) = make_request(app, "POST", "/api/supplier", Some(body))
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Name must be between 1 and 256 characters"),
        "Expected validation error for name, got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_create_supplier_validation_error_name_too_long() {
    // Setup
    let app_state = MockAppStateBuilder::new()
        .with_supplier_service(Arc::new(MockSupplierService::new_success()));

    let app = build_test_router(app_state);

    // Make create request with name that's too long (> 100 chars)
    let long_name = "a".repeat(257);
    let body = json!({
        "name": long_name,
        "code": "SUP001"
    });

    let (status, response) = make_request(app, "POST", "/api/supplier", Some(body))
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Name must be between 1 and 256 characters"),
        "Expected validation error for name, got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_create_supplier_service_error() {
    // Setup - use mock supplier service that returns error
    let app_state = MockAppStateBuilder::new()
        .with_supplier_service(Arc::new(MockSupplierService::new_failure()));

    let app = build_test_router(app_state);

    // Make create request
    let body = json!({
        "name": "PT. Test Supplier",
        "code": "SUP001"
    });

    let (status, response) = make_request(app, "POST", "/api/supplier", Some(body))
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Internal error"),
        "Expected internal error message, got: {}",
        error_msg
    );
}

// ============================================================================
// PUT /api/supplier/{id} - Update Supplier Tests
// ============================================================================

#[tokio::test]
async fn test_update_supplier_success() {
    // Setup
    let app_state = MockAppStateBuilder::new()
        .with_supplier_service(Arc::new(MockSupplierService::new_success()));

    let app = build_test_router(app_state);

    // Make update request - only include fields that don't use Update<T> wrapper
    let body = json!({
        "name": "PT. Updated Supplier"
    });

    let (status, _response) = make_request(app, "PUT", "/api/supplier/1", Some(body))
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_update_supplier_not_found() {
    // Setup
    let app_state = MockAppStateBuilder::new()
        .with_supplier_service(Arc::new(MockSupplierService::new_success()));

    let app = build_test_router(app_state);

    // Make update request for non-existent supplier
    let body = json!({
        "name": "PT. Updated Supplier"
    });

    let (status, response) = make_request(app, "PUT", "/api/supplier/999", Some(body))
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::NOT_FOUND);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Supplier with id 999 not found"),
        "Expected not found error, got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_update_supplier_service_error() {
    // Setup - use mock supplier service that returns error
    let app_state = MockAppStateBuilder::new()
        .with_supplier_service(Arc::new(MockSupplierService::new_failure()));

    let app = build_test_router(app_state);

    // Make update request
    let body = json!({
        "name": "PT. Updated Supplier"
    });

    let (status, response) = make_request(app, "PUT", "/api/supplier/1", Some(body))
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Internal error"),
        "Expected internal error message, got: {}",
        error_msg
    );
}

// ============================================================================
// DELETE /api/supplier/{id} - Delete Supplier Tests
// ============================================================================

#[tokio::test]
async fn test_delete_supplier_success() {
    // Setup
    let app_state = MockAppStateBuilder::new()
        .with_supplier_service(Arc::new(MockSupplierService::new_success()));

    let app = build_test_router(app_state);

    // Make delete request
    let (status, _response) = make_request(app, "DELETE", "/api/supplier/1", None)
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_supplier_not_found() {
    // Setup
    let app_state = MockAppStateBuilder::new()
        .with_supplier_service(Arc::new(MockSupplierService::new_success()));

    let app = build_test_router(app_state);

    // Make delete request for non-existent supplier
    let (status, response) = make_request(app, "DELETE", "/api/supplier/999", None)
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::NOT_FOUND);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Supplier with id 999 not found"),
        "Expected not found error, got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_delete_supplier_service_error() {
    // Setup - use mock supplier service that returns error
    let app_state = MockAppStateBuilder::new()
        .with_supplier_service(Arc::new(MockSupplierService::new_failure()));

    let app = build_test_router(app_state);

    // Make delete request
    let (status, response) = make_request(app, "DELETE", "/api/supplier/1", None)
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Internal error"),
        "Expected internal error message, got: {}",
        error_msg
    );
}

// ============================================================================
// GET /api/supplier/{id} - Get Supplier By ID Tests
// ============================================================================

#[tokio::test]
async fn test_get_supplier_by_id_success() {
    // Setup
    let app_state = MockAppStateBuilder::new()
        .with_supplier_service(Arc::new(MockSupplierService::new_success()));

    let app = build_test_router(app_state);

    // Make get request
    let (status, response) = make_request(app, "GET", "/api/supplier/1", None)
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["name"].as_str().unwrap(), "PT. Test Supplier");
    assert_eq!(response["code"].as_str().unwrap(), "SUP001");
    assert_eq!(response["email"].as_str().unwrap(), "supplier@test.com");
    assert_eq!(response["phone"].as_str().unwrap(), "08123456789");
}

#[tokio::test]
async fn test_get_supplier_by_id_not_found() {
    // Setup
    let app_state = MockAppStateBuilder::new()
        .with_supplier_service(Arc::new(MockSupplierService::new_success()));

    let app = build_test_router(app_state);

    // Make get request for non-existent supplier
    let (status, response) = make_request(app, "GET", "/api/supplier/999", None)
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::NOT_FOUND);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Supplier with id 999 not found"),
        "Expected not found error, got: {}",
        error_msg
    );
}

// ============================================================================
// GET /api/supplier - Get All Suppliers Tests
// ============================================================================

#[tokio::test]
async fn test_get_all_suppliers_success() {
    // Setup
    let app_state = MockAppStateBuilder::new()
        .with_supplier_service(Arc::new(MockSupplierService::new_success()));

    let app = build_test_router(app_state);

    // Make get all request
    let (status, response) = make_request(app, "GET", "/api/supplier", None)
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::OK);
    let data = response["data"].as_array().unwrap();
    assert_eq!(data.len(), 2);

    // Check first supplier
    assert_eq!(data[0]["name"].as_str().unwrap(), "PT. Test Supplier");
    assert_eq!(data[0]["code"].as_str().unwrap(), "SUP001");

    // Check second supplier
    assert_eq!(data[1]["name"].as_str().unwrap(), "CV. Another Supplier");
    assert_eq!(data[1]["code"].as_str().unwrap(), "SUP002");
}

#[tokio::test]
async fn test_get_all_suppliers_empty() {
    // Setup
    let app_state = MockAppStateBuilder::new()
        .with_supplier_service(Arc::new(MockSupplierService::new_success()));

    let app = build_test_router(app_state);

    // Make get all request with filter that returns empty
    let (status, response) = make_request(app, "GET", "/api/supplier?name=empty", None)
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::OK);
    let data = response["data"].as_array().unwrap();
    assert_eq!(data.len(), 0);
}

#[tokio::test]
async fn test_get_all_suppliers_with_name_filter() {
    // Setup
    let app_state = MockAppStateBuilder::new()
        .with_supplier_service(Arc::new(MockSupplierService::new_success()));

    let app = build_test_router(app_state);

    // Make get all request with name filter
    let (status, response) = make_request(app, "GET", "/api/supplier?name=PT", None)
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::OK);
    let data = response["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["name"].as_str().unwrap(), "PT. Test Supplier");
}

#[tokio::test]
async fn test_get_all_suppliers_with_code_filter() {
    // Setup
    let app_state = MockAppStateBuilder::new()
        .with_supplier_service(Arc::new(MockSupplierService::new_success()));

    let app = build_test_router(app_state);

    // Make get all request with code filter
    let (status, response) = make_request(app, "GET", "/api/supplier?code=SUP001", None)
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::OK);
    let data = response["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["code"].as_str().unwrap(), "SUP001");
}

#[tokio::test]
async fn test_get_all_suppliers_with_phone_filter() {
    // Setup
    let app_state = MockAppStateBuilder::new()
        .with_supplier_service(Arc::new(MockSupplierService::new_success()));

    let app = build_test_router(app_state);

    // Make get all request with phone filter
    let (status, response) = make_request(app, "GET", "/api/supplier?phone=08123456789", None)
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::OK);
    let data = response["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["phone"].as_str().unwrap(), "08123456789");
}

#[tokio::test]
async fn test_get_all_suppliers_with_email_filter() {
    // Setup
    let app_state = MockAppStateBuilder::new()
        .with_supplier_service(Arc::new(MockSupplierService::new_success()));

    let app = build_test_router(app_state);

    // Make get all request with email filter
    let (status, response) =
        make_request(app, "GET", "/api/supplier?email=supplier@test.com", None)
            .await
            .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::OK);
    let data = response["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["email"].as_str().unwrap(), "supplier@test.com");
}

#[tokio::test]
async fn test_get_all_suppliers_with_npwp_filter() {
    // Setup
    let app_state = MockAppStateBuilder::new()
        .with_supplier_service(Arc::new(MockSupplierService::new_success()));

    let app = build_test_router(app_state);

    // Make get all request with npwp filter
    let (status, response) = make_request(app, "GET", "/api/supplier?npwp=12.345", None)
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::OK);
    let data = response["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["npwp"].as_str().unwrap(), "12.345.678.9-012.000");
}

#[tokio::test]
async fn test_get_all_suppliers_with_pagination() {
    // Setup
    let app_state = MockAppStateBuilder::new()
        .with_supplier_service(Arc::new(MockSupplierService::new_success()));

    let app = build_test_router(app_state);

    // Make get all request with pagination
    let (status, response) = make_request(app, "GET", "/api/supplier?page=1&page_size=10", None)
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::OK);
    let data = response["data"].as_array().unwrap();
    assert!(data.len() <= 10); // Should respect page_size
}

#[tokio::test]
async fn test_get_all_suppliers_service_error() {
    // Setup - use mock supplier service that returns error
    let app_state = MockAppStateBuilder::new()
        .with_supplier_service(Arc::new(MockSupplierService::new_failure()));

    let app = build_test_router(app_state);

    // Make get all request
    let (status, response) = make_request(app, "GET", "/api/supplier", None)
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Internal error"),
        "Expected internal error message, got: {}",
        error_msg
    );
}
