mod common;

use axum::Router;
use axum::http::StatusCode;
use serde_json::json;
use std::sync::Arc;

use common::{
    create_mock_app_state, make_request, mock_auth_service::MockAuthService,
    mock_category_service::MockCategoryService,
};
use sultan::web::auth_router::auth_router;

#[tokio::test]
async fn test_login_success() {
    // Setup - use mock auth service
    let mock_auth = Arc::new(MockAuthService::new_success());
    let mock_category = Arc::new(MockCategoryService::new_success());
    let app_state = create_mock_app_state(mock_auth, mock_category);

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
    let mock_auth = Arc::new(MockAuthService::new_success());
    let app_state = create_mock_app_state(mock_auth, Arc::new(MockCategoryService::new_success()));

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
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Username cannot be empty"),
        "Expected error message to contain 'validation', got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_login_validation_password_error() {
    // Setup - use mock auth service
    let mock_auth = Arc::new(MockAuthService::new_success());
    let app_state = create_mock_app_state(mock_auth, Arc::new(MockCategoryService::new_success()));

    // Build router
    let app = Router::new()
        .nest("/api/auth", auth_router())
        .with_state(app_state);

    // Make login request
    let body = json!({
        "username": "username",
        "password": ""
    });

    let (status, response) = make_request(app, "POST", "/api/auth", Some(body))
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Password cannot be empty"),
        "Expected error message to contain 'validation', got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    // Setup - use mock auth service
    let mock_auth = Arc::new(MockAuthService::new_success());
    let app_state = create_mock_app_state(mock_auth, Arc::new(MockCategoryService::new_success()));

    // Build router
    let app = Router::new()
        .nest("/api/auth", auth_router())
        .with_state(app_state);

    // Make login request
    let body = json!({
        "username": "username",
        "password": "password"
    });

    let (status, response) = make_request(app, "POST", "/api/auth", Some(body))
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Invalid credentials"),
        "Expected error message to contain 'validation', got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_refresh_token_validation_error() {
    // Setup - use mock auth service
    let mock_auth = Arc::new(MockAuthService::new_success());
    let app_state = create_mock_app_state(mock_auth, Arc::new(MockCategoryService::new_success()));

    // Build router
    let app = Router::new()
        .nest("/api/auth", auth_router())
        .with_state(app_state);

    // Make refresh request with empty token
    let body = json!({
        "refresh_token": ""
    });

    let (status, response) = make_request(app, "POST", "/api/auth/refresh", Some(body))
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Refresh token cannot be empty"),
        "Expected error message to contain 'Refresh token cannot be empty', got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_refresh_token_service_error() {
    // Setup - use mock auth service that returns error
    let mock_auth = Arc::new(MockAuthService::new_failure());
    let app_state = create_mock_app_state(mock_auth, Arc::new(MockCategoryService::new_success()));

    // Build router
    let app = Router::new()
        .nest("/api/auth", auth_router())
        .with_state(app_state);

    // Make refresh request with valid token format but service returns error
    let body = json!({
        "refresh_token": "invalid_refresh_token"
    });

    let (status, response) = make_request(app, "POST", "/api/auth/refresh", Some(body))
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Invalid refresh token"),
        "Expected error message to contain 'Invalid refresh token', got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_refresh_token_success() {
    // Setup - use mock auth service
    let mock_auth = Arc::new(MockAuthService::new_success());
    let app_state = create_mock_app_state(mock_auth, Arc::new(MockCategoryService::new_success()));

    // Build router
    let app = Router::new()
        .nest("/api/auth", auth_router())
        .with_state(app_state);

    // Make refresh request with valid token
    let body = json!({
        "refresh_token": "valid_refresh_token_xyz"
    });

    let (status, response) = make_request(app, "POST", "/api/auth/refresh", Some(body))
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
async fn test_logout_validation_error() {
    // Setup - use mock auth service
    let mock_auth = Arc::new(MockAuthService::new_success());
    let app_state = create_mock_app_state(mock_auth, Arc::new(MockCategoryService::new_success()));

    // Build router
    let app = Router::new()
        .nest("/api/auth", auth_router())
        .with_state(app_state);

    // Make logout request with empty token
    let body = json!({
        "refresh_token": ""
    });

    let (status, response) = make_request(app, "DELETE", "/api/auth", Some(body))
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Refresh token cannot be empty"),
        "Expected error message to contain 'Refresh token cannot be empty', got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_logout_service_error() {
    // Setup - use mock auth service that returns error
    let mock_auth = Arc::new(MockAuthService::new_failure());
    let app_state = create_mock_app_state(mock_auth, Arc::new(MockCategoryService::new_success()));

    // Build router
    let app = Router::new()
        .nest("/api/auth", auth_router())
        .with_state(app_state);

    // Make logout request with valid token format but service returns error
    let body = json!({
        "refresh_token": "invalid_refresh_token"
    });

    let (status, response) = make_request(app, "DELETE", "/api/auth", Some(body))
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let error_msg = response["error"].as_str().unwrap();
    assert!(
        error_msg.contains("Invalid refresh token"),
        "Expected error message to contain 'Invalid refresh token', got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_logout_success() {
    // Setup - use mock auth service
    let mock_auth = Arc::new(MockAuthService::new_success());
    let app_state = create_mock_app_state(mock_auth, Arc::new(MockCategoryService::new_success()));

    // Build router
    let app = Router::new()
        .nest("/api/auth", auth_router())
        .with_state(app_state);

    // Make logout request with valid token
    let body = json!({
        "refresh_token": "valid_refresh_token_xyz"
    });

    let (status, response) = make_request(app, "DELETE", "/api/auth", Some(body))
        .await
        .expect("Request failed");

    // Assert
    assert_eq!(status, StatusCode::NO_CONTENT);
    // NO_CONTENT means empty body
    assert!(response.is_null() || response.as_object().is_none_or(|o| o.is_empty()));
}
