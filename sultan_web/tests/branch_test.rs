mod common;

use axum::Router;
use axum::http::StatusCode;
use axum::middleware::from_fn;
use serde_json::json;
use std::sync::Arc;

use common::{MockAppStateBuilder, make_request, mock_branch_service::MockBranchService};
use sultan_web::{handler::branch_router::branch_router, middleware::context_middleware};

// ============================================================================
// Helper Functions
// ============================================================================

fn build_test_router(app_state: MockAppStateBuilder) -> Router {
    Router::new()
        .nest("/api/branch", branch_router())
        .layer(from_fn(context_middleware))
        .with_state(app_state.build())
}

// ============================================================================
// POST /api/branch - Create Branch Tests
// ============================================================================

#[tokio::test]
async fn test_create_branch_success() {
    let app_state =
        MockAppStateBuilder::new().with_branch_service(Arc::new(MockBranchService::new_success()));
    let app = build_test_router(app_state);

    let body = json!({
        "name": "Main Branch",
        "code": "MAIN",
        "is_main": true
    });

    let (status, response) = make_request(app, "POST", "/api/branch", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::CREATED);
    assert!(response.get("id").is_some());
}

#[tokio::test]
async fn test_create_branch_with_all_fields() {
    let app_state =
        MockAppStateBuilder::new().with_branch_service(Arc::new(MockBranchService::new_success()));
    let app = build_test_router(app_state);

    let body = json!({
        "name": "Secondary Branch",
        "code": "SEC",
        "is_main": false,
        "address": "Jl. Test No. 1",
        "phone": "0812345678",
        "npwp": "12.345.678.9-000.000",
        "image": "https://example.com/image.png"
    });

    let (status, response) = make_request(app, "POST", "/api/branch", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::CREATED);
    assert!(response.get("id").is_some());
}

#[tokio::test]
async fn test_create_branch_validation_error_empty_name() {
    let app_state =
        MockAppStateBuilder::new().with_branch_service(Arc::new(MockBranchService::new_success()));
    let app = build_test_router(app_state);

    let body = json!({
        "name": "",
        "code": "MAIN"
    });

    let (status, response) = make_request(app, "POST", "/api/branch", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        response["error"]
            .as_str()
            .unwrap()
            .contains("Name must be between 1 and 100 characters")
    );
}

#[tokio::test]
async fn test_create_branch_validation_error_name_too_long() {
    let app_state =
        MockAppStateBuilder::new().with_branch_service(Arc::new(MockBranchService::new_success()));
    let app = build_test_router(app_state);

    let body = json!({
        "name": "a".repeat(101),
        "code": "MAIN"
    });

    let (status, response) = make_request(app, "POST", "/api/branch", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        response["error"]
            .as_str()
            .unwrap()
            .contains("Name must be between 1 and 100 characters")
    );
}

#[tokio::test]
async fn test_create_branch_validation_error_empty_code() {
    let app_state =
        MockAppStateBuilder::new().with_branch_service(Arc::new(MockBranchService::new_success()));
    let app = build_test_router(app_state);

    let body = json!({
        "name": "Main Branch",
        "code": ""
    });

    let (status, response) = make_request(app, "POST", "/api/branch", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        response["error"]
            .as_str()
            .unwrap()
            .contains("Code must be between 1 and 50 characters")
    );
}

#[tokio::test]
async fn test_create_branch_service_error() {
    let app_state =
        MockAppStateBuilder::new().with_branch_service(Arc::new(MockBranchService::new_failure()));
    let app = build_test_router(app_state);

    let body = json!({
        "name": "Main Branch",
        "code": "MAIN"
    });

    let (status, response) = make_request(app, "POST", "/api/branch", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response.get("error").is_some());
}

// ============================================================================
// PUT /api/branch/:id - Update Branch Tests
// ============================================================================

