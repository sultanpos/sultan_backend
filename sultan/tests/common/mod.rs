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

use mock_auth_service::MockAuthService;
use mock_category_service::MockCategoryService;

/// Builder for creating test AppState with optional service overrides
pub struct MockAppStateBuilder {
    auth_service: Option<Arc<dyn AuthServiceTrait<BranchContext>>>,
    category_service: Option<Arc<dyn CategoryServiceTrait<BranchContext>>>,
}

impl MockAppStateBuilder {
    /// Create a new builder with default mock services
    pub fn new() -> Self {
        Self {
            auth_service: None,
            category_service: None,
        }
    }

    /// Override the auth service
    pub fn with_auth_service(mut self, service: Arc<dyn AuthServiceTrait<BranchContext>>) -> Self {
        self.auth_service = Some(service);
        self
    }

    /// Override the category service
    #[allow(dead_code)]
    pub fn with_category_service(
        mut self,
        service: Arc<dyn CategoryServiceTrait<BranchContext>>,
    ) -> Self {
        self.category_service = Some(service);
        self
    }

    /// Build the AppState with provided or default services
    pub fn build(self) -> AppState {
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
            auth_service: self
                .auth_service
                .unwrap_or_else(|| Arc::new(MockAuthService::new_success())),
            jwt_manager: Arc::new(jwt_manager),
            category_service: self
                .category_service
                .unwrap_or_else(|| Arc::new(MockCategoryService::new_success())),
        }
    }
}

impl Default for MockAppStateBuilder {
    fn default() -> Self {
        Self::new()
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
