use async_trait::async_trait;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, ExprTrait, Order,
    QueryFilter, QueryOrder, QuerySelect, Set,
};

use crate::{
    domain::{
        DomainResult, Error,
        model::customer::{
            Customer, CustomerCreate, CustomerCursor, CustomerPage, CustomerQuery, CustomerUpdate,
        },
    },
    storage::{
        RepoCtx,
        customer_repo::CustomerRepository,
        sqlite::entity::{CustomerActiveModel, CustomerColumn, CustomerEntity, CustomerModel},
    },
};

/// SQLite implementation of CustomerRepository using SeaORM.
///
/// This repository uses SeaORM's `ConnectionTrait` which allows it to work
/// with both direct database connections and transactions seamlessly.
#[derive(Clone, Default)]
pub struct SqliteCustomerRepository {}

impl SqliteCustomerRepository {
    pub fn new() -> Self {
        SqliteCustomerRepository {}
    }
}

#[async_trait]
impl CustomerRepository for SqliteCustomerRepository {
    async fn create(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        customer: &CustomerCreate,
    ) -> DomainResult<()> {
        let metadata_json = customer
            .metadata
            .as_ref()
            .map(|m| serde_json::to_string(m).unwrap_or_default());

        let customer_model = CustomerActiveModel {
            id: Set(id),
            number: Set(customer.number.clone()),
            name: Set(customer.name.clone()),
            address: Set(customer.address.clone()),
            email: Set(customer.email.clone()),
            phone: Set(customer.phone.clone()),
            level: Set(customer.level),
            metadata: Set(metadata_json),
            ..Default::default()
        };

        customer_model.insert(&ctx.db).await.map_err(|e| {
            if let sea_orm::DbErr::Query(sea_orm::RuntimeErr::SqlxError(sqlx_err)) = &e
                && let sqlx::Error::Database(db_err) = sqlx_err.as_ref()
                && db_err.is_unique_violation()
            {
                return Error::Conflict(format!(
                    "Customer with number '{}' already exists",
                    customer.number
                ));
            }
            e.into()
        })?;

        Ok(())
    }

    async fn get_by_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<Customer>> {
        let customer = CustomerEntity::find_by_id(id)
            .filter(CustomerColumn::IsDeleted.eq(false))
            .one(&ctx.db)
            .await?;

        Ok(customer.map(|c| c.to_domain()))
    }

    async fn get_by_number(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        number: &str,
    ) -> DomainResult<Option<Customer>> {
        let customer = CustomerEntity::find()
            .filter(CustomerColumn::Number.eq(number))
            .filter(CustomerColumn::IsDeleted.eq(false))
            .one(&ctx.db)
            .await?;

        Ok(customer.map(|c| c.to_domain()))
    }

