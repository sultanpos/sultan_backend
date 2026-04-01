use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set, sea_query::Expr,
};

use super::entity::{CashierSessionActiveModel, CashierSessionColumn, CashierSessionEntity};
use crate::{
    domain::{
        DomainResult,
        error::Error,
        model::cashier_session::{
            CashierSession, CashierSessionClose, CashierSessionCreate, CashierSessionFilter,
        },
    },
    storage::{CashierSessionRepository, RepoCtx},
};

#[derive(Clone, Default)]
pub struct SqliteCashierSessionRepository {}

impl SqliteCashierSessionRepository {
    pub fn new() -> Self {
        SqliteCashierSessionRepository {}
    }
}

#[async_trait]
impl CashierSessionRepository for SqliteCashierSessionRepository {
    async fn create(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        data: &CashierSessionCreate,
    ) -> DomainResult<()> {
        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.fZ")
            .to_string();

        let model = CashierSessionActiveModel {
            id: Set(id),
            created_at: Set(now.clone()),
            updated_at: Set(now.clone()),
            deleted_at: Set(None),
            is_deleted: Set(false),
            branch_id: Set(data.branch_id),
            user_id: Set(data.user_id),
            opened_at: Set(now),
            closed_at: Set(None),
            status: Set("open".to_string()),
            opening_cash: Set(data.opening_cash),
            closing_cash: Set(None),
            notes: Set(data.notes.clone()),
            metadata: Set(None),
        };

        model.insert(&ctx.db).await?;
        Ok(())
    }

    async fn get_by_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<CashierSession>> {
        let session = CashierSessionEntity::find_by_id(id)
            .filter(CashierSessionColumn::IsDeleted.eq(false))
            .one(&ctx.db)
            .await?;
        Ok(session.map(|s| s.to_domain()))
    }

    async fn get_open_by_user(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        branch_id: i64,
        user_id: i64,
    ) -> DomainResult<Option<CashierSession>> {
        let session = CashierSessionEntity::find()
            .filter(CashierSessionColumn::BranchId.eq(branch_id))
            .filter(CashierSessionColumn::UserId.eq(user_id))
            .filter(CashierSessionColumn::Status.eq("open"))
            .filter(CashierSessionColumn::IsDeleted.eq(false))
            .one(&ctx.db)
            .await?;
        Ok(session.map(|s| s.to_domain()))
    }

    async fn close(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        data: &CashierSessionClose,
    ) -> DomainResult<()> {
        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.fZ")
            .to_string();

        let mut update_query = CashierSessionEntity::update_many()
            .filter(CashierSessionColumn::Id.eq(id))
            .filter(CashierSessionColumn::IsDeleted.eq(false))
            .filter(CashierSessionColumn::Status.eq("open"))
            .col_expr(CashierSessionColumn::Status, Expr::value("closed"))
            .col_expr(
                CashierSessionColumn::ClosedAt,
                Expr::value(Some(now.clone())),
            )
            .col_expr(
                CashierSessionColumn::ClosingCash,
                Expr::value(Some(data.closing_cash)),
            )
            .col_expr(CashierSessionColumn::UpdatedAt, Expr::value(now));

        if let Some(notes) = &data.notes {
            update_query =
                update_query.col_expr(CashierSessionColumn::Notes, Expr::value(notes.clone()));
        }

        let result = update_query.exec(&ctx.db).await?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!(
                "Open cashier session with id {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn get_all(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        filter: &CashierSessionFilter,
    ) -> DomainResult<Vec<CashierSession>> {
        let mut query =
            CashierSessionEntity::find().filter(CashierSessionColumn::IsDeleted.eq(false));

        if let Some(branch_id) = filter.branch_id {
            query = query.filter(CashierSessionColumn::BranchId.eq(branch_id));
        }

        if let Some(user_id) = filter.user_id {
            query = query.filter(CashierSessionColumn::UserId.eq(user_id));
        }

        if let Some(status) = &filter.status {
            query = query.filter(CashierSessionColumn::Status.eq(status.as_str()));
        }

        let sessions = query.all(&ctx.db).await?;
        Ok(sessions.into_iter().map(|s| s.to_domain()).collect())
    }

    async fn delete(&self, ctx: &RepoCtx<impl ConnectionTrait>, id: i64) -> DomainResult<()> {
        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.fZ")
            .to_string();

        let result = CashierSessionEntity::update_many()
            .filter(CashierSessionColumn::Id.eq(id))
            .filter(CashierSessionColumn::IsDeleted.eq(false))
            .col_expr(CashierSessionColumn::IsDeleted, Expr::value(true))
            .col_expr(
                CashierSessionColumn::DeletedAt,
                Expr::value(Some(now.clone())),
            )
            .col_expr(CashierSessionColumn::UpdatedAt, Expr::value(now))
            .exec(&ctx.db)
            .await?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!(
                "Cashier session with id {} not found",
                id
            )));
        }

        Ok(())
    }
}
