use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, ExprTrait, Order,
    QueryFilter, QueryOrder, QuerySelect, Set, sea_query::Expr,
};

use crate::{
    domain::{
        DomainResult, Error,
        model::product::{
            SortDirection, UnitCursor, UnitOfMeasure, UnitOfMeasureCreate, UnitOfMeasureUpdate,
            UnitPage, UnitQuery, UnitSortField,
        },
    },
    storage::{
        RepoCtx,
        sqlite::entity::{UnitActiveModel, UnitColumn, UnitEntity},
        unit_repo::UnitOfMeasureRepository,
    },
};

/// SQLite implementation of UnitOfMeasureRepository using SeaORM.
///
/// This repository uses SeaORM's `ConnectionTrait` which allows it to work
/// with both direct database connections and transactions seamlessly.
///
/// # Example
///
/// ```rust,ignore
/// // Using with direct connection
/// let repo = SqliteUnitOfMeasureRepository::new();
/// let ctx = RepoCtx { ctx: Context::new(), db: &db_connection };
/// repo.create(&ctx, id, &uom).await?;
///
/// // Using within a transaction
/// let txn = db.begin().await?;
/// let ctx = RepoCtx { ctx: Context::new(), db: &txn };
/// repo.create(&ctx, id, &uom).await?;
/// txn.commit().await?;
/// ```
#[derive(Clone, Default)]
pub struct SqliteUnitOfMeasureRepository {}

impl SqliteUnitOfMeasureRepository {
    pub fn new() -> Self {
        SqliteUnitOfMeasureRepository {}
    }
}

#[async_trait]
impl UnitOfMeasureRepository for SqliteUnitOfMeasureRepository {
    async fn create(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        uom: &UnitOfMeasureCreate,
    ) -> DomainResult<()> {
        let unit = UnitActiveModel {
            id: Set(id),
            name: Set(uom.name.clone()),
            description: Set(uom.description.clone()),
            ..Default::default()
        };

        unit.insert(&ctx.db).await?;
        Ok(())
    }

    async fn get_by_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<UnitOfMeasure>> {
        let unit = UnitEntity::find_by_id(id)
            .filter(UnitColumn::IsDeleted.eq(false))
            .one(&ctx.db)
            .await?;

        Ok(unit.map(|u| u.to_domain()))
    }

    async fn update(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        uom: &UnitOfMeasureUpdate,
    ) -> DomainResult<()> {
        use sea_orm::{UpdateMany, sea_query::Expr};

        // Build update query with filters
        let mut update_query: UpdateMany<UnitEntity> = UnitEntity::update_many()
            .filter(UnitColumn::Id.eq(id))
            .filter(UnitColumn::IsDeleted.eq(false));

        // Update fields if provided
        if let Some(name) = &uom.name {
            update_query = update_query.col_expr(UnitColumn::Name, Expr::value(name.clone()));
        }

        if uom.description.should_update() {
            update_query = update_query.col_expr(
                UnitColumn::Description,
                Expr::value(uom.description.to_bind_value()),
            );
        }

        // Always update the updated_at timestamp
        update_query = update_query.col_expr(
            UnitColumn::UpdatedAt,
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
            return Err(Error::NotFound(format!("Unit with id {} not found", id)));
        }

        Ok(())
    }

    async fn delete(&self, ctx: &RepoCtx<impl ConnectionTrait>, id: i64) -> DomainResult<()> {
        use sea_orm::sea_query::Expr;

        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.fZ")
            .to_string();

        // Soft delete: mark as deleted with a single UPDATE query
        let result = UnitEntity::update_many()
            .filter(UnitColumn::Id.eq(id))
            .filter(UnitColumn::IsDeleted.eq(false))
            .col_expr(UnitColumn::IsDeleted, Expr::value(true))
            .col_expr(UnitColumn::DeletedAt, Expr::value(Some(now.clone())))
            .col_expr(UnitColumn::UpdatedAt, Expr::value(now))
            .exec(&ctx.db)
            .await?;

        // Check if any rows were affected
        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!("Unit with id {} not found", id)));
        }

        Ok(())
    }

    async fn get_all(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        query: &UnitQuery,
    ) -> DomainResult<UnitPage> {
        let mut select = UnitEntity::find().filter(UnitColumn::IsDeleted.eq(false));

        // ── Map sort direction ────────────────────────────────────────────────
        let order = match query.sort_direction {
            SortDirection::Asc => Order::Asc,
            SortDirection::Desc => Order::Desc,
        };

        // ── Cursor condition ──────────────────────────────────────────────────
        if let Some(cursor) = &query.cursor {
            let cond = match query.sort_field {
                // For Id sort, id IS the sort column — no tiebreaker needed
                UnitSortField::Id => match query.sort_direction {
                    SortDirection::Asc => {
                        Condition::all().add(Expr::col(UnitColumn::Id).gt(cursor.id))
                    }
                    SortDirection::Desc => {
                        Condition::all().add(Expr::col(UnitColumn::Id).lt(cursor.id))
                    }
                },
                // For string/date fields: (field > val) OR (field = val AND id > cursor_id)
                UnitSortField::UpdatedAt | UnitSortField::Name => {
                    let sort_col = match query.sort_field {
                        UnitSortField::UpdatedAt => UnitColumn::UpdatedAt,
                        UnitSortField::Name => UnitColumn::Name,
                        UnitSortField::Id => unreachable!(),
                    };
                    match query.sort_direction {
                        SortDirection::Asc => Condition::any()
                            .add(Expr::col(sort_col).gt(cursor.field_value.clone()))
                            .add(
                                Condition::all()
                                    .add(Expr::col(sort_col).eq(cursor.field_value.clone()))
                                    .add(Expr::col(UnitColumn::Id).gt(cursor.id)),
                            ),
                        SortDirection::Desc => Condition::any()
                            .add(Expr::col(sort_col).lt(cursor.field_value.clone()))
                            .add(
                                Condition::all()
                                    .add(Expr::col(sort_col).eq(cursor.field_value.clone()))
                                    .add(Expr::col(UnitColumn::Id).lt(cursor.id)),
                            ),
                    }
                }
            };
            select = select.filter(cond);
        }

        // ── Ordering: (sort_field, id) ────────────────────────────────────────
        select = match query.sort_field {
            UnitSortField::Id => select.order_by(UnitColumn::Id, order),
            UnitSortField::UpdatedAt => select
                .order_by(UnitColumn::UpdatedAt, order.clone())
                .order_by(UnitColumn::Id, order),
            UnitSortField::Name => select
                .order_by(UnitColumn::Name, order.clone())
                .order_by(UnitColumn::Id, order),
        };

        // Fetch limit + 1 to detect whether there is a next page
        let fetch_limit = query.limit + 1;
        let rows = select.limit(fetch_limit).all(&ctx.db).await?;

        let has_next = rows.len() as u64 > query.limit;
        let models: Vec<_> = rows.into_iter().take(query.limit as usize).collect();

        // ── Build next_cursor from the last item ──────────────────────────────
        let next_cursor = if has_next {
            models.last().map(|last| {
                let field_value = match query.sort_field {
                    UnitSortField::Id => last.id.to_string(),
                    UnitSortField::UpdatedAt => last.updated_at.clone(),
                    UnitSortField::Name => last.name.clone(),
                };
                UnitCursor {
                    field_value,
                    id: last.id,
                }
            })
        } else {
            None
        };

        let items: Vec<UnitOfMeasure> = models.into_iter().map(|m| m.to_domain()).collect();

        Ok(UnitPage { items, next_cursor })
    }
}
