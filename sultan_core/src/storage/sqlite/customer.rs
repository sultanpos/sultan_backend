use async_trait::async_trait;
use serde::Serialize;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use super::{
    QueryBuilderExt, TableName, check_rows_affected, map_results, serialize_metadata_update,
    soft_delete,
};
use crate::{
    domain::{
        Context, DomainResult,
        model::{
            customer::{Customer, CustomerCreate, CustomerFilter, CustomerUpdate},
            pagination::PaginationOptions,
        },
    },
    storage::CustomerRepository,
};

#[derive(Clone)]
pub struct SqliteCustomerRepository {
    pool: SqlitePool,
}

impl SqliteCustomerRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

// Database model for Customer - SQLite
#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct CustomerDbSqlite {
    pub id: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub is_deleted: bool,
    pub number: String,
    pub name: String,
    pub address: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub level: i32,
    pub metadata: Option<String>,
}

impl From<CustomerDbSqlite> for Customer {
    fn from(customer_db: CustomerDbSqlite) -> Self {
        Customer {
            id: customer_db.id,
            created_at: super::parse_sqlite_date(&customer_db.created_at),
            updated_at: super::parse_sqlite_date(&customer_db.updated_at),
            deleted_at: customer_db.deleted_at.map(|d| super::parse_sqlite_date(&d)),
            is_deleted: customer_db.is_deleted,
            number: customer_db.number,
            name: customer_db.name,
            address: customer_db.address,
            email: customer_db.email,
            phone: customer_db.phone,
            level: customer_db.level,
            metadata: customer_db
                .metadata
                .and_then(|m| serde_json::from_str(&m).ok()),
        }
    }
}

#[async_trait]
impl CustomerRepository for SqliteCustomerRepository {
    async fn create(&self, _: &Context, id: i64, customer: &CustomerCreate) -> DomainResult<()> {
        let metadata_json = super::serialize_metadata(&customer.metadata);

        let query = sqlx::query(
            r#"
            INSERT INTO customers (
                id, number, name, address, email, phone, level, metadata
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(&customer.number)
        .bind(&customer.name)
        .bind(&customer.address)
        .bind(&customer.email)
        .bind(&customer.phone)
        .bind(customer.level)
        .bind(&metadata_json)
        .execute(&self.pool);

        query.await?;
        Ok(())
    }

    async fn update(&self, _: &Context, id: i64, customer: &CustomerUpdate) -> DomainResult<()> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new("UPDATE customers SET ");
        let mut separated = builder.separated(", ");

        if let Some(number) = &customer.number {
            separated.push("number = ").push_bind_unseparated(number);
        }
        if let Some(name) = &customer.name {
            separated.push("name = ").push_bind_unseparated(name);
        }
        if customer.address.should_update() {
            separated
                .push("address = ")
                .push_bind_unseparated(customer.address.to_bind_value());
        }
        if customer.email.should_update() {
            separated
                .push("email = ")
                .push_bind_unseparated(customer.email.to_bind_value());
        }
        if customer.phone.should_update() {
            separated
                .push("phone = ")
                .push_bind_unseparated(customer.phone.to_bind_value());
        }
        if let Some(level) = customer.level {
            separated.push("level = ").push_bind_unseparated(level);
        }
        if customer.metadata.should_update() {
            let metadata_json = serialize_metadata_update(&customer.metadata);
            separated
                .push("metadata = ")
                .push_bind_unseparated(metadata_json);
        }

        separated.push("updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')");
        builder.push(" WHERE id = ").push_bind(id);
        builder.push(" AND is_deleted = 0");

        let query = builder.build();
        let result = query.execute(&self.pool).await?;
        check_rows_affected(result.rows_affected(), "Customer", id)
    }

    async fn delete(&self, _: &Context, id: i64) -> DomainResult<()> {
        let query = soft_delete(&self.pool, TableName::Customers, id);
        let result = query.await?;
        check_rows_affected(result.rows_affected(), "Customer", id)
    }

    async fn get_by_number(&self, _: &Context, number: &str) -> DomainResult<Option<Customer>> {
        let query = sqlx::query_as::<_, CustomerDbSqlite>(
            r#"
            SELECT id, created_at, updated_at, deleted_at, is_deleted, number, name, address, email, phone, level, metadata
            FROM customers WHERE number = ? AND is_deleted = 0
            "#,
        )
        .bind(number)
        .fetch_optional(&self.pool);

        let customer = query.await?;

        Ok(customer.map(|c| c.into()))
    }

    async fn get_by_id(&self, _: &Context, id: i64) -> DomainResult<Option<Customer>> {
        let query = sqlx::query_as::<_, CustomerDbSqlite>(
            r#"
            SELECT id, created_at, updated_at, deleted_at, is_deleted, number, name, address, email, phone, level, metadata
            FROM customers WHERE id = ? AND is_deleted = 0
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool);

        Ok(query.await?.map(Customer::from))
    }

    async fn get_all(
        &self,
        _: &Context,
        filter: &CustomerFilter,
        pagination: &PaginationOptions,
    ) -> DomainResult<Vec<Customer>> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "SELECT id, created_at, updated_at, deleted_at, is_deleted, number, name, address, email, phone, level, metadata FROM customers WHERE is_deleted = 0",
        );

        builder
            .push_like_filter("number", &filter.number)
            .push_like_filter("name", &filter.name)
            .push_like_filter("email", &filter.email)
            .push_like_filter("phone", &filter.phone);

        if let Some(level) = filter.level {
            builder.push(" AND level = ");
            builder.push_bind(level);
        }

        builder.push(" ORDER BY id DESC");
        builder.push(" LIMIT ");
        builder.push_bind(pagination.limit());
        builder.push(" OFFSET ");
        builder.push_bind(pagination.offset());

        let query = builder.build_query_as::<CustomerDbSqlite>();
        let customers = query.fetch_all(&self.pool).await?;
        Ok(map_results(customers))
    }
}
