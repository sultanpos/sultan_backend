use async_trait::async_trait;
use sea_orm::ConnectionTrait;

use crate::domain::{
    DomainResult,
    model::machine::{Machine, MachineCreate, MachineFilter, MachineUpdate},
};

#[async_trait]
pub trait MachineRepository: Send + Sync {
    async fn create(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        machine: &MachineCreate,
    ) -> DomainResult<()>;

    async fn update(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        machine: &MachineUpdate,
    ) -> DomainResult<()>;

    async fn delete(&self, ctx: &super::RepoCtx<impl ConnectionTrait>, id: i64)
    -> DomainResult<()>;

    async fn get_by_id(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<Machine>>;

    async fn get_all(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        filter: &MachineFilter,
    ) -> DomainResult<Vec<Machine>>;
}
