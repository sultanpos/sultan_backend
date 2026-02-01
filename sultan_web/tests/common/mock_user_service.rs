use async_trait::async_trait;
use chrono::Utc;
use sultan_core::application::UserServiceTrait;
use sultan_core::domain::model::pagination::PaginationOptions;
use sultan_core::domain::model::permission::{Permission, PermissionCreate, action, resource};
use sultan_core::domain::model::user::{UserCreate, UserFilter, UserUpdate};
use sultan_core::domain::{Context, DomainResult, Error, User};

pub struct MockUserService {
    pub should_succeed: bool,
}

impl MockUserService {
    pub fn new_success() -> Self {
        Self {
            should_succeed: true,
        }
    }

    #[allow(dead_code)]
    pub fn new_failure() -> Self {
        Self {
            should_succeed: false,
        }
    }
}

#[async_trait]
impl UserServiceTrait for MockUserService {
    async fn create(
        &self,
        _ctx: &Context,
        _user: &UserCreate,
        _permissions: &[PermissionCreate],
    ) -> DomainResult<i64> {
        if self.should_succeed {
            Ok(1)
        } else {
            Err(Error::Database("Mock create user error".to_string()))
        }
    }

    async fn update(
        &self,
        _ctx: &Context,
        _id: i64,
        _user: &UserUpdate,
        _permissions: Option<Vec<PermissionCreate>>,
    ) -> DomainResult<()> {
        if self.should_succeed {
            Ok(())
        } else {
            Err(Error::Database("Mock update user error".to_string()))
        }
    }

    async fn get_by_id(&self, _ctx: &Context, user_id: i64) -> DomainResult<Option<User>> {
        if !self.should_succeed {
            return Err(Error::Database("Mock get user error".to_string()));
        }

        // Return a mock user for ID 1
        if user_id == 1 {
            Ok(Some(User {
                id: 1,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                deleted_at: None,
                is_deleted: false,
                username: "testuser".to_string(),
                password: "hashed_password".to_string(),
                name: "Test User".to_string(),
                email: Some("test@example.com".to_string()),
                photo: None,
                pin: None,
                address: None,
                phone: None,
                permissions: None,
            }))
        } else {
            Ok(None)
        }
    }

    async fn reset_password(
        &self,
        _ctx: &Context,
        _user_id: i64,
        _new_password: String,
    ) -> DomainResult<()> {
        if self.should_succeed {
            Ok(())
        } else {
            Err(Error::Database("Mock reset password error".to_string()))
        }
    }

    async fn delete(&self, _ctx: &Context, _user_id: i64) -> DomainResult<()> {
        if self.should_succeed {
            Ok(())
        } else {
            Err(Error::Database("Mock delete user error".to_string()))
        }
    }

    async fn get_user_permission(
        &self,
        _ctx: &Context,
        user_id: i64,
    ) -> DomainResult<Vec<Permission>> {
        if !self.should_succeed {
            return Err(Error::Database(
                "Mock get user permission error".to_string(),
            ));
        }

        // Return mock permissions for user ID 1
        if user_id == 1 {
            Ok(vec![Permission {
                user_id: 1,
                branch_id: None,
                resource: resource::USER,
                action: action::READ | action::CREATE, // Combined actions using bitwise OR
            }])
        } else {
            Ok(vec![])
        }
    }

    async fn get_all(
        &self,
        ctx: &Context,
        filter: &UserFilter,
        pagination: &PaginationOptions,
    ) -> DomainResult<Vec<User>> {
        Err(Error::Database("Mock get all users error".to_string()))
    }

    async fn change_my_password(
        &self,
        ctx: &Context,
        old_password: String,
        new_password: String,
    ) -> DomainResult<()> {
        Err(Error::Database("Mock change my password error".to_string()))
    }
}
