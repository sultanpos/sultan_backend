use axum::extract::FromRef;
use std::sync::Arc;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
};
use sultan_core::application::{
    AuthServiceTrait, CategoryServiceTrait, CustomerServiceTrait, SupplierServiceTrait,
    UserServiceTrait,
};
use sultan_core::crypto::JwtManager;

#[derive(Clone)]
pub struct AppState {
    pub auth_service: Arc<dyn AuthServiceTrait>,
    pub jwt_manager: Arc<dyn JwtManager>,
    pub category_service: Arc<dyn CategoryServiceTrait>,
    pub customer_service: Arc<dyn CustomerServiceTrait>,
    pub supplier_service: Arc<dyn SupplierServiceTrait>,
    pub user_service: Arc<dyn UserServiceTrait>,
    pub extensions: Arc<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

impl AppState {
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.extensions
            .get(&TypeId::of::<T>())
            .and_then(|val| val.clone().downcast::<T>().ok())
    }
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

impl FromRef<AppState> for Arc<dyn SupplierServiceTrait> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.supplier_service.clone()
    }
}

impl FromRef<AppState> for Arc<dyn UserServiceTrait> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.user_service.clone()
    }
}
