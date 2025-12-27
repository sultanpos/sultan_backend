mod common;

use axum::Router;
use axum::http::StatusCode;
use axum::middleware::from_fn;
use serde_json::json;
use std::sync::Arc;

use common::{MockAppStateBuilder, make_request, mock_customer_service::MockCustomerService};
use sultan_web::handler::customer_router::customer_router;
use sultan_web::handler::middleware::context_middleware;

// ============================================================================
// Helper Functions
// ============================================================================

/// Helper function to build a test router with the context middleware
fn build_test_router(app_state: MockAppStateBuilder) -> Router {
    Router::new()
        .nest("/api/customer", customer_router())
        .layer(from_fn(context_middleware))
        .with_state(app_state.build())
}

// ============================================================================
// POST /api/customer - Create Customer Tests
// ============================================================================

#[tokio::test]
async fn test_create_customer_validation_error_empty_name() {
    let app = build_test_router(MockAppStateBuilder::new());

    let body = json!({
        "name": "",
        "level": 1
    });

    let (status, response) = make_request(app, "POST", "/api/customer", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response.get("error").is_some());
    assert!(response["error"].as_str().unwrap().contains("Name"));
}

#[tokio::test]
async fn test_create_customer_validation_error_name_too_long() {
    let app = build_test_router(MockAppStateBuilder::new());

    let body = json!({
        "name": "a".repeat(101),
        "level": 1
    });

    let (status, response) = make_request(app, "POST", "/api/customer", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response.get("error").is_some());
    assert!(response["error"].as_str().unwrap().contains("Name"));
}

#[tokio::test]
async fn test_create_customer_service_error() {
    let mock_service = Arc::new(MockCustomerService::new_failure());
    let app_state = MockAppStateBuilder::new().with_customer_service(mock_service);
    let app = build_test_router(app_state);

    let body = json!({
        "name": "John Doe",
        "level": 1
    });

    let (status, response) = make_request(app, "POST", "/api/customer", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response.get("error").is_some());
}

#[tokio::test]
async fn test_create_customer_success() {
    let app = build_test_router(MockAppStateBuilder::new());

    let body = json!({
        "name": "John Doe",
        "number": "CUST001",
        "phone": "1234567890",
        "email": "john@example.com",
        "address": "123 Main St",
        "level": 1
    });

    let (status, response) = make_request(app, "POST", "/api/customer", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::CREATED);
    assert!(response.get("id").is_some());
}

// ============================================================================
// PUT /api/customer/{id} - Update Customer Tests
// ============================================================================

