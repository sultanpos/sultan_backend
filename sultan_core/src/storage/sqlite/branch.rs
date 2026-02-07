use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};

use crate::{
    domain::{
        DomainResult, Error,
        model::branch::{Branch, BranchCreate, BranchUpdate},
    },
    storage::{
        RepoCtx,
        branch_repo::BranchRepository,
        sqlite::entity::{BranchActiveModel, BranchColumn, BranchEntity},
    },
};

/// SQLite implementation of BranchRepository using SeaORM.
///
/// This repository uses SeaORM's `ConnectionTrait` which allows it to work
/// with both direct database connections and transactions seamlessly.
///
/// # Example
///
/// ```rust,ignore
/// // Using with direct connection
/// let repo = SqliteBranchRepository::new();
/// let ctx = RepoCtx { ctx: Context::new(), db: &db_connection };
/// repo.create(&ctx, id, &branch).await?;
///
/// // Using within a transaction
/// let txn = db.begin().await?;
/// let ctx = RepoCtx { ctx: Context::new(), db: &txn };
/// repo.create(&ctx, id, &branch).await?;
/// txn.commit().await?;
/// ```
#[derive(Clone, Default)]
pub struct SqliteBranchRepository {}

impl SqliteBranchRepository {
    pub fn new() -> Self {
        SqliteBranchRepository {}
    }
}

#[async_trait]
impl BranchRepository for SqliteBranchRepository {
    async fn create(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        branch: &BranchCreate,
    ) -> DomainResult<()> {
        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.fZ")
            .to_string();

        let branch_model = BranchActiveModel {
            id: Set(id),
            created_at: Set(now.clone()),
            updated_at: Set(now),
            is_main: Set(branch.is_main),
            name: Set(branch.name.clone()),
            code: Set(branch.code.clone()),
            address: Set(branch.address.clone()),
            phone: Set(branch.phone.clone()),
            npwp: Set(branch.npwp.clone()),
            image: Set(branch.image.clone()),
            ..Default::default()
        };

        branch_model.insert(&ctx.db).await?;
        Ok(())
    }

    async fn get_by_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<Branch>> {
        let branch = BranchEntity::find_by_id(id)
            .filter(BranchColumn::IsDeleted.eq(false))
            .one(&ctx.db)
            .await?;

        Ok(branch.map(|b| b.to_domain()))
    }

    async fn update(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        branch: &BranchUpdate,
    ) -> DomainResult<()> {
        use sea_orm::{UpdateMany, sea_query::Expr};

        // Build update query with filters
        let mut update_query: UpdateMany<BranchEntity> = BranchEntity::update_many()
            .filter(BranchColumn::Id.eq(id))
            .filter(BranchColumn::IsDeleted.eq(false));

        // Update fields if provided
        if let Some(is_main) = branch.is_main {
            update_query = update_query.col_expr(BranchColumn::IsMain, Expr::value(is_main));
        }

        if let Some(name) = &branch.name {
            update_query = update_query.col_expr(BranchColumn::Name, Expr::value(name.clone()));
        }

        if let Some(code) = &branch.code {
            update_query = update_query.col_expr(BranchColumn::Code, Expr::value(code.clone()));
        }

        if branch.address.should_update() {
            update_query = update_query.col_expr(
                BranchColumn::Address,
                Expr::value(branch.address.to_bind_value()),
            );
        }

        if branch.phone.should_update() {
            update_query = update_query.col_expr(
                BranchColumn::Phone,
                Expr::value(branch.phone.to_bind_value()),
            );
        }

        if branch.npwp.should_update() {
            update_query =
                update_query.col_expr(BranchColumn::Npwp, Expr::value(branch.npwp.to_bind_value()));
        }

        if branch.image.should_update() {
            update_query = update_query.col_expr(
                BranchColumn::Image,
                Expr::value(branch.image.to_bind_value()),
            );
        }

        // Always update the updated_at timestamp
        update_query = update_query.col_expr(
            BranchColumn::UpdatedAt,
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
            return Err(Error::NotFound(format!("Branch with id {} not found", id)));
        }

        Ok(())
    }

    async fn delete(&self, ctx: &RepoCtx<impl ConnectionTrait>, id: i64) -> DomainResult<()> {
        use sea_orm::sea_query::Expr;

        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.fZ")
            .to_string();

        // Soft delete: mark as deleted with a single UPDATE query
        let result = BranchEntity::update_many()
            .filter(BranchColumn::Id.eq(id))
            .filter(BranchColumn::IsDeleted.eq(false))
            .col_expr(BranchColumn::IsDeleted, Expr::value(true))
            .col_expr(BranchColumn::DeletedAt, Expr::value(Some(now.clone())))
            .col_expr(BranchColumn::UpdatedAt, Expr::value(now))
            .exec(&ctx.db)
            .await?;

        // Check if any rows were affected
        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!("Branch with id {} not found", id)));
        }

        Ok(())
    }

    async fn get_all(&self, ctx: &RepoCtx<impl ConnectionTrait>) -> DomainResult<Vec<Branch>> {
        let branches = BranchEntity::find()
            .filter(BranchColumn::IsDeleted.eq(false))
            .all(&ctx.db)
            .await?;

        Ok(branches.into_iter().map(|b| b.to_domain()).collect())
    }

    async fn set_all_is_main_false(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        except_id: Option<i64>,
    ) -> DomainResult<()> {
        use sea_orm::sea_query::Expr;

        let mut update_query = BranchEntity::update_many()
            .filter(BranchColumn::IsDeleted.eq(false))
            .col_expr(BranchColumn::IsMain, Expr::value(false))
            .col_expr(
                BranchColumn::UpdatedAt,
                Expr::value(
                    chrono::Utc::now()
                        .format("%Y-%m-%dT%H:%M:%S%.fZ")
                        .to_string(),
                ),
            );

        // Exclude the specified branch if provided
        if let Some(id) = except_id {
            update_query = update_query.filter(BranchColumn::Id.ne(id));
        }

        update_query.exec(&ctx.db).await?;
        Ok(())
    }
}
