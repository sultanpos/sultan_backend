use axum::extract::FromRef;
use std::sync::Arc;
use sultan_core::application::{AuthServiceTrait, CategoryServiceTrait, CustomerServiceTrait};
use sultan_core::crypto::JwtManager;

use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub auth_service: Arc<dyn AuthServiceTrait>,
    pub jwt_manager: Arc<dyn JwtManager>,
    pub category_service: Arc<dyn CategoryServiceTrait>,
    pub customer_service: Arc<dyn CustomerServiceTrait>,
}

impl FromRef<AppState> for Arc<dyn AuthServiceTrait> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.auth_service.clone()
    }
}

impl FromRef<AppState> for Arc<dyn CategoryServiceTrait> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.category_service.clone()
    }
}

impl FromRef<AppState> for Arc<dyn CustomerServiceTrait> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.customer_service.clone()
    }
}
