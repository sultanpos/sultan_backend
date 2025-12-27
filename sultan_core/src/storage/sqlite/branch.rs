use async_trait::async_trait;

use serde::Serialize;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::{
    domain::{
        Context, DomainResult, Error,
        model::branch::{Branch, BranchCreate, BranchUpdate},
    },
    storage::branch_repo::BranchRepository,
};

#[derive(Clone)]
pub struct SqliteBranchRepository {
    pool: SqlitePool,
}

impl SqliteBranchRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

// Database model for Branch - SQLite
#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct BranchDbSqlite {
    pub id: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub is_deleted: bool,
    pub is_main: bool,
    pub name: String,
    pub code: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub npwp: Option<String>,
    pub image: Option<String>,
}

impl From<BranchDbSqlite> for Branch {
    fn from(branch_db: BranchDbSqlite) -> Self {
        Branch {
            id: branch_db.id,
            created_at: super::parse_sqlite_date(&branch_db.created_at),
            updated_at: super::parse_sqlite_date(&branch_db.updated_at),
            deleted_at: branch_db.deleted_at.map(|d| super::parse_sqlite_date(&d)),
            is_deleted: branch_db.is_deleted,
            is_main: branch_db.is_main,
            name: branch_db.name,
            code: branch_db.code,
            address: branch_db.address,
            phone: branch_db.phone,
            npwp: branch_db.npwp,
            image: branch_db.image,
        }
    }
}

#[async_trait]
impl BranchRepository for SqliteBranchRepository {
    async fn create(&self, _: &Context, id: i64, branch: &BranchCreate) -> DomainResult<()> {
        let query = sqlx::query(
            r#"
            INSERT INTO branches (
                id, name, code, address, phone, npwp, image, is_main
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(&branch.name)
        .bind(&branch.code)
        .bind(&branch.address)
        .bind(&branch.phone)
        .bind(&branch.npwp)
        .bind(&branch.image)
        .bind(branch.is_main)
        .execute(&self.pool);

        let result = query.await?;

        if result.rows_affected() == 0 {
            return Err(Error::Database("Failed to insert branch".to_string()));
        }

        Ok(())
    }

    async fn update(&self, _: &Context, id: i64, branch: &BranchUpdate) -> DomainResult<()> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new("UPDATE branches SET ");
        let mut separated = builder.separated(", ");

        if let Some(name) = &branch.name {
            separated.push("name = ").push_bind_unseparated(name);
        }
        if let Some(code) = &branch.code {
            separated.push("code = ").push_bind_unseparated(code);
        }
        if branch.address.should_update() {
            separated
                .push("address = ")
                .push_bind_unseparated(branch.address.to_bind_value());
        }
        if branch.phone.should_update() {
            separated
                .push("phone = ")
                .push_bind_unseparated(branch.phone.to_bind_value());
        }
        if branch.npwp.should_update() {
            separated
                .push("npwp = ")
                .push_bind_unseparated(branch.npwp.to_bind_value());
        }
        if branch.image.should_update() {
            separated
                .push("image = ")
                .push_bind_unseparated(branch.image.to_bind_value());
        }
        if let Some(is_main) = &branch.is_main {
            separated.push("is_main = ").push_bind_unseparated(is_main);
        }

        separated.push("updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')");

        builder.push(" WHERE id = ").push_bind(id);

        let query = builder.build();
        let result = query.execute(&self.pool).await?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound(format!("Branch with id {} not found", id)));
        }

        Ok(())
    }

    async fn delete(&self, _: &Context, id: i64) -> DomainResult<()> {
        let query = sqlx::query(
            r#"
            UPDATE branches SET
                is_deleted = 1,
                deleted_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ? AND is_deleted = 0
            "#,
        )
        .bind(id)
        .execute(&self.pool);

        let result = query.await?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound(format!("Branch with id {} not found", id)));
        }

        Ok(())
    }

    async fn get_all(&self, _: &Context) -> DomainResult<Vec<Branch>> {
        let query = sqlx::query_as::<_, BranchDbSqlite>(
            r#"
            SELECT * FROM branches WHERE is_deleted = 0
            "#,
        )
        .fetch_all(&self.pool);

        let branches = query.await?;

        Ok(branches.into_iter().map(|b| b.into()).collect())
    }

    async fn get_by_id(&self, _: &Context, id: i64) -> DomainResult<Option<Branch>> {
        let query = sqlx::query_as::<_, BranchDbSqlite>(
            r#"
            SELECT * FROM branches WHERE id = ? AND is_deleted = 0
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool);

        let branch = query.await?;

        Ok(branch.map(|b| b.into()))
    }
}
