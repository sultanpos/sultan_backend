use async_trait::async_trait;

use crate::domain::Context;
use crate::domain::DomainResult;
use crate::domain::model::pagination::PaginationOptions;
use crate::domain::model::permission::Permission;
use crate::domain::model::user::{User, UserCreate, UserFilter, UserUpdate};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(&self, ctx: &Context, id: i64, user: &UserCreate) -> DomainResult<()>;
    async fn get_user_by_username(
        &self,
        ctx: &Context,
        username: &str,
    ) -> DomainResult<Option<User>>;
    async fn update_user(&self, ctx: &Context, id: i64, user: &UserUpdate) -> DomainResult<()>;
    async fn update_password(
        &self,
        ctx: &Context,
        id: i64,
        password_hash: &str,
    ) -> DomainResult<()>;
    async fn delete_user(&self, ctx: &Context, user_id: i64) -> DomainResult<()>;
    async fn get_all(
        &self,
        ctx: &Context,
        filter: UserFilter,
        pagination: PaginationOptions,
    ) -> DomainResult<Vec<User>>;
    async fn get_by_id(&self, ctx: &Context, user_id: i64) -> DomainResult<Option<User>>;
    async fn save_user_permission(
        &self,
        ctx: &Context,
        user_id: i64,
        branch_id: Option<i64>,
        permission: i32,
        action: i32,
    ) -> DomainResult<()>;
    async fn delete_user_permission(
        &self,
        ctx: &Context,
        user_id: i64,
        branch_id: Option<i64>,
        permission: i32,
    ) -> DomainResult<()>;
    async fn get_user_permission(
        &self,
        ctx: &Context,
        user_id: i64,
    ) -> DomainResult<Vec<Permission>>;
}
