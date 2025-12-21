use axum::extract::FromRef;
use std::sync::Arc;
use sultan_core::application::AuthServiceTrait;
use sultan_core::crypto::JwtManager;
use sultan_core::domain::context::BranchContext;

use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub auth_service: Arc<dyn AuthServiceTrait<BranchContext>>,
    pub jwt_manager: Arc<dyn JwtManager>,
}

impl FromRef<AppState> for Arc<dyn AuthServiceTrait<BranchContext>> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.auth_service.clone()
    }
}
