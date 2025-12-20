mod common;

use axum::Router;
use axum::http::StatusCode;
use serde_json::json;
use std::sync::Arc;

use common::{create_mock_app_state, make_request, mock_auth_service::MockAuthService};
use sultan::web::auth_router::auth_router;

#[tokio::test]
async fn test_login_success() {
    // Setup - use mock auth service
    let mock_service = Arc::new(MockAuthService::new_success());
    let app_state = create_mock_app_state(mock_service);

    // Build router
    let app = Router::new()
        .nest("/api/auth", auth_router())
        .with_state(app_state);

    // Make login request
    let body = json!({
        "username": "testuser",
        "password": "testpassword123"
    });

    let (status, response) = make_request(app, "POST", "/api/auth", Some(body))
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::OK);
    assert!(response.get("access_token").is_some());
    assert!(response.get("refresh_token").is_some());
    assert_eq!(
        response["access_token"].as_str().unwrap(),
        "mock_access_token_12345"
    );
    assert_eq!(
        response["refresh_token"].as_str().unwrap(),
        "mock_refresh_token_67890"
    );
}

#[tokio::test]
async fn test_login_validation_error() {
    // Setup - use mock auth service
    let mock_service = Arc::new(MockAuthService::new_success());
    let app_state = create_mock_app_state(mock_service);

    // Build router
    let app = Router::new()
        .nest("/api/auth", auth_router())
        .with_state(app_state);

    // Make login request
    let body = json!({
        "username": "",
        "password": "testpassword123"
    });

    let (status, response) = make_request(app, "POST", "/api/auth", Some(body))
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
/*
#[tokio::test]
async fn test_login_invalid_credentials() {
    // Setup
    let pool = setup_test_db().await.expect("Failed to setup test db");
    let app_state = setup_test_app_state()
        .await
        .expect("Failed to setup app state");

    // Create a test user
    let username = "testuser";
    let password = "testpassword123";
    create_test_user(&pool, username, password)
        .await
        .expect("Failed to create test user");

    // Build router
    let app = Router::new()
        .nest("/api/auth", auth_router())
        .with_state(app_state);

    // Make login request with wrong password
    let body = json!({
        "username": username,
        "password": "wrongpassword"
    });

    let (status, _response) = make_request(app, "POST", "/api/auth", Some(body))
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_login_user_not_found() {
    // Setup
    let _pool = setup_test_db().await.expect("Failed to setup test db");
    let app_state = setup_test_app_state()
        .await
        .expect("Failed to setup app state");

    // Build router (no user created)
    let app = Router::new()
        .nest("/api/auth", auth_router())
        .with_state(app_state);

    // Make login request for non-existent user
    let body = json!({
        "username": "nonexistent",
        "password": "password123"
    });

    let (status, _response) = make_request(app, "POST", "/api/auth", Some(body))
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_login_missing_fields() {
    // Setup
    let app_state = setup_test_app_state()
        .await
        .expect("Failed to setup app state");

    // Build router
    let app = Router::new()
        .nest("/api/auth", auth_router())
        .with_state(app_state);

    // Make login request with missing password
    let body = json!({
        "username": "testuser"
    });

    let (status, _response) = make_request(app, "POST", "/api/auth", Some(body))
        .await
        .expect("Request failed");

    // Assert - Axum returns 422 for deserialization errors
    assert!(
        status == StatusCode::UNPROCESSABLE_ENTITY || status == StatusCode::BAD_REQUEST,
        "Expected 422 or 400, got {}",
        status
    );
}

#[tokio::test]
async fn test_login_empty_credentials() {
    // Setup
    let _pool = setup_test_db().await.expect("Failed to setup test db");
    let app_state = setup_test_app_state()
        .await
        .expect("Failed to setup app state");

    // Build router
    let app = Router::new()
        .nest("/api/auth", auth_router())
        .with_state(app_state);

    // Make login request with empty strings
    let body = json!({
        "username": "",
        "password": ""
    });

    let (status, _response) = make_request(app, "POST", "/api/auth", Some(body))
        .await
        .expect("Request failed");

    // Assert - should be unauthorized (user not found)
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_login_malformed_json() {
    // Setup
    let app_state = setup_test_app_state()
        .await
        .expect("Failed to setup app state");

    // Build router
    let app = Router::new()
        .nest("/api/auth", auth_router())
        .with_state(app_state);

    // Make request with invalid JSON
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    let request = Request::builder()
        .method("POST")
        .uri("/api/auth")
        .header("content-type", "application/json")
        .body(Body::from("{invalid json"))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    // Assert - should be bad request
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Expected 400 or 422, got {}",
        status
    );
}
*/
