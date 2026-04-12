use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, Set,
    sea_query::Expr,
};

use super::entity::{PaymentChannelActiveModel, PaymentChannelColumn, PaymentChannelEntity};

fn now_str() -> String {
    chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.fZ")
        .to_string()
}
use crate::{
    domain::{
        DomainResult, Error,
        model::payment_channel::{
            PaymentChannel, PaymentChannelCreate, PaymentChannelFilter,
            PaymentChannelPriorityUpdate, PaymentChannelUpdate,
        },
    },
    storage::{PaymentChannelRepository, RepoCtx},
};

#[derive(Clone, Default)]
pub struct SqlitePaymentChannelRepository {}

impl SqlitePaymentChannelRepository {
    pub fn new() -> Self {
        SqlitePaymentChannelRepository {}
    }
}

#[async_trait]
impl PaymentChannelRepository for SqlitePaymentChannelRepository {
    async fn create(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        data: &PaymentChannelCreate,
    ) -> DomainResult<()> {
        let now = now_str();
        let metadata_json = super::serialize_metadata(&data.metadata);

        let model = PaymentChannelActiveModel {
            id: Set(id),
            created_at: Set(now.clone()),
            updated_at: Set(now),
            deleted_at: Set(None),
            is_deleted: Set(false),
            branch_id: Set(data.branch_id),
            name: Set(data.name.clone()),
            priority: Set(data.priority),
            metadata: Set(metadata_json),
        };

        model.insert(&ctx.db).await?;
        Ok(())
    }

    async fn get_by_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<PaymentChannel>> {
        let model = PaymentChannelEntity::find_by_id(id)
            .filter(PaymentChannelColumn::IsDeleted.eq(false))
            .one(&ctx.db)
            .await?;
        Ok(model.map(|m| m.to_domain()))
    }

    async fn get_all(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        filter: &PaymentChannelFilter,
    ) -> DomainResult<Vec<PaymentChannel>> {
        let mut q = PaymentChannelEntity::find().filter(PaymentChannelColumn::IsDeleted.eq(false));

        if let Some(branch_id) = filter.branch_id {
            q = q.filter(PaymentChannelColumn::BranchId.eq(branch_id));
        }
        if let Some(name) = &filter.name {
            q = q.filter(PaymentChannelColumn::Name.contains(name));
        }

        let models = q
            .order_by_asc(PaymentChannelColumn::Priority)
            .order_by_asc(PaymentChannelColumn::Id)
            .all(&ctx.db)
            .await?;

        Ok(models.into_iter().map(|m| m.to_domain()).collect())
    }

    async fn update(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        data: &PaymentChannelUpdate,
    ) -> DomainResult<()> {
        let mut q = PaymentChannelEntity::update_many()
            .filter(PaymentChannelColumn::Id.eq(id))
            .filter(PaymentChannelColumn::IsDeleted.eq(false));

        if let Some(name) = &data.name {
            q = q.col_expr(PaymentChannelColumn::Name, Expr::value(name.clone()));
        }
        if let Some(priority) = data.priority {
            q = q.col_expr(PaymentChannelColumn::Priority, Expr::value(priority));
        }
        if data.metadata.should_update() {
            let metadata_json = super::serialize_metadata_update(&data.metadata);
            q = q.col_expr(PaymentChannelColumn::Metadata, Expr::value(metadata_json));
        }

        q = q.col_expr(PaymentChannelColumn::UpdatedAt, Expr::value(now_str()));

        let result = q.exec(&ctx.db).await?;
        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!(
                "Payment channel with id {} not found",
                id
            )));
        }
        Ok(())
    }

    async fn delete(&self, ctx: &RepoCtx<impl ConnectionTrait>, id: i64) -> DomainResult<()> {
        let now = now_str();
        let result = PaymentChannelEntity::update_many()
            .filter(PaymentChannelColumn::Id.eq(id))
            .filter(PaymentChannelColumn::IsDeleted.eq(false))
            .col_expr(PaymentChannelColumn::IsDeleted, Expr::value(true))
            .col_expr(
                PaymentChannelColumn::DeletedAt,
                Expr::value(Some(now.clone())),
            )
            .col_expr(PaymentChannelColumn::UpdatedAt, Expr::value(now))
            .exec(&ctx.db)
            .await?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!(
                "Payment channel with id {} not found",
                id
            )));
        }
        Ok(())
    }

    async fn update_priorities(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        updates: &[PaymentChannelPriorityUpdate],
    ) -> DomainResult<()> {
        if updates.is_empty() {
            return Ok(());
        }

        let now = now_str();
        for entry in updates {
            PaymentChannelEntity::update_many()
                .filter(PaymentChannelColumn::Id.eq(entry.id))
                .filter(PaymentChannelColumn::IsDeleted.eq(false))
                .col_expr(PaymentChannelColumn::Priority, Expr::value(entry.priority))
                .col_expr(PaymentChannelColumn::UpdatedAt, Expr::value(now.clone()))
                .exec(&ctx.db)
                .await?;
        }
        Ok(())
    }
}
