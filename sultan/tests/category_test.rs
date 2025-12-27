mod common;

use axum::Router;
use axum::http::StatusCode;
use axum::middleware::from_fn;
use serde_json::json;
use std::sync::Arc;

use common::{MockAppStateBuilder, make_request, mock_category_service::MockCategoryService};
use sultan::web::category_router::category_router;
use sultan::web::middleware::context_middleware;

// ============================================================================
// POST /api/category - Create Category Tests
// ============================================================================

/// Helper function to build a test router with the context middleware
fn build_test_router(app_state: common::MockAppStateBuilder) -> Router {
    Router::new()
        .nest("/api/category", category_router())
        .layer(from_fn(context_middleware))
        .with_state(app_state.build())
}

#[tokio::test]
async fn test_create_category_success() {
    // Setup - use mock category service
    let app_state = MockAppStateBuilder::new()
        .with_category_service(Arc::new(MockCategoryService::new_success()));

    // Build router
    let app = build_test_router(app_state);

    // Make create request
    let body = json!({
        "name": "Electronics",
        "description": "Electronic devices and accessories",
        "parent_id": null
    });

    let (status, response) = make_request(app, "POST", "/api/category", Some(body))
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::CREATED);
    assert!(response.get("id").is_some());
    assert_eq!(response["id"].as_i64().unwrap(), 1);
}

#[tokio::test]
async fn test_create_category_validation_error_empty_name() {
    // Setup - use mock category service
    let app_state = MockAppStateBuilder::new()
        .with_category_service(Arc::new(MockCategoryService::new_success()));

    // Build router
    let app = build_test_router(app_state);

    // Make create request with empty name
    let body = json!({
        "name": "",
        "description": "Electronic devices",
        "parent_id": null
    });

    let (status, response) = make_request(app, "POST", "/api/category", Some(body))
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Name must be between 1 and 100 characters"),
        "Expected validation error for name, got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_create_category_validation_error_name_too_long() {
    // Setup - use mock category service
    let app_state = MockAppStateBuilder::new()
        .with_category_service(Arc::new(MockCategoryService::new_success()));

    // Build router
    let app = build_test_router(app_state);

    // Make create request with name that's too long (> 100 chars)
    let long_name = "a".repeat(101);
    let body = json!({
        "name": long_name,
        "description": "Electronic devices",
        "parent_id": null
    });

    let (status, response) = make_request(app, "POST", "/api/category", Some(body))
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Name must be between 1 and 100 characters"),
        "Expected validation error for name, got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_create_category_validation_error_description_too_long() {
    // Setup - use mock category service
    let app_state = MockAppStateBuilder::new()
        .with_category_service(Arc::new(MockCategoryService::new_success()));

    // Build router
    let app = build_test_router(app_state);

    // Make create request with description that's too long (> 500 chars)
    let long_description = "a".repeat(501);
    let body = json!({
        "name": "Electronics",
        "description": long_description,
        "parent_id": null
    });

    let (status, response) = make_request(app, "POST", "/api/category", Some(body))
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Description must not exceed 500 characters"),
        "Expected validation error for description, got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_create_category_service_error() {
    // Setup - use mock category service that returns error
    let app_state = MockAppStateBuilder::new()
        .with_category_service(Arc::new(MockCategoryService::new_failure()));

    // Build router
    let app = build_test_router(app_state);

    // Make create request
    let body = json!({
        "name": "Electronics",
        "description": "Electronic devices",
        "parent_id": null
    });

    let (status, response) = make_request(app, "POST", "/api/category", Some(body))
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Internal error"),
        "Expected internal error, got: {}",
        error_msg
    );
}

// ============================================================================
// PUT /api/category/{id} - Update Category Tests
// ============================================================================