#[tokio::test]
async fn test_update_branch_success() {
    let app_state =
        MockAppStateBuilder::new().with_branch_service(Arc::new(MockBranchService::new_success()));
    let app = build_test_router(app_state);

    let body = json!({
        "name": "Updated Branch"
    });

    let (status, _) = make_request(app, "PUT", "/api/branch/1", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_update_branch_not_found() {
    let app_state =
        MockAppStateBuilder::new().with_branch_service(Arc::new(MockBranchService::new_success()));
    let app = build_test_router(app_state);

    let body = json!({
        "name": "Updated Branch"
    });

    let (status, response) = make_request(app, "PUT", "/api/branch/999", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(response.get("error").is_some());
}

#[tokio::test]
async fn test_update_branch_service_error() {
    let app_state =
        MockAppStateBuilder::new().with_branch_service(Arc::new(MockBranchService::new_failure()));
    let app = build_test_router(app_state);

    let body = json!({
        "name": "Updated Branch"
    });

    let (status, response) = make_request(app, "PUT", "/api/branch/1", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response.get("error").is_some());
}

// ============================================================================
// DELETE /api/branch/:id - Delete Branch Tests
// ============================================================================

#[tokio::test]
async fn test_delete_branch_success() {
    let app_state =
        MockAppStateBuilder::new().with_branch_service(Arc::new(MockBranchService::new_success()));
    let app = build_test_router(app_state);

    let (status, _) = make_request(app, "DELETE", "/api/branch/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_branch_not_found() {
    let app_state =
        MockAppStateBuilder::new().with_branch_service(Arc::new(MockBranchService::new_success()));
    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "DELETE", "/api/branch/999", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(response.get("error").is_some());
}

#[tokio::test]
async fn test_delete_branch_service_error() {
    let app_state =
        MockAppStateBuilder::new().with_branch_service(Arc::new(MockBranchService::new_failure()));
    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "DELETE", "/api/branch/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response.get("error").is_some());
}

// ============================================================================
// GET /api/branch/:id - Get By ID Tests
// ============================================================================

#[tokio::test]
async fn test_get_branch_by_id_success() {
    let app_state =
        MockAppStateBuilder::new().with_branch_service(Arc::new(MockBranchService::new_success()));
    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/branch/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["name"].as_str().unwrap(), "Sultan");
    assert_eq!(response["code"].as_str().unwrap(), "SULTAN");
    assert!(response["is_main"].as_bool().unwrap());
}

#[tokio::test]
async fn test_get_branch_by_id_not_found() {
    let app_state =
        MockAppStateBuilder::new().with_branch_service(Arc::new(MockBranchService::new_success()));
    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/branch/999", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(response.get("error").is_some());
}

#[tokio::test]
async fn test_get_branch_by_id_service_error() {
    let app_state =
        MockAppStateBuilder::new().with_branch_service(Arc::new(MockBranchService::new_failure()));
    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/branch/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response.get("error").is_some());
}

// ============================================================================
// GET /api/branch - Get All / List Tests
// ============================================================================

#[tokio::test]
async fn test_get_all_branches_success() {
    let app_state =
        MockAppStateBuilder::new().with_branch_service(Arc::new(MockBranchService::new_success()));
    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/branch", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    let items = response["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["name"].as_str().unwrap(), "Sultan");
    assert!(response["next_cursor"].is_null());
}

#[tokio::test]
async fn test_get_all_branches_response_structure() {
    let app_state =
        MockAppStateBuilder::new().with_branch_service(Arc::new(MockBranchService::new_success()));
    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/branch", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert!(response.get("items").is_some());
    assert!(response.get("next_cursor").is_some());
}

#[tokio::test]
async fn test_get_all_branches_with_name_filter() {
    let app_state =
        MockAppStateBuilder::new().with_branch_service(Arc::new(MockBranchService::new_success()));
    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/branch?name=Sultan", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert!(response["items"].is_array());
}

#[tokio::test]
async fn test_get_all_branches_sort_by_name_asc() {
    let app_state =
        MockAppStateBuilder::new().with_branch_service(Arc::new(MockBranchService::new_success()));
    let app = build_test_router(app_state);

    let (status, response) = make_request(
        app,
        "GET",
        "/api/branch?sort_field=name&sort_direction=asc",
        None,
    )
    .await
    .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert!(response["items"].is_array());
}

#[tokio::test]
async fn test_get_all_branches_sort_by_created_at_desc() {
    let app_state =
        MockAppStateBuilder::new().with_branch_service(Arc::new(MockBranchService::new_success()));
    let app = build_test_router(app_state);

    let (status, response) = make_request(
        app,
        "GET",
        "/api/branch?sort_field=created_at&sort_direction=desc",
        None,
    )
    .await
    .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert!(response["items"].is_array());
}

#[tokio::test]
async fn test_get_all_branches_with_limit() {
    let app_state =
        MockAppStateBuilder::new().with_branch_service(Arc::new(MockBranchService::new_success()));
    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/branch?limit=5", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert!(response["items"].as_array().unwrap().len() <= 5);
}

#[tokio::test]
async fn test_get_all_branches_invalid_sort_field() {
    let app_state =
        MockAppStateBuilder::new().with_branch_service(Arc::new(MockBranchService::new_success()));
    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/branch?sort_field=invalid", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["error"].as_str().unwrap().contains("sort_field"));
}

#[tokio::test]
async fn test_get_all_branches_invalid_sort_direction() {
    let app_state =
        MockAppStateBuilder::new().with_branch_service(Arc::new(MockBranchService::new_success()));
    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/branch?sort_direction=invalid", None)
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
async fn test_get_all_branches_invalid_cursor() {
    let app_state =
        MockAppStateBuilder::new().with_branch_service(Arc::new(MockBranchService::new_success()));
    let app = build_test_router(app_state);

    let (status, response) =
        make_request(app, "GET", "/api/branch?cursor=not_valid_base64!!!", None)
            .await
            .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["error"].as_str().unwrap().contains("cursor"));
}

#[tokio::test]
async fn test_get_all_branches_service_error() {
    let app_state =
        MockAppStateBuilder::new().with_branch_service(Arc::new(MockBranchService::new_failure()));
    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/branch", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response.get("error").is_some());
}
