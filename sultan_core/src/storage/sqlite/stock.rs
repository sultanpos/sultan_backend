use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};

use crate::{
    domain::{
        DomainResult, Error,
        model::stock::{Stock, StockCreate, StockUpdate},
    },
    storage::{
        RepoCtx,
        sqlite::entity::{StockActiveModel, StockColumn, StockEntity},
        stock_repo::StockRepository,
    },
};

/// SQLite implementation of StockRepository using SeaORM.
///
/// This repository uses SeaORM's `ConnectionTrait` which allows it to work
/// with both direct database connections and transactions seamlessly.
///
/// # Notes
///
/// The stocks table does not use soft delete — records are hard-deleted.
/// There is a unique constraint on `(branch_id, product_variant_id)`.
///
/// # Example
///
/// ```rust,ignore
/// // Using with direct connection
/// let repo = SqliteStockRepository::new();
/// let ctx = RepoCtx { ctx: Context::new(), db: &db_connection };
/// repo.create(&ctx, id, &stock).await?;
///
/// // Using within a transaction
/// let txn = db.begin().await?;
/// let ctx = RepoCtx { ctx: Context::new(), db: &txn };
/// repo.create(&ctx, id, &stock).await?;
/// txn.commit().await?;
/// ```
#[derive(Clone, Default)]
pub struct SqliteStockRepository {}

impl SqliteStockRepository {
    pub fn new() -> Self {
        SqliteStockRepository {}
    }
}

#[async_trait]
impl StockRepository for SqliteStockRepository {
    async fn create(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        stock: &StockCreate,
    ) -> DomainResult<()> {
        let metadata_str = super::serialize_metadata(&stock.metadata);

        let model = StockActiveModel {
            id: Set(id),
            branch_id: Set(stock.branch_id),
            product_variant_id: Set(stock.product_variant_id),
            quantity: Set(stock.quantity),
            min_stock: Set(stock.min_stock),
            max_stock: Set(stock.max_stock),
            last_buy_price: Set(stock.last_buy_price),
            metadata: Set(metadata_str),
            ..Default::default()
        };

        model.insert(&ctx.db).await?;
        Ok(())
    }

    async fn get_by_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<Stock>> {
        let stock = StockEntity::find_by_id(id).one(&ctx.db).await?;

        Ok(stock.map(|s| s.to_domain()))
    }

    async fn get_by_branch_and_variant(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        branch_id: i64,
        product_variant_id: i64,
    ) -> DomainResult<Option<Stock>> {
        let stock = StockEntity::find()
            .filter(StockColumn::BranchId.eq(branch_id))
            .filter(StockColumn::ProductVariantId.eq(product_variant_id))
            .one(&ctx.db)
            .await?;

        Ok(stock.map(|s| s.to_domain()))
    }

    async fn update(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        branch_id: i64,
        product_variant_id: i64,
        stock: &StockUpdate,
    ) -> DomainResult<()> {
        use sea_orm::{UpdateMany, sea_query::Expr};

        let mut update_query: UpdateMany<StockEntity> = StockEntity::update_many()
            .filter(StockColumn::BranchId.eq(branch_id))
            .filter(StockColumn::ProductVariantId.eq(product_variant_id));

        if stock.min_stock.should_update() {
            update_query = update_query.col_expr(
                StockColumn::MinStock,
                Expr::value(stock.min_stock.to_bind_value()),
            );
        }

        if stock.max_stock.should_update() {
            update_query = update_query.col_expr(
                StockColumn::MaxStock,
                Expr::value(stock.max_stock.to_bind_value()),
            );
        }

        if stock.last_buy_price.should_update() {
            update_query = update_query.col_expr(
                StockColumn::LastBuyPrice,
                Expr::value(stock.last_buy_price.to_bind_value()),
            );
        }

        if stock.metadata.should_update() {
            let metadata_json = super::serialize_metadata_update(&stock.metadata);
            update_query = update_query.col_expr(StockColumn::Metadata, Expr::value(metadata_json));
        }

        // Always update the updated_at timestamp
        update_query = update_query.col_expr(
            StockColumn::UpdatedAt,
            Expr::value(Some(
                chrono::Utc::now()
                    .format("%Y-%m-%dT%H:%M:%S%.fZ")
                    .to_string(),
            )),
        );

        let result = update_query.exec(&ctx.db).await?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!(
                "Stock with branch_id {} and product_variant_id {} not found",
                branch_id, product_variant_id
            )));
        }

        Ok(())
    }

    async fn delete(&self, ctx: &RepoCtx<impl ConnectionTrait>, id: i64) -> DomainResult<()> {
        let result = StockEntity::delete_by_id(id).exec(&ctx.db).await?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!("Stock with id {} not found", id)));
        }

        Ok(())
    }
}