#[tokio::test]
async fn test_update_customer_not_found() {
    let app = build_test_router(MockAppStateBuilder::new());

    let body = json!({
        "name": "Updated Name"
    });

    let (status, response) = make_request(app, "PUT", "/api/customer/999", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(response.get("error").is_some());
}

#[tokio::test]
async fn test_update_customer_service_error() {
    let mock_service = Arc::new(MockCustomerService::new_failure());
    let app_state = MockAppStateBuilder::new().with_customer_service(mock_service);
    let app = build_test_router(app_state);
    let body = json!({
        "name": "Updated Name"
    });

    let (status, response) = make_request(app, "PUT", "/api/customer/1", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response.get("error").is_some());
}

#[tokio::test]
async fn test_update_customer_success() {
    let app = build_test_router(MockAppStateBuilder::new());

    let body = json!({
        "name": "Updated Name",
        "phone": "0987654321"
    });

    let (status, _response) = make_request(app, "PUT", "/api/customer/1", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
}

// ============================================================================
// DELETE /api/customer/{id} - Delete Customer Tests
// ============================================================================

#[tokio::test]
async fn test_delete_customer_not_found() {
    let app = build_test_router(MockAppStateBuilder::new());

    let (status, response) = make_request(app, "DELETE", "/api/customer/999", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(response.get("error").is_some());
}

#[tokio::test]
async fn test_delete_customer_service_error() {
    let mock_service = Arc::new(MockCustomerService::new_failure());
    let app_state = MockAppStateBuilder::new().with_customer_service(mock_service);
    let app = build_test_router(app_state);
    let (status, response) = make_request(app, "DELETE", "/api/customer/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response.get("error").is_some());
}

#[tokio::test]
async fn test_delete_customer_success() {
    let app = build_test_router(MockAppStateBuilder::new());

    let (status, _response) = make_request(app, "DELETE", "/api/customer/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
}

// ============================================================================
// GET /api/customer/{id} - Get Customer by ID Tests
// ============================================================================

#[tokio::test]
async fn test_get_customer_by_id_not_found() {
    let app = build_test_router(MockAppStateBuilder::new());

    let (status, response) = make_request(app, "GET", "/api/customer/999", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(response.get("error").is_some());
}

#[tokio::test]
async fn test_get_customer_by_id_service_error() {
    let mock_service = Arc::new(MockCustomerService::new_failure());
    let app_state = MockAppStateBuilder::new().with_customer_service(mock_service);
    let app = build_test_router(app_state);
    let (status, response) = make_request(app, "GET", "/api/customer/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response.get("error").is_some());
}

#[tokio::test]
async fn test_get_customer_by_id_success() {
    let app = build_test_router(MockAppStateBuilder::new());

    let (status, response) = make_request(app, "GET", "/api/customer/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["id"].as_i64().unwrap(), 1);
    assert_eq!(response["name"].as_str().unwrap(), "John Doe");
    assert!(response.get("phone").is_some());
    assert!(response.get("email").is_some());
}

// ============================================================================
// GET /api/customer - Get All Customers Tests
// ============================================================================

#[tokio::test]
async fn test_get_all_customers_empty() {
    let mock_service = Arc::new(MockCustomerService::new_empty());
    let app_state = MockAppStateBuilder::new().with_customer_service(mock_service);
    let app = build_test_router(app_state);
    let (status, response) = make_request(app, "GET", "/api/customer", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert!(response["customers"].is_array());
    assert_eq!(response["customers"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_get_all_customers_service_error() {
    let mock_service = Arc::new(MockCustomerService::new_failure());
    let app_state = MockAppStateBuilder::new().with_customer_service(mock_service);
    let app = build_test_router(app_state);
    let (status, response) = make_request(app, "GET", "/api/customer", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response.get("error").is_some());
}

#[tokio::test]
async fn test_get_all_customers_success() {
    let app = build_test_router(MockAppStateBuilder::new());

    let (status, response) = make_request(app, "GET", "/api/customer", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert!(response["customers"].is_array());
    let customers = response["customers"].as_array().unwrap();
    assert_eq!(customers.len(), 2);
    assert_eq!(customers[0]["name"].as_str().unwrap(), "John Doe");
    assert_eq!(customers[1]["name"].as_str().unwrap(), "Jane Smith");
}

#[tokio::test]
async fn test_get_all_customers_with_pagination() {
    let app = build_test_router(MockAppStateBuilder::new());

    let (status, response) = make_request(app, "GET", "/api/customer?page=1&page_size=10", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert!(response["customers"].is_array());
    // Mock returns 2 customers regardless of pagination params
    assert_eq!(response["customers"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_get_all_customers_with_name_filter() {
    let app = build_test_router(MockAppStateBuilder::new());

    let (status, response) = make_request(app, "GET", "/api/customer?name=John", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert!(response["customers"].is_array());
    // Mock returns all customers regardless of filter
    assert_eq!(response["customers"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_get_all_customers_with_level_filter() {
    let app = build_test_router(MockAppStateBuilder::new());

    let (status, response) = make_request(app, "GET", "/api/customer?level=1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert!(response["customers"].is_array());
    assert_eq!(response["customers"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_get_all_customers_with_multiple_filters() {
    let app = build_test_router(MockAppStateBuilder::new());

    let (status, response) = make_request(
        app,
        "GET",
        "/api/customer?name=John&level=1&page=1&page_size=20&order_by=name&order_direction=asc",
        None,
    )
    .await
    .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert!(response["customers"].is_array());
    assert_eq!(response["customers"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_get_all_customers_with_email_filter() {
    let app = build_test_router(MockAppStateBuilder::new());

    let (status, response) = make_request(app, "GET", "/api/customer?email=example.com", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert!(response["customers"].is_array());
    assert_eq!(response["customers"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_get_all_customers_with_phone_filter() {
    let app = build_test_router(MockAppStateBuilder::new());

    let (status, response) = make_request(app, "GET", "/api/customer?phone=1234", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert!(response["customers"].is_array());
    assert_eq!(response["customers"].as_array().unwrap().len(), 2);
}
