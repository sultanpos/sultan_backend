use async_trait::async_trait;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, ExprTrait, Order,
    QueryFilter, QueryOrder, QuerySelect, Set,
};

use super::entity::{SupplierActiveModel, SupplierColumn, SupplierEntity, SupplierModel};
use crate::{
    domain::{
        DomainResult,
        error::Error,
        model::supplier::{
            Supplier, SupplierCreate, SupplierCursor, SupplierPage, SupplierQuery, SupplierUpdate,
        },
    },
    storage::{RepoCtx, SupplierRepository},
};

/// SQLite implementation of [`SupplierRepository`] using SeaORM.
///
/// This repository handles all supplier-related database operations
/// including CRUD operations, soft-delete, and filtered queries.
#[derive(Clone, Default)]
pub struct SqliteSupplierRepository {}

impl SqliteSupplierRepository {
    pub fn new() -> Self {
        SqliteSupplierRepository {}
    }
}

#[async_trait]
impl SupplierRepository for SqliteSupplierRepository {
    async fn create(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        supplier: &SupplierCreate,
    ) -> DomainResult<()> {
        let metadata_json = supplier
            .metadata
            .as_ref()
            .map(|m| serde_json::to_string(m).unwrap_or_default());

        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.fZ")
            .to_string();

        let supplier_model = SupplierActiveModel {
            id: Set(id),
            created_at: Set(now.clone()),
            updated_at: Set(now),
            deleted_at: Set(None),
            is_deleted: Set(false),
            name: Set(supplier.name.clone()),
            code: Set(supplier.code.clone()),
            email: Set(supplier.email.clone()),
            address: Set(supplier.address.clone()),
            phone: Set(supplier.phone.clone()),
            npwp: Set(supplier.npwp.clone()),
            npwp_name: Set(supplier.npwp_name.clone()),
            metadata: Set(metadata_json),
        };

        supplier_model.insert(&ctx.db).await?;
        Ok(())
    }

