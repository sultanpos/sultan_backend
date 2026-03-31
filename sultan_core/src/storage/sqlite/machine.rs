use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, ExprTrait, Order,
    QueryFilter, QueryOrder, QuerySelect, Set, sea_query::Expr,
};

use super::entity::{MachineActiveModel, MachineColumn, MachineEntity};
use crate::{
    domain::{
        DomainResult,
        error::Error,
        model::{
            machine::{
                Machine, MachineCreate, MachineCursor, MachinePage, MachineQuery, MachineSortField,
                MachineUpdate,
            },
            product::SortDirection,
        },
    },
    storage::{MachineRepository, RepoCtx},
};

#[derive(Clone, Default)]
pub struct SqliteMachineRepository {}

impl SqliteMachineRepository {
    pub fn new() -> Self {
        SqliteMachineRepository {}
    }
}

#[async_trait]
impl MachineRepository for SqliteMachineRepository {
    async fn create(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        machine: &MachineCreate,
    ) -> DomainResult<()> {
        let metadata_json = machine
            .metadata
            .as_ref()
            .map(|m| serde_json::to_string(m).unwrap_or_default());

        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.fZ")
            .to_string();

        let model = MachineActiveModel {
            id: Set(id),
            created_at: Set(now.clone()),
            updated_at: Set(now),
            deleted_at: Set(None),
            is_deleted: Set(false),
            branch_id: Set(machine.branch_id),
            key: Set(machine.key.clone()),
            name: Set(machine.name.clone()),
            description: Set(machine.description.clone()),
            metadata: Set(metadata_json),
        };

        model.insert(&ctx.db).await?;
        Ok(())
    }

    async fn update(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        machine: &MachineUpdate,
    ) -> DomainResult<()> {
        use sea_orm::sea_query::Expr;

        let mut update_query = MachineEntity::update_many()
            .filter(MachineColumn::Id.eq(id))
            .filter(MachineColumn::IsDeleted.eq(false));

        if let Some(key) = &machine.key {
            update_query = update_query.col_expr(MachineColumn::Key, Expr::value(key.clone()));
        }

        if let Some(name) = &machine.name {
            update_query = update_query.col_expr(MachineColumn::Name, Expr::value(name.clone()));
        }

        if machine.description.should_update() {
            update_query = update_query.col_expr(
                MachineColumn::Description,
                Expr::value(machine.description.to_bind_value()),
            );
        }

        if machine.metadata.should_update() {
            let metadata_json = machine
                .metadata
                .as_value()
                .map(|m| serde_json::to_string(m).unwrap_or_default());
            update_query =
                update_query.col_expr(MachineColumn::Metadata, Expr::value(metadata_json));
        }

        update_query = update_query.col_expr(
            MachineColumn::UpdatedAt,
            Expr::value(
                chrono::Utc::now()
                    .format("%Y-%m-%dT%H:%M:%S%.fZ")
                    .to_string(),
            ),
        );

        let result = update_query.exec(&ctx.db).await?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!("Machine with id {} not found", id)));
        }

        Ok(())
    }

    async fn delete(&self, ctx: &RepoCtx<impl ConnectionTrait>, id: i64) -> DomainResult<()> {
        use sea_orm::sea_query::Expr;

        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.fZ")
            .to_string();

        let result = MachineEntity::update_many()
            .filter(MachineColumn::Id.eq(id))
            .filter(MachineColumn::IsDeleted.eq(false))
            .col_expr(MachineColumn::IsDeleted, Expr::value(true))
            .col_expr(MachineColumn::DeletedAt, Expr::value(Some(now.clone())))
            .col_expr(MachineColumn::UpdatedAt, Expr::value(now))
            .exec(&ctx.db)
            .await?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!("Machine with id {} not found", id)));
        }

        Ok(())
    }

    async fn get_by_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<Machine>> {
        let model = MachineEntity::find_by_id(id)
            .filter(MachineColumn::IsDeleted.eq(false))
            .one(&ctx.db)
            .await?;

        Ok(model.map(|m| m.to_domain()))
    }

    async fn get_all(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        query: &MachineQuery,
    ) -> DomainResult<MachinePage> {
        let mut select = MachineEntity::find().filter(MachineColumn::IsDeleted.eq(false));

        // ── Filters ──────────────────────────────────────────────────────
        if let Some(branch_id) = query.filter.branch_id {
            select = select.filter(MachineColumn::BranchId.eq(branch_id));
        }

        if let Some(name) = &query.filter.name {
            select = select.filter(MachineColumn::Name.contains(name));
        }

        // ── Map sort field to column ──────────────────────────────────────
        let sort_col = match query.sort_field {
            MachineSortField::Name => MachineColumn::Name,
            MachineSortField::CreatedAt => MachineColumn::CreatedAt,
        };

        let order = match query.sort_direction {
            SortDirection::Asc => Order::Asc,
            SortDirection::Desc => Order::Desc,
        };

        // ── Cursor condition ──────────────────────────────────────────────
        // WHERE (field > val) OR (field = val AND id > cursor_id)  [Asc]
        // WHERE (field < val) OR (field = val AND id < cursor_id)  [Desc]
        if let Some(cursor) = &query.cursor {
            let cond = match query.sort_direction {
                SortDirection::Asc => Condition::any()
                    .add(Expr::col(sort_col).gt(cursor.field_value.clone()))
                    .add(
                        Condition::all()
                            .add(Expr::col(sort_col).eq(cursor.field_value.clone()))
                            .add(Expr::col(MachineColumn::Id).gt(cursor.id)),
                    ),
                SortDirection::Desc => Condition::any()
                    .add(Expr::col(sort_col).lt(cursor.field_value.clone()))
                    .add(
                        Condition::all()
                            .add(Expr::col(sort_col).eq(cursor.field_value.clone()))
                            .add(Expr::col(MachineColumn::Id).lt(cursor.id)),
                    ),
            };
            select = select.filter(cond);
        }

        // ── Ordering: (sort_field, id) ────────────────────────────────────
        select = select
            .order_by(sort_col, order.clone())
            .order_by(MachineColumn::Id, order);

        // Fetch limit + 1 to detect whether there is a next page
        let fetch_limit = query.limit + 1;
        let rows = select.limit(fetch_limit).all(&ctx.db).await?;

        let has_next = rows.len() as u64 > query.limit;
        let models: Vec<_> = rows.into_iter().take(query.limit as usize).collect();

        // ── Build next_cursor from the last item ──────────────────────────
        let next_cursor = if has_next {
            models.last().map(|last| {
                let field_value = match query.sort_field {
                    MachineSortField::Name => last.name.clone(),
                    MachineSortField::CreatedAt => last.created_at.clone(),
                };
                MachineCursor {
                    field_value,
                    id: last.id,
                }
            })
        } else {
            None
        };

        let items = models.into_iter().map(|m| m.to_domain()).collect();

        Ok(MachinePage { items, next_cursor })
    }
}
