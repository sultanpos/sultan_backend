mod common;

use axum::Router;
use axum::http::StatusCode;
use axum::middleware::from_fn;
use serde_json::json;
use std::sync::Arc;

use common::{MockAppStateBuilder, make_request, mock_machine_service::MockMachineService};
use sultan_web::{handler::machine_router::machine_router, middleware::context_middleware};

// ============================================================================
// Helper Functions
// ============================================================================

fn build_test_router(app_state: MockAppStateBuilder) -> Router {
    Router::new()
        .nest("/api/machine", machine_router())
        .layer(from_fn(context_middleware))
        .with_state(app_state.build())
}

// ============================================================================
// POST /api/machine - Create Machine Tests
// ============================================================================

#[tokio::test]
async fn test_create_machine_success() {
    let app_state = MockAppStateBuilder::new()
        .with_machine_service(Arc::new(MockMachineService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "branch_id": "1",
        "key": "POS-01",
        "name": "Counter 1",
        "description": "Main counter"
    });

    let (status, response) = make_request(app, "POST", "/api/machine", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::CREATED);
    assert!(response.get("id").is_some());
    assert_eq!(response["id"].as_str().unwrap(), "1");
}

#[tokio::test]
async fn test_create_machine_validation_error_empty_name() {
    let app_state = MockAppStateBuilder::new()
        .with_machine_service(Arc::new(MockMachineService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "branch_id": "1",
        "key": "POS-01",
        "name": ""
    });

    let (status, response) = make_request(app, "POST", "/api/machine", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        response["error"]
            .as_str()
            .unwrap()
            .contains("Name must be between")
    );
}

#[tokio::test]
async fn test_create_machine_validation_error_empty_key() {
    let app_state = MockAppStateBuilder::new()
        .with_machine_service(Arc::new(MockMachineService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({
        "branch_id": "1",
        "key": "",
        "name": "Counter 1"
    });

    let (status, response) = make_request(app, "POST", "/api/machine", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        response["error"]
            .as_str()
            .unwrap()
            .contains("Key must be between")
    );
}

#[tokio::test]
async fn test_create_machine_service_failure() {
    let app_state = MockAppStateBuilder::new()
        .with_machine_service(Arc::new(MockMachineService::new_failure()));

    let app = build_test_router(app_state);

    let body = json!({
        "branch_id": "1",
        "key": "POS-01",
        "name": "Counter 1"
    });

    let (status, _) = make_request(app, "POST", "/api/machine", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
}

// ============================================================================
// PUT /api/machine/{id} - Update Machine Tests
// ============================================================================

#[tokio::test]
async fn test_update_machine_success() {
    let app_state = MockAppStateBuilder::new()
        .with_machine_service(Arc::new(MockMachineService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({ "name": "Updated Counter" });

    let (status, _) = make_request(app, "PUT", "/api/machine/1", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_update_machine_not_found() {
    let app_state = MockAppStateBuilder::new()
        .with_machine_service(Arc::new(MockMachineService::new_success()));

    let app = build_test_router(app_state);

    let body = json!({ "name": "Updated Counter" });

    let (status, response) = make_request(app, "PUT", "/api/machine/999", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(response["error"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn test_update_machine_service_failure() {
    let app_state = MockAppStateBuilder::new()
        .with_machine_service(Arc::new(MockMachineService::new_failure()));

    let app = build_test_router(app_state);

    let body = json!({ "name": "Updated Counter" });

    let (status, _) = make_request(app, "PUT", "/api/machine/1", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
}

// ============================================================================
// DELETE /api/machine/{id} - Delete Machine Tests
// ============================================================================

#[tokio::test]
async fn test_delete_machine_success() {
    let app_state = MockAppStateBuilder::new()
        .with_machine_service(Arc::new(MockMachineService::new_success()));

    let app = build_test_router(app_state);

    let (status, _) = make_request(app, "DELETE", "/api/machine/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_machine_not_found() {
    let app_state = MockAppStateBuilder::new()
        .with_machine_service(Arc::new(MockMachineService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "DELETE", "/api/machine/999", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(response["error"].as_str().unwrap().contains("not found"));
}

// ============================================================================
// GET /api/machine/{id} - Get Machine By ID Tests
// ============================================================================

#[tokio::test]
async fn test_get_machine_by_id_success() {
    let app_state = MockAppStateBuilder::new()
        .with_machine_service(Arc::new(MockMachineService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/machine/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["id"].as_str().unwrap(), "1");
    assert_eq!(response["key"].as_str().unwrap(), "POS-01");
    assert_eq!(response["name"].as_str().unwrap(), "Counter 1");
    assert_eq!(response["branch_id"].as_str().unwrap(), "1");
}

#[tokio::test]
async fn test_get_machine_by_id_not_found() {
    let app_state = MockAppStateBuilder::new()
        .with_machine_service(Arc::new(MockMachineService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/machine/999", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(response["error"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn test_get_machine_by_id_service_failure() {
    let app_state = MockAppStateBuilder::new()
        .with_machine_service(Arc::new(MockMachineService::new_failure()));

    let app = build_test_router(app_state);

    let (status, _) = make_request(app, "GET", "/api/machine/1", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
}

// ============================================================================
// GET /api/machine - List Machines Tests
// ============================================================================

#[tokio::test]
async fn test_get_machines_success() {
    let app_state = MockAppStateBuilder::new()
        .with_machine_service(Arc::new(MockMachineService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/machine", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    let items = response["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["key"].as_str().unwrap(), "POS-01");
}

#[tokio::test]
async fn test_get_machines_empty() {
    let app_state = MockAppStateBuilder::new()
        .with_machine_service(Arc::new(MockMachineService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/machine?name=empty", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["items"].as_array().unwrap().len(), 0);
    assert!(response["next_cursor"].is_null());
}

#[tokio::test]
async fn test_get_machines_with_filters() {
    let app_state = MockAppStateBuilder::new()
        .with_machine_service(Arc::new(MockMachineService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(
        app,
        "GET",
        "/api/machine?branch_id=1&name=Counter&limit=10",
        None,
    )
    .await
    .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert!(response.get("items").is_some());
    assert!(response.get("next_cursor").is_some());
}

#[tokio::test]
async fn test_get_machines_invalid_sort_field() {
    let app_state = MockAppStateBuilder::new()
        .with_machine_service(Arc::new(MockMachineService::new_success()));

    let app = build_test_router(app_state);

    let (status, response) = make_request(app, "GET", "/api/machine?sort_field=invalid", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        response["error"]
            .as_str()
            .unwrap()
            .contains("Invalid sort_field")
    );
}

#[tokio::test]
async fn test_get_machines_service_failure() {
    let app_state = MockAppStateBuilder::new()
        .with_machine_service(Arc::new(MockMachineService::new_failure()));

    let app = build_test_router(app_state);

    let (status, _) = make_request(app, "GET", "/api/machine", None)
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
}
