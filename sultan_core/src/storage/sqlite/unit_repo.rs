use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};

use crate::{
    domain::{
        DomainResult, Error,
        model::product::{UnitOfMeasure, UnitOfMeasureCreate, UnitOfMeasureUpdate},
    },
    storage::{
        sqlite::entity::{UnitActiveModel, UnitColumn, UnitEntity},
        unit_repo::{RepoCtx, UnitOfMeasureRepository},
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
        // First, check if the unit exists and is not deleted
        let existing = UnitEntity::find_by_id(id)
            .filter(UnitColumn::IsDeleted.eq(false))
            .one(&ctx.db)
            .await?;

        let existing = existing.ok_or(Error::NotFound(format!("Unit with id {} not found", id)))?;

        let mut unit: UnitActiveModel = existing.into();

        // Update fields if provided
        if let Some(name) = &uom.name {
            unit.name = Set(name.clone());
        }

        if uom.description.should_update() {
            unit.description = Set(uom.description.to_bind_value());
        }

        // Update the updated_at timestamp
        unit.updated_at = Set(chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.fZ")
            .to_string());

        unit.update(&ctx.db).await?;
        Ok(())
    }

    async fn delete(&self, ctx: &RepoCtx<impl ConnectionTrait>, id: i64) -> DomainResult<()> {
        // Soft delete: mark as deleted instead of physically removing
        let existing = UnitEntity::find_by_id(id)
            .filter(UnitColumn::IsDeleted.eq(false))
            .one(&ctx.db)
            .await?;

        let existing = existing.ok_or(Error::NotFound(format!("Unit with id {} not found", id)))?;

        let mut unit: UnitActiveModel = existing.into();
        unit.is_deleted = Set(true);
        unit.deleted_at = Set(Some(
            chrono::Utc::now()
                .format("%Y-%m-%dT%H:%M:%S%.fZ")
                .to_string(),
        ));
        unit.updated_at = Set(chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.fZ")
            .to_string());

        unit.update(&ctx.db).await?;
        Ok(())
    }

    async fn list(&self, ctx: &RepoCtx<impl ConnectionTrait>) -> DomainResult<Vec<UnitOfMeasure>> {
        let units = UnitEntity::find()
            .filter(UnitColumn::IsDeleted.eq(false))
            .all(&ctx.db)
            .await?;

        Ok(units.into_iter().map(|u| u.to_domain()).collect())
    }
}