#[tokio::test]
async fn test_update_category_success() {
    // Setup - use mock category service
    let app_state = MockAppStateBuilder::new()
        .with_category_service(Arc::new(MockCategoryService::new_success()));

    // Build router
    let app = build_test_router(app_state);

    // Make update request
    let body = json!({
        "name": "Updated Electronics",
        "description": "Updated description",
        "parent_id": null
    });

    let (status, response) = make_request(app, "PUT", "/api/category/1", Some(body))
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::NO_CONTENT);
    // NO_CONTENT means empty body
    assert!(response.is_null() || response.as_object().is_none_or(|o| o.is_empty()));
}

#[tokio::test]
async fn test_update_category_validation_error_empty_name() {
    // Setup - use mock category service
    let app_state = MockAppStateBuilder::new()
        .with_category_service(Arc::new(MockCategoryService::new_success()));

    // Build router
    let app = build_test_router(app_state);

    // Make update request with empty name
    let body = json!({
        "name": "",
        "description": "Updated description",
        "parent_id": null
    });

    let (status, response) = make_request(app, "PUT", "/api/category/1", Some(body))
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Name must be between 1 and 100 characters"),
        "Expected validation error for name, got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_update_category_validation_error_name_too_long() {
    // Setup - use mock category service
    let app_state = MockAppStateBuilder::new()
        .with_category_service(Arc::new(MockCategoryService::new_success()));

    // Build router
    let app = build_test_router(app_state);

    // Make update request with name that's too long (> 100 chars)
    let long_name = "a".repeat(101);
    let body = json!({
        "name": long_name,
        "description": "Updated description",
        "parent_id": null
    });

    let (status, response) = make_request(app, "PUT", "/api/category/1", Some(body))
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Name must be between 1 and 100 characters"),
        "Expected validation error for name, got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_update_category_service_error() {
    // Setup - use mock category service that returns error
    let app_state = MockAppStateBuilder::new()
        .with_category_service(Arc::new(MockCategoryService::new_failure()));

    // Build router
    let app = build_test_router(app_state);

    // Make update request
    let body = json!({
        "name": "Updated Electronics",
        "description": "Updated description",
        "parent_id": null
    });

    let (status, response) = make_request(app, "PUT", "/api/category/1", Some(body))
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Internal error"),
        "Expected internal error, got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_update_category_not_found() {
    // Setup - use mock category service
    let app_state = MockAppStateBuilder::new()
        .with_category_service(Arc::new(MockCategoryService::new_success()));

    // Build router
    let app = build_test_router(app_state);

    // Make update request for non-existent category
    let body = json!({
        "name": "Updated Electronics",
        "description": "Updated description",
        "parent_id": null
    });

    let (status, response) = make_request(app, "PUT", "/api/category/999", Some(body))
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::NOT_FOUND);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Category with id 999 not found"),
        "Expected not found error, got: {}",
        error_msg
    );
}

// ============================================================================
// DELETE /api/category/{id} - Delete Category Tests
// ============================================================================

#[tokio::test]
async fn test_delete_category_success() {
    // Setup - use mock category service
    let app_state = MockAppStateBuilder::new()
        .with_category_service(Arc::new(MockCategoryService::new_success()));

    // Build router
    let app = build_test_router(app_state);

    // Make delete request
    let (status, response) = make_request(app, "DELETE", "/api/category/1", None)
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::NO_CONTENT);
    // NO_CONTENT means empty body
    assert!(response.is_null() || response.as_object().is_none_or(|o| o.is_empty()));
}

#[tokio::test]
async fn test_delete_category_not_found() {
    // Setup - use mock category service
    let app_state = MockAppStateBuilder::new()
        .with_category_service(Arc::new(MockCategoryService::new_success()));

    // Build router
    let app = build_test_router(app_state);

    // Make delete request for non-existent category
    let (status, response) = make_request(app, "DELETE", "/api/category/999", None)
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::NOT_FOUND);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Category with id 999 not found"),
        "Expected not found error, got: {}",
        error_msg
    );
}