    async fn update(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        customer: &CustomerUpdate,
    ) -> DomainResult<()> {
        use sea_orm::{UpdateMany, sea_query::Expr};

        // Build update query with filters
        let mut update_query: UpdateMany<CustomerEntity> = CustomerEntity::update_many()
            .filter(CustomerColumn::Id.eq(id))
            .filter(CustomerColumn::IsDeleted.eq(false));

        // Update fields if provided
        if let Some(number) = &customer.number {
            update_query =
                update_query.col_expr(CustomerColumn::Number, Expr::value(number.clone()));
        }

        if let Some(name) = &customer.name {
            update_query = update_query.col_expr(CustomerColumn::Name, Expr::value(name.clone()));
        }

        if customer.address.should_update() {
            update_query = update_query.col_expr(
                CustomerColumn::Address,
                Expr::value(customer.address.to_bind_value()),
            );
        }

        if customer.email.should_update() {
            update_query = update_query.col_expr(
                CustomerColumn::Email,
                Expr::value(customer.email.to_bind_value()),
            );
        }

        if customer.phone.should_update() {
            update_query = update_query.col_expr(
                CustomerColumn::Phone,
                Expr::value(customer.phone.to_bind_value()),
            );
        }

        if let Some(level) = customer.level {
            update_query = update_query.col_expr(CustomerColumn::Level, Expr::value(level));
        }

        if customer.metadata.should_update() {
            let metadata_json = customer
                .metadata
                .as_value()
                .map(|m| serde_json::to_string(m).unwrap_or_default());
            update_query =
                update_query.col_expr(CustomerColumn::Metadata, Expr::value(metadata_json));
        }

        // Always update the updated_at timestamp
        update_query = update_query.col_expr(
            CustomerColumn::UpdatedAt,
            Expr::value(
                chrono::Utc::now()
                    .format("%Y-%m-%dT%H:%M:%S%.fZ")
                    .to_string(),
            ),
        );

        // Execute the update
        let result = update_query.exec(&ctx.db).await.map_err(|e| {
            if let sea_orm::DbErr::Query(sea_orm::RuntimeErr::SqlxError(sqlx_err)) = &e
                && let sqlx::Error::Database(db_err) = sqlx_err.as_ref()
                && db_err.is_unique_violation()
            {
                return Error::Conflict(format!(
                    "Customer with number '{}' already exists",
                    customer.number.as_ref().unwrap_or(&"unknown".to_string())
                ));
            }
            e.into()
        })?;

        // Check if any rows were affected
        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!(
                "Customer with id {} not found",
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

        let result = CustomerEntity::update_many()
            .filter(CustomerColumn::Id.eq(id))
            .filter(CustomerColumn::IsDeleted.eq(false))
            .col_expr(CustomerColumn::IsDeleted, Expr::value(true))
            .col_expr(CustomerColumn::DeletedAt, Expr::value(Some(now.clone())))
            .col_expr(CustomerColumn::UpdatedAt, Expr::value(now))
            .exec(&ctx.db)
            .await?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!(
                "Customer with id {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn get_all(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        query: &CustomerQuery,
    ) -> DomainResult<CustomerPage> {
        use crate::domain::model::customer::CustomerSortField;
        use crate::domain::model::product::SortDirection;

        let mut select = CustomerEntity::find().filter(CustomerColumn::IsDeleted.eq(false));

        // ── Filters ──────────────────────────────────────────────────────────
        let mut condition = Condition::all();

        if let Some(number) = &query.filter.number {
            condition = condition.add(CustomerColumn::Number.contains(number));
        }

        if let Some(name) = &query.filter.name {
            condition = condition.add(CustomerColumn::Name.contains(name));
        }

        if let Some(email) = &query.filter.email {
            condition = condition.add(CustomerColumn::Email.contains(email));
        }

        if let Some(phone) = &query.filter.phone {
            condition = condition.add(CustomerColumn::Phone.contains(phone));
        }

        if let Some(level) = query.filter.level {
            condition = condition.add(CustomerColumn::Level.eq(level));
        }

        select = select.filter(condition);

        // ── Map sort field to column ──────────────────────────────────────────
        let order = match query.sort_direction {
            SortDirection::Asc => Order::Asc,
            SortDirection::Desc => Order::Desc,
        };

        // ── Cursor condition ──────────────────────────────────────────────────
        if let Some(cursor) = &query.cursor {
            let cond = match query.sort_field {
                // For Id sort, id IS the sort column — no tiebreaker needed
                CustomerSortField::Id => match query.sort_direction {
                    SortDirection::Asc => {
                        Condition::all().add(Expr::col(CustomerColumn::Id).gt(cursor.id))
                    }
                    SortDirection::Desc => {
                        Condition::all().add(Expr::col(CustomerColumn::Id).lt(cursor.id))
                    }
                },
                // For string/date fields: (field > val) OR (field = val AND id > cursor_id)
                CustomerSortField::UpdatedAt | CustomerSortField::Name => {
                    let sort_col = match query.sort_field {
                        CustomerSortField::UpdatedAt => CustomerColumn::UpdatedAt,
                        CustomerSortField::Name => CustomerColumn::Name,
                        CustomerSortField::Id => unreachable!(),
                    };
                    match query.sort_direction {
                        SortDirection::Asc => Condition::any()
                            .add(Expr::col(sort_col).gt(cursor.field_value.clone()))
                            .add(
                                Condition::all()
                                    .add(Expr::col(sort_col).eq(cursor.field_value.clone()))
                                    .add(Expr::col(CustomerColumn::Id).gt(cursor.id)),
                            ),
                        SortDirection::Desc => Condition::any()
                            .add(Expr::col(sort_col).lt(cursor.field_value.clone()))
                            .add(
                                Condition::all()
                                    .add(Expr::col(sort_col).eq(cursor.field_value.clone()))
                                    .add(Expr::col(CustomerColumn::Id).lt(cursor.id)),
                            ),
                    }
                }
            };
            select = select.filter(cond);
        }

        // ── Ordering: (sort_field, id) ────────────────────────────────────────
        select = match query.sort_field {
            CustomerSortField::Id => select.order_by(CustomerColumn::Id, order),
            CustomerSortField::UpdatedAt => select
                .order_by(CustomerColumn::UpdatedAt, order.clone())
                .order_by(CustomerColumn::Id, order),
            CustomerSortField::Name => select
                .order_by(CustomerColumn::Name, order.clone())
                .order_by(CustomerColumn::Id, order),
        };

        // Fetch limit + 1 to detect whether there is a next page
        let fetch_limit = query.limit + 1;
        let rows: Vec<CustomerModel> = select.limit(fetch_limit).all(&ctx.db).await?;

        let has_next = rows.len() as u64 > query.limit;
        let models: Vec<_> = rows.into_iter().take(query.limit as usize).collect();

        // ── Build next_cursor from the last item ──────────────────────────────
        let next_cursor = if has_next {
            models.last().map(|last| {
                let field_value = match query.sort_field {
                    CustomerSortField::Id => last.id.to_string(),
                    CustomerSortField::UpdatedAt => last.updated_at.clone(),
                    CustomerSortField::Name => last.name.clone(),
                };
                CustomerCursor {
                    field_value,
                    id: last.id,
                }
            })
        } else {
            None
        };

        let items: Vec<Customer> = models.into_iter().map(|m| m.to_domain()).collect();

        Ok(CustomerPage { items, next_cursor })
    }
}
