use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, ExprTrait, Order,
    QueryFilter, QueryOrder, QuerySelect, Set, sea_query::Expr,
};

use super::entity::{CashierSessionActiveModel, CashierSessionColumn, CashierSessionEntity};
use crate::{
    domain::{
        DomainResult,
        error::Error,
        model::{
            cashier_session::{
                CashierSession, CashierSessionClose, CashierSessionCreate, CashierSessionCursor,
                CashierSessionPage, CashierSessionQuery, CashierSessionSortField,
            },
            product::SortDirection,
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
        query: &CashierSessionQuery,
    ) -> DomainResult<CashierSessionPage> {
        let mut select =
            CashierSessionEntity::find().filter(CashierSessionColumn::IsDeleted.eq(false));

        // ── Filters ──────────────────────────────────────────────────────
        if let Some(branch_id) = query.filter.branch_id {
            select = select.filter(CashierSessionColumn::BranchId.eq(branch_id));
        }

        if let Some(user_id) = query.filter.user_id {
            select = select.filter(CashierSessionColumn::UserId.eq(user_id));
        }

        if let Some(status) = &query.filter.status {
            select = select.filter(CashierSessionColumn::Status.eq(status.as_str()));
        }

        // ── Map sort field to column ──────────────────────────────────────
        let sort_col = match query.sort_field {
            CashierSessionSortField::OpenedAt => CashierSessionColumn::OpenedAt,
        };

        let order = match query.sort_direction {
            SortDirection::Asc => Order::Asc,
            SortDirection::Desc => Order::Desc,
        };

        // ── Cursor condition ──────────────────────────────────────────────
        if let Some(cursor) = &query.cursor {
            let cond = match query.sort_direction {
                SortDirection::Asc => Condition::any()
                    .add(Expr::col(sort_col).gt(cursor.field_value.clone()))
                    .add(
                        Condition::all()
                            .add(Expr::col(sort_col).eq(cursor.field_value.clone()))
                            .add(Expr::col(CashierSessionColumn::Id).gt(cursor.id)),
                    ),
                SortDirection::Desc => Condition::any()
                    .add(Expr::col(sort_col).lt(cursor.field_value.clone()))
                    .add(
                        Condition::all()
                            .add(Expr::col(sort_col).eq(cursor.field_value.clone()))
                            .add(Expr::col(CashierSessionColumn::Id).lt(cursor.id)),
                    ),
            };
            select = select.filter(cond);
        }

        // ── Ordering: (sort_field, id) ────────────────────────────────────
        select = select
            .order_by(sort_col, order.clone())
            .order_by(CashierSessionColumn::Id, order);

        // Fetch limit + 1 to detect whether there is a next page
        let fetch_limit = query.limit + 1;
        let rows = select.limit(fetch_limit).all(&ctx.db).await?;

        let has_next = rows.len() as u64 > query.limit;
        let models: Vec<_> = rows.into_iter().take(query.limit as usize).collect();

        // ── Build next_cursor from the last item ──────────────────────────
        let next_cursor = if has_next {
            models.last().map(|last| {
                let field_value = last.opened_at.clone();
                CashierSessionCursor {
                    field_value,
                    id: last.id,
                }
            })
        } else {
            None
        };

        let items = models.into_iter().map(|s| s.to_domain()).collect();

        Ok(CashierSessionPage { items, next_cursor })
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