// ============================================================================
// GET /api/category/{id} - Get Category By ID Tests
// ============================================================================

#[tokio::test]
async fn test_get_category_by_id_success() {
    // Setup - use mock category service
    let app_state = MockAppStateBuilder::new()
        .with_category_service(Arc::new(MockCategoryService::new_success()));

    // Build router
    let app = build_test_router(app_state);

    // Make get request
    let (status, response) = make_request(app, "GET", "/api/category/1", None)
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["name"].as_str().unwrap(), "Electronics");
    assert_eq!(
        response["description"].as_str().unwrap(),
        "Electronic devices"
    );
}

#[tokio::test]
async fn test_get_category_by_id_not_found() {
    // Setup - use mock category service
    let app_state = MockAppStateBuilder::new()
        .with_category_service(Arc::new(MockCategoryService::new_success()));

    // Build router
    let app = build_test_router(app_state);

    // Make get request for non-existent category
    let (status, response) = make_request(app, "GET", "/api/category/999", None)
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::NOT_FOUND);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Category not found"),
        "Expected not found error, got: {}",
        error_msg
    );
}

// ============================================================================
// GET /api/category - Get All Categories Tests
// ============================================================================

#[tokio::test]
async fn test_get_all_categories_success() {
    // Setup - use mock category service
    let app_state = MockAppStateBuilder::new()
        .with_category_service(Arc::new(MockCategoryService::new_success()));

    // Build router
    let app = build_test_router(app_state);

    // Make get all request
    let (status, response) = make_request(app, "GET", "/api/category", None)
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::OK);
    let categories = response.as_array().unwrap();
    assert_eq!(categories.len(), 2);

    // Check first category
    assert_eq!(categories[0]["name"].as_str().unwrap(), "Electronics");
    assert_eq!(
        categories[0]["description"].as_str().unwrap(),
        "Electronic devices"
    );

    // Check second category
    assert_eq!(categories[1]["name"].as_str().unwrap(), "Books");
    assert_eq!(
        categories[1]["description"].as_str().unwrap(),
        "Books and magazines"
    );
}

#[tokio::test]
async fn test_get_all_categories_empty() {
    // Setup - create a custom mock that returns empty list
    struct EmptyMockCategoryService;

    #[async_trait::async_trait]
    impl sultan_core::application::CategoryServiceTrait for EmptyMockCategoryService {
        async fn create(
            &self,
            _ctx: &sultan_core::domain::context::Context,
            _category: &sultan_core::domain::model::category::CategoryCreate,
        ) -> sultan_core::domain::DomainResult<i64> {
            Ok(1)
        }

        async fn update(
            &self,
            _ctx: &sultan_core::domain::context::Context,
            _id: i64,
            _category: &sultan_core::domain::model::category::CategoryUpdate,
        ) -> sultan_core::domain::DomainResult<()> {
            Ok(())
        }

        async fn delete(
            &self,
            _ctx: &sultan_core::domain::context::Context,
            _id: i64,
        ) -> sultan_core::domain::DomainResult<()> {
            Ok(())
        }

        async fn get_all(
            &self,
            _ctx: &sultan_core::domain::context::Context,
        ) -> sultan_core::domain::DomainResult<Vec<sultan_core::domain::model::category::Category>>
        {
            Ok(vec![])
        }

        async fn get_by_id(
            &self,
            _ctx: &sultan_core::domain::context::Context,
            _id: i64,
        ) -> sultan_core::domain::DomainResult<Option<sultan_core::domain::model::category::Category>>
        {
            Ok(None)
        }
    }

    let app_state =
        MockAppStateBuilder::new().with_category_service(Arc::new(EmptyMockCategoryService));

    // Build router
    let app = build_test_router(app_state);

    // Make get all request
    let (status, response) = make_request(app, "GET", "/api/category", None)
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::OK);
    let categories = response.as_array().unwrap();
    assert_eq!(categories.len(), 0);
}
