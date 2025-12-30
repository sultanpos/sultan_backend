use async_trait::async_trait;
use serde::Serialize;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::{
    domain::{
        Context, DomainResult, Error,
        model::product::{UnitOfMeasure, UnitOfMeasureCreate, UnitOfMeasureUpdate},
    },
    storage::{
        UnitOfMeasureRepository,
        sqlite::{TableName, check_rows_affected, map_results, soft_delete},
    },
};

#[derive(Clone)]
pub struct SqliteUnitOfMeasureRepository {
    pool: SqlitePool,
}

impl SqliteUnitOfMeasureRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

// Database model for UnitOfMeasure - SQLite
#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct UnitOfMeasureDbSqlite {
    pub id: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub is_deleted: bool,
    pub name: String,
    pub description: Option<String>,
}

impl From<UnitOfMeasureDbSqlite> for UnitOfMeasure {
    fn from(db: UnitOfMeasureDbSqlite) -> Self {
        UnitOfMeasure {
            id: db.id,
            created_at: super::parse_sqlite_date(&db.created_at),
            updated_at: super::parse_sqlite_date(&db.updated_at),
            deleted_at: db.deleted_at.map(|d| super::parse_sqlite_date(&d)),
            is_deleted: db.is_deleted,
            name: db.name,
            description: db.description,
        }
    }
}

#[async_trait]
impl UnitOfMeasureRepository for SqliteUnitOfMeasureRepository {
    async fn create(&self, _: &Context, id: i64, uom: &UnitOfMeasureCreate) -> DomainResult<()> {
        let query = sqlx::query(
            r#"
            INSERT INTO units (
                id, name, description
            ) VALUES (?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(&uom.name)
        .bind(&uom.description)
        .execute(&self.pool);

        query.await?;
        Ok(())
    }

    async fn update(&self, _: &Context, id: i64, uom: &UnitOfMeasureUpdate) -> DomainResult<()> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new("UPDATE units SET ");
        let mut separated = builder.separated(", ");

        if let Some(name) = &uom.name {
            separated.push("name = ").push_bind_unseparated(name);
        }

        if uom.description.should_update() {
            separated
                .push("description = ")
                .push_bind_unseparated(uom.description.to_bind_value());
        }

        separated.push("updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')");

        builder.push(" WHERE id = ").push_bind(id);
        builder.push(" AND is_deleted = 0");

        let query = builder.build();
        let result = query.execute(&self.pool).await?;
        check_rows_affected(result.rows_affected(), "Unit of measure", id)?;

        Ok(())
    }

    async fn delete(&self, _: &Context, id: i64) -> DomainResult<()> {
        let query = soft_delete(&self.pool, TableName::Units, id);
        let result = query.await?;
        check_rows_affected(result.rows_affected(), "Unit of measure", id)
    }

    async fn get_all(&self, _: &Context) -> DomainResult<Vec<UnitOfMeasure>> {
        let query = sqlx::query_as::<_, UnitOfMeasureDbSqlite>(
            r#"
            SELECT id, created_at, updated_at, deleted_at, is_deleted, name, description
            FROM units 
            WHERE is_deleted = 0
            "#,
        )
        .fetch_all(&self.pool);
        let units = query.await?;
        Ok(map_results(units))
    }

    async fn get_by_id(&self, _: &Context, id: i64) -> DomainResult<Option<UnitOfMeasure>> {
        let query = sqlx::query_as::<_, UnitOfMeasureDbSqlite>(
            r#"
            SELECT id, created_at, updated_at, deleted_at, is_deleted, name, description
            FROM units 
            WHERE id = ? AND is_deleted = 0
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool);

        Ok(query.await?.map(UnitOfMeasure::from))
    }
}
