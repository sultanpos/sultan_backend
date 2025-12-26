pub mod mock_auth_service;
pub mod mock_category_service;

use anyhow::Result;
use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use std::sync::Arc;
use sultan::config::AppConfig;
use sultan::web::AppState;
use sultan_core::application::{AuthServiceTrait, CategoryServiceTrait};
use sultan_core::crypto::{DefaultJwtManager, JwtConfig};
use sultan_core::domain::context::BranchContext;
use time::Duration;
use tower::ServiceExt;

/// Create a test app state with a mock auth service
pub fn create_mock_app_state(
    auth_service: Arc<dyn AuthServiceTrait<BranchContext>>,
    category_service: Arc<dyn CategoryServiceTrait<BranchContext>>,
) -> AppState {
    let config = AppConfig {
        database_url: "sqlite://test.db".to_string(),
        jwt_secret: "test_secret".to_string(),
        access_token_ttl: Duration::minutes(30),
        refresh_token_ttl: Duration::days(30),
        database_max_connections: 1,
        write_log_to_file: false,
    };

    let jwt_manager = DefaultJwtManager::new(JwtConfig::new(
        config.jwt_secret.clone(),
        config.access_token_ttl.whole_minutes(),
    ));

    AppState {
        config: Arc::new(config),
        auth_service,
        jwt_manager: Arc::new(jwt_manager),
        category_service,
    }
}

/// Make an HTTP request to the test app
pub async fn make_request(
    app: Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> Result<(StatusCode, Value)> {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");

    let body = if let Some(json_body) = body {
        Body::from(serde_json::to_vec(&json_body)?)
    } else {
        Body::empty()
    };

    let request = request.body(body)?;

    let response = app.oneshot(request).await?;
    let status = response.status();

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let json: Value = if body_bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&body_bytes)?
    };

    Ok((status, json))
}
