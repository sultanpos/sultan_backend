use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, ExprTrait, QueryFilter, Set,
    sea_query::Expr,
};

use crate::{
    domain::{DomainResult, model::number::NumberGenerateParams},
    storage::{
        NumberRepository, RepoCtx,
        sqlite::entity::{NumberSequenceActiveModel, NumberSequenceColumn, NumberSequenceEntity},
    },
};

/// SQLite implementation of NumberRepository using SeaORM.
///
/// This repository uses SeaORM's `ConnectionTrait` which allows it to work
/// with both direct database connections and transactions seamlessly.
///
/// The implementation ensures atomic number generation to prevent duplicates
/// under concurrent access by using database-level UPDATE ... RETURNING or
/// INSERT operations.
///
/// # Example
///
/// ```rust,ignore
/// // Using with direct connection
/// let repo = SqliteNumberRepository::new();
/// let ctx = RepoCtx { ctx: Context::new(), db: &db_connection };
/// let next_number = repo.generate_next(&ctx, &params).await?;
///
/// // Using within a transaction
/// let txn = db.begin().await?;
/// let ctx = RepoCtx { ctx: Context::new(), db: &txn };
/// let next_number = repo.generate_next(&ctx, &params).await?;
/// txn.commit().await?;
/// ```
#[derive(Clone, Default)]
pub struct SqliteNumberRepository {}

impl SqliteNumberRepository {
    pub fn new() -> Self {
        SqliteNumberRepository {}
    }
}

#[async_trait]
impl NumberRepository for SqliteNumberRepository {
    async fn generate_next(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        params: &NumberGenerateParams,
    ) -> DomainResult<i32> {
        // Try to increment existing sequence and get the new value
        let update_result = NumberSequenceEntity::update_many()
            .filter(NumberSequenceColumn::Prefix.eq(&params.prefix))
            .filter(NumberSequenceColumn::Year.eq(params.year))
            .filter(match params.branch_id {
                Some(id) => NumberSequenceColumn::BranchId.eq(Some(id)),
                None => NumberSequenceColumn::BranchId.is_null(),
            })
            .filter(match params.month {
                Some(m) => NumberSequenceColumn::Month.eq(Some(m)),
                None => NumberSequenceColumn::Month.is_null(),
            })
            .col_expr(
                NumberSequenceColumn::LastNumber,
                Expr::col(NumberSequenceColumn::LastNumber).add(1),
            )
            .col_expr(
                NumberSequenceColumn::UpdatedAt,
                Expr::value(
                    chrono::Utc::now()
                        .format("%Y-%m-%dT%H:%M:%S%.fZ")
                        .to_string(),
                ),
            )
            .exec(&ctx.db)
            .await?;

        if update_result.rows_affected > 0 {
            // Sequence was updated, fetch the new value
            let sequence = self.get_sequence(ctx, params).await?;
            Ok(sequence.map(|s| s.last_number).unwrap_or(1))
        } else {
            // Sequence doesn't exist, create it with initial value
            let initial_number = 1;
            let id = crate::snowflake::SnowflakeGenerator::new(1)?.generate()?;

            let new_sequence = NumberSequenceActiveModel {
                id: Set(id),
                prefix: Set(params.prefix.clone()),
                branch_id: Set(params.branch_id),
                year: Set(params.year),
                month: Set(params.month),
                last_number: Set(initial_number),
                ..Default::default()
            };

            new_sequence.insert(&ctx.db).await?;
            Ok(initial_number)
        }
    }

    async fn get_sequence(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        params: &NumberGenerateParams,
    ) -> DomainResult<Option<crate::domain::model::number::NumberSequence>> {
        let sequence = NumberSequenceEntity::find()
            .filter(NumberSequenceColumn::Prefix.eq(&params.prefix))
            .filter(NumberSequenceColumn::Year.eq(params.year))
            .filter(match params.branch_id {
                Some(id) => NumberSequenceColumn::BranchId.eq(Some(id)),
                None => NumberSequenceColumn::BranchId.is_null(),
            })
            .filter(match params.month {
                Some(m) => NumberSequenceColumn::Month.eq(Some(m)),
                None => NumberSequenceColumn::Month.is_null(),
            })
            .one(&ctx.db)
            .await?;

        Ok(sequence.map(|s| s.to_domain()))
    }
}
