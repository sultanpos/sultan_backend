use async_trait::async_trait;
use sea_orm::ConnectionTrait;

use crate::domain::{
    DomainResult,
    model::payment_channel::{
        PaymentChannel, PaymentChannelCreate, PaymentChannelFilter, PaymentChannelPriorityUpdate,
        PaymentChannelUpdate,
    },
};

#[async_trait]
pub trait PaymentChannelRepository: Send + Sync {
    /// Creates a new payment channel.
    async fn create(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        data: &PaymentChannelCreate,
    ) -> DomainResult<()>;

    /// Retrieves a payment channel by ID. Returns `None` if soft-deleted or not found.
    async fn get_by_id(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<PaymentChannel>>;

    /// Returns all non-deleted payment channels matching the filter, ordered by priority asc then id asc.
    async fn get_all(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        filter: &PaymentChannelFilter,
    ) -> DomainResult<Vec<PaymentChannel>>;

    /// Partially updates a payment channel.
    /// Returns `NotFound` if the channel is soft-deleted or does not exist.
    async fn update(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        data: &PaymentChannelUpdate,
    ) -> DomainResult<()>;

    /// Soft-deletes a payment channel.
    /// Returns `NotFound` if already deleted or not found.
    async fn delete(&self, ctx: &super::RepoCtx<impl ConnectionTrait>, id: i64)
    -> DomainResult<()>;

    /// Bulk-updates the `priority` field for the given channel IDs.
    /// Each entry in `updates` is `(id, new_priority)`.
    /// Silently skips IDs that do not exist or are soft-deleted.
    async fn update_priorities(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        updates: &[PaymentChannelPriorityUpdate],
    ) -> DomainResult<()>;
}
