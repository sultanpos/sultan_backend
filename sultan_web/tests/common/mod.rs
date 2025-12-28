pub mod mock_auth_service;
pub mod mock_category_service;
pub mod mock_customer_service;
pub mod mock_supplier_service;

pub use mock_auth_service::MockAuthService;
pub use mock_category_service::MockCategoryService;
pub use mock_customer_service::MockCustomerService;
pub use mock_supplier_service::MockSupplierService;

use anyhow::Result;
use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use sultan_core::application::{
    AuthServiceTrait, CategoryServiceTrait, CustomerServiceTrait, SupplierServiceTrait,
};
use sultan_core::crypto::{DefaultJwtManager, JwtConfig};
use sultan_web::AppState;
use tower::ServiceExt;

/// Builder for creating test AppState with optional service overrides
pub struct MockAppStateBuilder {
    auth_service: Option<Arc<dyn AuthServiceTrait>>,
    category_service: Option<Arc<dyn CategoryServiceTrait>>,
    customer_service: Option<Arc<dyn CustomerServiceTrait>>,
    supplier_service: Option<Arc<dyn SupplierServiceTrait>>,
    extensions: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl MockAppStateBuilder {
    /// Create a new builder with default mock services
    pub fn new() -> Self {
        Self {
            auth_service: None,
            category_service: None,
            customer_service: None,
            supplier_service: None,
            extensions: HashMap::new(),
        }
    }

    /// Override the auth service
    #[allow(dead_code)]
    pub fn with_auth_service(mut self, service: Arc<dyn AuthServiceTrait>) -> Self {
        self.auth_service = Some(service);
        self
    }

    /// Override the category service
    #[allow(dead_code)]
    pub fn with_category_service(mut self, service: Arc<dyn CategoryServiceTrait>) -> Self {
        self.category_service = Some(service);
        self
    }

    /// Override the customer service
    #[allow(dead_code)]
    pub fn with_customer_service(mut self, service: Arc<dyn CustomerServiceTrait>) -> Self {
        self.customer_service = Some(service);
        self
    }

    /// Override the supplier service
    #[allow(dead_code)]
    pub fn with_supplier_service(mut self, service: Arc<dyn SupplierServiceTrait>) -> Self {
        self.supplier_service = Some(service);
        self
    }

    /// Add an extension to the AppState
    #[allow(dead_code)]
    pub fn add_extension<T: Send + Sync + 'static>(mut self, value: Arc<T>) -> Self {
        self.extensions.insert(TypeId::of::<T>(), value);
        self
    }

    /// Build the AppState with provided or default services
    pub fn build(self) -> AppState {
        let jwt_manager = DefaultJwtManager::new(JwtConfig::new(
            "test_secret_key_which_is_long_enough".to_string(),
            60,
        ));

        AppState {
            auth_service: self
                .auth_service
                .unwrap_or_else(|| Arc::new(MockAuthService::new_success())),
            jwt_manager: Arc::new(jwt_manager),
            category_service: self
                .category_service
                .unwrap_or_else(|| Arc::new(MockCategoryService::new_success())),
            customer_service: self
                .customer_service
                .unwrap_or_else(|| Arc::new(MockCustomerService::new_success())),
            supplier_service: self
                .supplier_service
                .unwrap_or_else(|| Arc::new(MockSupplierService::new_success())),
            extensions: Arc::new(self.extensions),
        }
    }
}

impl Default for MockAppStateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Make an HTTP request to the test app
#[allow(dead_code)]
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
