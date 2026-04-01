use async_trait::async_trait;
use sea_orm::ConnectionTrait;

use crate::domain::{
    DomainResult,
    model::cashier_session::{
        CashierSession, CashierSessionClose, CashierSessionCreate, CashierSessionPage,
        CashierSessionQuery,
    },
};

#[async_trait]
pub trait CashierSessionRepository: Send + Sync {
    async fn create(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        data: &CashierSessionCreate,
    ) -> DomainResult<()>;

    async fn get_by_id(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<CashierSession>>;

    /// Returns the open session for the given branch and user, if any.
    async fn get_open_by_user(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        branch_id: i64,
        user_id: i64,
    ) -> DomainResult<Option<CashierSession>>;

    /// Closes an open session. Returns `NotFound` if the session doesn't exist
    /// or is already closed.
    async fn close(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        data: &CashierSessionClose,
    ) -> DomainResult<()>;

    async fn get_all(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        query: &CashierSessionQuery,
    ) -> DomainResult<CashierSessionPage>;

    async fn delete(&self, ctx: &super::RepoCtx<impl ConnectionTrait>, id: i64)
    -> DomainResult<()>;
}
