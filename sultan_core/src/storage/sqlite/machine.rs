use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, QueryFilter, Set,
};

use super::entity::{MachineActiveModel, MachineColumn, MachineEntity};
use crate::{
    domain::{
        DomainResult,
        error::Error,
        model::machine::{Machine, MachineCreate, MachineFilter, MachineUpdate},
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
            update_query =
                update_query.col_expr(MachineColumn::Key, Expr::value(key.clone()));
        }

        if let Some(name) = &machine.name {
            update_query =
                update_query.col_expr(MachineColumn::Name, Expr::value(name.clone()));
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
            return Err(Error::NotFound(format!(
                "Machine with id {} not found",
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

        let result = MachineEntity::update_many()
            .filter(MachineColumn::Id.eq(id))
            .filter(MachineColumn::IsDeleted.eq(false))
            .col_expr(MachineColumn::IsDeleted, Expr::value(true))
            .col_expr(MachineColumn::DeletedAt, Expr::value(Some(now.clone())))
            .col_expr(MachineColumn::UpdatedAt, Expr::value(now))
            .exec(&ctx.db)
            .await?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!(
                "Machine with id {} not found",
                id
            )));
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
        filter: &MachineFilter,
    ) -> DomainResult<Vec<Machine>> {
        let mut condition = Condition::all().add(MachineColumn::IsDeleted.eq(false));

        if let Some(branch_id) = filter.branch_id {
            condition = condition.add(MachineColumn::BranchId.eq(branch_id));
        }

        if let Some(name) = &filter.name {
            condition = condition.add(MachineColumn::Name.contains(name));
        }

        let models = MachineEntity::find()
            .filter(condition)
            .all(&ctx.db)
            .await?;

        Ok(models.into_iter().map(|m| m.to_domain()).collect())
    }
}
