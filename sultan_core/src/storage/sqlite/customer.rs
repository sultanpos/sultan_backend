use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QuerySelect, Set,
};

use crate::{
    domain::{
        DomainResult, Error,
        model::{
            customer::{Customer, CustomerCreate, CustomerFilter, CustomerUpdate},
            pagination::PaginationOptions,
        },
    },
    storage::{
        RepoCtx,
        customer_repo::CustomerRepository,
        sqlite::entity::{CustomerActiveModel, CustomerColumn, CustomerEntity},
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
        filter: &CustomerFilter,
        pagination: &PaginationOptions,
    ) -> DomainResult<Vec<Customer>> {
        use sea_orm::Condition;

        let mut query = CustomerEntity::find().filter(CustomerColumn::IsDeleted.eq(false));

        // Apply filters
        let mut condition = Condition::all();

        if let Some(number) = &filter.number {
            condition = condition.add(CustomerColumn::Number.contains(number));
        }

        if let Some(name) = &filter.name {
            condition = condition.add(CustomerColumn::Name.contains(name));
        }

        if let Some(email) = &filter.email {
            condition = condition.add(CustomerColumn::Email.contains(email));
        }

        if let Some(phone) = &filter.phone {
            condition = condition.add(CustomerColumn::Phone.contains(phone));
        }

        if let Some(level) = filter.level {
            condition = condition.add(CustomerColumn::Level.eq(level));
        }

        query = query.filter(condition);

        // Apply pagination
        let customers = query
            .limit(pagination.limit() as u64)
            .offset(pagination.offset() as u64)
            .all(&ctx.db)
            .await?;

        Ok(customers.into_iter().map(|c| c.to_domain()).collect())
    }
}