    async fn update(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        supplier: &SupplierUpdate,
    ) -> DomainResult<()> {
        use sea_orm::sea_query::Expr;

        let mut update_query = SupplierEntity::update_many()
            .filter(SupplierColumn::Id.eq(id))
            .filter(SupplierColumn::IsDeleted.eq(false));

        // Update fields if provided
        if let Some(name) = &supplier.name {
            update_query = update_query.col_expr(SupplierColumn::Name, Expr::value(name.clone()));
        }

        if supplier.code.should_update() {
            update_query = update_query.col_expr(
                SupplierColumn::Code,
                Expr::value(supplier.code.to_bind_value()),
            );
        }

        if supplier.email.should_update() {
            update_query = update_query.col_expr(
                SupplierColumn::Email,
                Expr::value(supplier.email.to_bind_value()),
            );
        }

        if supplier.address.should_update() {
            update_query = update_query.col_expr(
                SupplierColumn::Address,
                Expr::value(supplier.address.to_bind_value()),
            );
        }

        if supplier.phone.should_update() {
            update_query = update_query.col_expr(
                SupplierColumn::Phone,
                Expr::value(supplier.phone.to_bind_value()),
            );
        }

        if supplier.npwp.should_update() {
            update_query = update_query.col_expr(
                SupplierColumn::Npwp,
                Expr::value(supplier.npwp.to_bind_value()),
            );
        }

        if supplier.npwp_name.should_update() {
            update_query = update_query.col_expr(
                SupplierColumn::NpwpName,
                Expr::value(supplier.npwp_name.to_bind_value()),
            );
        }

        if supplier.metadata.should_update() {
            let metadata_json = supplier
                .metadata
                .as_value()
                .map(|m| serde_json::to_string(m).unwrap_or_default());
            update_query =
                update_query.col_expr(SupplierColumn::Metadata, Expr::value(metadata_json));
        }

        // Always update the updated_at timestamp
        update_query = update_query.col_expr(
            SupplierColumn::UpdatedAt,
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
            return Err(Error::NotFound(format!(
                "Supplier with id {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn delete(&self, ctx: &RepoCtx<impl ConnectionTrait>, id: i64) -> DomainResult<()> {
        use sea_orm::sea_query::Expr;

        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.fZ")
            .to_string();

        let result = SupplierEntity::update_many()
            .filter(SupplierColumn::Id.eq(id))
            .filter(SupplierColumn::IsDeleted.eq(false))
            .col_expr(SupplierColumn::IsDeleted, Expr::value(true))
            .col_expr(SupplierColumn::DeletedAt, Expr::value(Some(now.clone())))
            .col_expr(SupplierColumn::UpdatedAt, Expr::value(now))
            .exec(&ctx.db)
            .await?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!(
                "Supplier with id {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn get_all(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        query: &SupplierQuery,
    ) -> DomainResult<SupplierPage> {
        use crate::domain::model::product::SortDirection;
        use crate::domain::model::supplier::SupplierSortField;

        let mut select = SupplierEntity::find().filter(SupplierColumn::IsDeleted.eq(false));

        // ── Filters ──────────────────────────────────────────────────────────
        let mut condition = Condition::all();

        if let Some(name) = &query.filter.name {
            condition = condition.add(SupplierColumn::Name.contains(name));
        }

        if let Some(code) = &query.filter.code {
            condition = condition.add(SupplierColumn::Code.contains(code));
        }

        if let Some(email) = &query.filter.email {
            condition = condition.add(SupplierColumn::Email.contains(email));
        }

        if let Some(phone) = &query.filter.phone {
            condition = condition.add(SupplierColumn::Phone.contains(phone));
        }

        if let Some(npwp) = &query.filter.npwp {
            condition = condition.add(SupplierColumn::Npwp.contains(npwp));
        }

        select = select.filter(condition);

        // ── Sort direction ────────────────────────────────────────────────────
        let order = match query.sort_direction {
            SortDirection::Asc => Order::Asc,
            SortDirection::Desc => Order::Desc,
        };

        // ── Cursor condition ──────────────────────────────────────────────────
        if let Some(cursor) = &query.cursor {
            let cond = match query.sort_field {
                // For Id sort, id IS the sort column — no tiebreaker needed
                SupplierSortField::Id => match query.sort_direction {
                    SortDirection::Asc => {
                        Condition::all().add(Expr::col(SupplierColumn::Id).gt(cursor.id))
                    }
                    SortDirection::Desc => {
                        Condition::all().add(Expr::col(SupplierColumn::Id).lt(cursor.id))
                    }
                },
                // For string/date fields: (field > val) OR (field = val AND id > cursor_id)
                SupplierSortField::UpdatedAt | SupplierSortField::Name => {
                    let sort_col = match query.sort_field {
                        SupplierSortField::UpdatedAt => SupplierColumn::UpdatedAt,
                        SupplierSortField::Name => SupplierColumn::Name,
                        SupplierSortField::Id => unreachable!(),
                    };
                    match query.sort_direction {
                        SortDirection::Asc => Condition::any()
                            .add(Expr::col(sort_col).gt(cursor.field_value.clone()))
                            .add(
                                Condition::all()
                                    .add(Expr::col(sort_col).eq(cursor.field_value.clone()))
                                    .add(Expr::col(SupplierColumn::Id).gt(cursor.id)),
                            ),
                        SortDirection::Desc => Condition::any()
                            .add(Expr::col(sort_col).lt(cursor.field_value.clone()))
                            .add(
                                Condition::all()
                                    .add(Expr::col(sort_col).eq(cursor.field_value.clone()))
                                    .add(Expr::col(SupplierColumn::Id).lt(cursor.id)),
                            ),
                    }
                }
            };
            select = select.filter(cond);
        }

        // ── Ordering: (sort_field, id) ────────────────────────────────────────
        select = match query.sort_field {
            SupplierSortField::Id => select.order_by(SupplierColumn::Id, order),
            SupplierSortField::UpdatedAt => select
                .order_by(SupplierColumn::UpdatedAt, order.clone())
                .order_by(SupplierColumn::Id, order),
            SupplierSortField::Name => select
                .order_by(SupplierColumn::Name, order.clone())
                .order_by(SupplierColumn::Id, order),
        };

        // Fetch limit + 1 to detect whether there is a next page
        let fetch_limit = query.limit + 1;
        let rows: Vec<SupplierModel> = select.limit(fetch_limit).all(&ctx.db).await?;

        let has_next = rows.len() as u64 > query.limit;
        let models: Vec<_> = rows.into_iter().take(query.limit as usize).collect();

        // ── Build next_cursor from the last item ──────────────────────────────
        let next_cursor = if has_next {
            models.last().map(|last| {
                let field_value = match query.sort_field {
                    SupplierSortField::Id => last.id.to_string(),
                    SupplierSortField::UpdatedAt => last.updated_at.clone(),
                    SupplierSortField::Name => last.name.clone(),
                };
                SupplierCursor {
                    field_value,
                    id: last.id,
                }
            })
        } else {
            None
        };

        let items: Vec<Supplier> = models.into_iter().map(|m| m.to_domain()).collect();

        Ok(SupplierPage { items, next_cursor })
    }

    async fn get_by_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<Supplier>> {
        let supplier = SupplierEntity::find_by_id(id)
            .filter(SupplierColumn::IsDeleted.eq(false))
            .one(&ctx.db)
            .await?;

        Ok(supplier.map(|s| s.to_domain()))
    }
}
