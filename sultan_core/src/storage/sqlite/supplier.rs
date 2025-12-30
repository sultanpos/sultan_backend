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
            pagination::PaginationOptions,
            supplier::{Supplier, SupplierCreate, SupplierFilter, SupplierUpdate},
        },
    },
    storage::SupplierRepository,
};

#[derive(Clone)]
pub struct SqliteSupplierRepository {
    pool: SqlitePool,
}

impl SqliteSupplierRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

// Database model for Supplier - SQLite
#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct SupplierDbSqlite {
    pub id: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub is_deleted: bool,
    pub name: String,
    pub code: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub npwp: Option<String>,
    pub npwp_name: Option<String>,
    pub metadata: Option<String>,
}

impl From<SupplierDbSqlite> for Supplier {
    fn from(supplier_db: SupplierDbSqlite) -> Self {
        Supplier {
            id: supplier_db.id,
            created_at: super::parse_sqlite_date(&supplier_db.created_at),
            updated_at: super::parse_sqlite_date(&supplier_db.updated_at),
            deleted_at: supplier_db.deleted_at.map(|d| super::parse_sqlite_date(&d)),
            is_deleted: supplier_db.is_deleted,
            name: supplier_db.name,
            code: supplier_db.code,
            address: supplier_db.address,
            phone: supplier_db.phone,
            npwp: supplier_db.npwp,
            npwp_name: supplier_db.npwp_name,
            email: supplier_db.email,
            metadata: supplier_db
                .metadata
                .and_then(|m| serde_json::from_str(&m).ok()),
        }
    }
}

#[async_trait]
impl SupplierRepository for SqliteSupplierRepository {
    async fn create(&self, _: &Context, id: i64, supplier: &SupplierCreate) -> DomainResult<()> {
        let metadata_json = super::serialize_metadata(&supplier.metadata);

        let query = sqlx::query(
            r#"
            INSERT INTO suppliers (
                id, name, code, email, address, phone, npwp, npwp_name, metadata
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(&supplier.name)
        .bind(&supplier.code)
        .bind(&supplier.email)
        .bind(&supplier.address)
        .bind(&supplier.phone)
        .bind(&supplier.npwp)
        .bind(&supplier.npwp_name)
        .bind(&metadata_json)
        .execute(&self.pool);

        query.await?;
        Ok(())
    }

    async fn update(&self, _: &Context, id: i64, supplier: &SupplierUpdate) -> DomainResult<()> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new("UPDATE suppliers SET ");
        let mut separated = builder.separated(", ");

        if let Some(name) = &supplier.name {
            separated.push("name = ").push_bind_unseparated(name);
        }
        if supplier.code.should_update() {
            separated
                .push("code = ")
                .push_bind_unseparated(supplier.code.to_bind_value());
        }
        if supplier.email.should_update() {
            separated
                .push("email = ")
                .push_bind_unseparated(supplier.email.to_bind_value());
        }
        if supplier.address.should_update() {
            separated
                .push("address = ")
                .push_bind_unseparated(supplier.address.to_bind_value());
        }
        if supplier.phone.should_update() {
            separated
                .push("phone = ")
                .push_bind_unseparated(supplier.phone.to_bind_value());
        }
        if supplier.npwp.should_update() {
            separated
                .push("npwp = ")
                .push_bind_unseparated(supplier.npwp.to_bind_value());
        }
        if supplier.npwp_name.should_update() {
            separated
                .push("npwp_name = ")
                .push_bind_unseparated(supplier.npwp_name.to_bind_value());
        }
        if supplier.metadata.should_update() {
            let metadata_json = serialize_metadata_update(&supplier.metadata);
            separated
                .push("metadata = ")
                .push_bind_unseparated(metadata_json);
        }

        separated.push("updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')");
        builder.push(" WHERE id = ").push_bind(id);
        builder.push(" AND is_deleted = 0");

        let query = builder.build();
        let result = query.execute(&self.pool).await?;
        check_rows_affected(result.rows_affected(), "Supplier", id)
    }

    async fn delete(&self, _: &Context, id: i64) -> DomainResult<()> {
        let query = soft_delete(&self.pool, TableName::Suppliers, id);
        let result = query.await?;
        check_rows_affected(result.rows_affected(), "Supplier", id)
    }

    async fn get_all(
        &self,
        _: &Context,
        filter: &SupplierFilter,
        pagination: &PaginationOptions,
    ) -> DomainResult<Vec<Supplier>> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "SELECT id, created_at, updated_at, deleted_at, is_deleted, name, code, email, address, phone, npwp, npwp_name, metadata FROM suppliers WHERE is_deleted = 0",
        );

        builder
            .push_like_filter("name", &filter.name)
            .push_like_filter("code", &filter.code)
            .push_like_filter("email", &filter.email)
            .push_like_filter("phone", &filter.phone)
            .push_like_filter("npwp", &filter.npwp);

        builder.push(" ORDER BY id DESC");
        builder.push(" LIMIT ");
        builder.push_bind(pagination.limit());
        builder.push(" OFFSET ");
        builder.push_bind(pagination.offset());

        let query = builder.build_query_as::<SupplierDbSqlite>();
        let suppliers = query.fetch_all(&self.pool).await?;
        Ok(map_results(suppliers))
    }

    async fn get_by_id(&self, _: &Context, id: i64) -> DomainResult<Option<Supplier>> {
        let query = sqlx::query_as::<_, SupplierDbSqlite>(
            r#"
            SELECT id, created_at, updated_at, deleted_at, is_deleted, name, code, email, address, phone, npwp, npwp_name, metadata
            FROM suppliers WHERE id = ? AND is_deleted = 0
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool);

        Ok(query.await?.map(Supplier::from))
    }
}
