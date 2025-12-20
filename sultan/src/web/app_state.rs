use axum::extract::FromRef;
use std::sync::Arc;
use sultan_core::application::AuthService;
use sultan_core::crypto::{Argon2PasswordHasher, DefaultJwtManager};
use sultan_core::domain::context::BranchContext;
use sultan_core::storage::sqlite::{SqliteTokenRepository, SqliteUserRepository};

use crate::config::AppConfig;

pub type ConcreteAuthService = AuthService<
    BranchContext,
    SqliteUserRepository<BranchContext>,
    SqliteTokenRepository<BranchContext>,
    Argon2PasswordHasher,
    DefaultJwtManager,
>;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub auth_service: Arc<ConcreteAuthService>,
}

impl FromRef<AppState> for Arc<ConcreteAuthService> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.auth_service.clone()
    }
}
