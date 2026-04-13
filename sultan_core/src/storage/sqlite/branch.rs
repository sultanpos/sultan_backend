use async_trait::async_trait;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, ExprTrait, Order,
    QueryFilter, QueryOrder, QuerySelect, Set, UpdateMany,
};

use crate::{
    domain::{
        DomainResult, Error,
        model::branch::{
            Branch, BranchCreate, BranchCursor, BranchPage, BranchQuery, BranchUpdate,
        },
    },
    storage::{
        RepoCtx,
        branch_repo::BranchRepository,
        sqlite::entity::{BranchActiveModel, BranchColumn, BranchEntity, BranchModel},
    },
};

/// SQLite implementation of BranchRepository using SeaORM.
///
/// This repository uses SeaORM's `ConnectionTrait` which allows it to work
/// with both direct database connections and transactions seamlessly.
///
/// # Example
///
/// ```rust,ignore
/// // Using with direct connection
/// let repo = SqliteBranchRepository::new();
/// let ctx = RepoCtx { ctx: Context::new(), db: &db_connection };
/// repo.create(&ctx, id, &branch).await?;
///
/// // Using within a transaction
/// let txn = db.begin().await?;
/// let ctx = RepoCtx { ctx: Context::new(), db: &txn };
/// repo.create(&ctx, id, &branch).await?;
/// txn.commit().await?;
/// ```
#[derive(Clone, Default)]
pub struct SqliteBranchRepository {}

impl SqliteBranchRepository {
    pub fn new() -> Self {
        SqliteBranchRepository {}
    }
}

#[async_trait]
impl BranchRepository for SqliteBranchRepository {
    async fn create(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        branch: &BranchCreate,
    ) -> DomainResult<()> {
        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.fZ")
            .to_string();

        let branch_model = BranchActiveModel {
            id: Set(id),
            created_at: Set(now.clone()),
            updated_at: Set(now),
            is_main: Set(branch.is_main),
            name: Set(branch.name.clone()),
            code: Set(branch.code.clone()),
            address: Set(branch.address.clone()),
            phone: Set(branch.phone.clone()),
            npwp: Set(branch.npwp.clone()),
            image: Set(branch.image.clone()),
            ..Default::default()
        };

        branch_model.insert(&ctx.db).await?;
        Ok(())
    }

    async fn get_by_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<Branch>> {
        let branch = BranchEntity::find_by_id(id)
            .filter(BranchColumn::IsDeleted.eq(false))
            .one(&ctx.db)
            .await?;

        Ok(branch.map(|b| b.to_domain()))
    }

    async fn update(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        branch: &BranchUpdate,
    ) -> DomainResult<()> {
        // Build update query with filters
        let mut update_query: UpdateMany<BranchEntity> = BranchEntity::update_many()
            .filter(BranchColumn::Id.eq(id))
            .filter(BranchColumn::IsDeleted.eq(false));

        // Update fields if provided
        if let Some(is_main) = branch.is_main {
            update_query = update_query.col_expr(BranchColumn::IsMain, Expr::value(is_main));
        }

        if let Some(name) = &branch.name {
            update_query = update_query.col_expr(BranchColumn::Name, Expr::value(name.clone()));
        }

        if let Some(code) = &branch.code {
            update_query = update_query.col_expr(BranchColumn::Code, Expr::value(code.clone()));
        }

        if branch.address.should_update() {
            update_query = update_query.col_expr(
                BranchColumn::Address,
                Expr::value(branch.address.to_bind_value()),
            );
        }

        if branch.phone.should_update() {
            update_query = update_query.col_expr(
                BranchColumn::Phone,
                Expr::value(branch.phone.to_bind_value()),
            );
        }

        if branch.npwp.should_update() {
            update_query =
                update_query.col_expr(BranchColumn::Npwp, Expr::value(branch.npwp.to_bind_value()));
        }

        if branch.image.should_update() {
            update_query = update_query.col_expr(
                BranchColumn::Image,
                Expr::value(branch.image.to_bind_value()),
            );
        }

        // Always update the updated_at timestamp
        update_query = update_query.col_expr(
            BranchColumn::UpdatedAt,
            Expr::value(
                chrono::Utc::now()
                    .format("%Y-%m-%dT%H:%M:%S%.fZ")
                    .to_string(),
            ),
        );

        // Execute the update
        let result = update_query.exec(&ctx.db).await?;

        // Check if any rows were affected
        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!("Branch with id {} not found", id)));
        }

        Ok(())
    }

    async fn delete(&self, ctx: &RepoCtx<impl ConnectionTrait>, id: i64) -> DomainResult<()> {
        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.fZ")
            .to_string();

        // Soft delete: mark as deleted with a single UPDATE query
        let result = BranchEntity::update_many()
            .filter(BranchColumn::Id.eq(id))
            .filter(BranchColumn::IsDeleted.eq(false))
            .col_expr(BranchColumn::IsDeleted, Expr::value(true))
            .col_expr(BranchColumn::DeletedAt, Expr::value(Some(now.clone())))
            .col_expr(BranchColumn::UpdatedAt, Expr::value(now))
            .exec(&ctx.db)
            .await?;

        // Check if any rows were affected
        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!("Branch with id {} not found", id)));
        }

        Ok(())
    }

    async fn get_all(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        query: &BranchQuery,
    ) -> DomainResult<BranchPage> {
        use crate::domain::model::branch::BranchSortField;
        use crate::domain::model::product::SortDirection;

        let mut select = BranchEntity::find().filter(BranchColumn::IsDeleted.eq(false));

        // ── Filters ──────────────────────────────────────────────────────────
        if let Some(name) = &query.filter.name {
            select = select.filter(BranchColumn::Name.contains(name));
        }

        // ── Map sort field to column ──────────────────────────────────────────
        let sort_col = match query.sort_field {
            BranchSortField::CreatedAt => BranchColumn::CreatedAt,
            BranchSortField::Name => BranchColumn::Name,
        };

        let order = match query.sort_direction {
            SortDirection::Asc => Order::Asc,
            SortDirection::Desc => Order::Desc,
        };

        // ── Cursor condition ──────────────────────────────────────────────────
        // WHERE (field > val) OR (field = val AND id > cursor_id)  [Asc]
        // WHERE (field < val) OR (field = val AND id < cursor_id)  [Desc]
        if let Some(cursor) = &query.cursor {
            let cond = match query.sort_direction {
                SortDirection::Asc => Condition::any()
                    .add(Expr::col(sort_col).gt(cursor.field_value.clone()))
                    .add(
                        Condition::all()
                            .add(Expr::col(sort_col).eq(cursor.field_value.clone()))
                            .add(Expr::col(BranchColumn::Id).gt(cursor.id)),
                    ),
                SortDirection::Desc => Condition::any()
                    .add(Expr::col(sort_col).lt(cursor.field_value.clone()))
                    .add(
                        Condition::all()
                            .add(Expr::col(sort_col).eq(cursor.field_value.clone()))
                            .add(Expr::col(BranchColumn::Id).lt(cursor.id)),
                    ),
            };
            select = select.filter(cond);
        }

        // ── Ordering: (sort_field, id) ────────────────────────────────────────
        select = select
            .order_by(sort_col, order.clone())
            .order_by(BranchColumn::Id, order);

        // Fetch limit + 1 to detect whether there is a next page
        let fetch_limit = query.limit + 1;
        let rows: Vec<BranchModel> = select.limit(fetch_limit).all(&ctx.db).await?;

        let has_next = rows.len() as u64 > query.limit;
        let models: Vec<_> = rows.into_iter().take(query.limit as usize).collect();

        // ── Build next_cursor from the last item ──────────────────────────────
        let next_cursor = if has_next {
            models.last().map(|last| {
                let field_value = match query.sort_field {
                    BranchSortField::CreatedAt => last.created_at.clone(),
                    BranchSortField::Name => last.name.clone(),
                };
                BranchCursor {
                    field_value,
                    id: last.id,
                }
            })
        } else {
            None
        };

        let items: Vec<Branch> = models.into_iter().map(|m| m.to_domain()).collect();

        Ok(BranchPage { items, next_cursor })
    }

    async fn set_all_is_main_false(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        except_id: Option<i64>,
    ) -> DomainResult<()> {
        let mut update_query = BranchEntity::update_many()
            .filter(BranchColumn::IsDeleted.eq(false))
            .col_expr(BranchColumn::IsMain, Expr::value(false))
            .col_expr(
                BranchColumn::UpdatedAt,
                Expr::value(
                    chrono::Utc::now()
                        .format("%Y-%m-%dT%H:%M:%S%.fZ")
                        .to_string(),
                ),
            );

        // Exclude the specified branch if provided
        if let Some(id) = except_id {
            update_query = update_query.filter(BranchColumn::Id.ne(id));
        }

        update_query.exec(&ctx.db).await?;
        Ok(())
    }
}
