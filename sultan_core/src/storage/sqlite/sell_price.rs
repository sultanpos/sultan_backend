use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};

use crate::{
    domain::{
        DomainResult, Error,
        model::sell_price::{
            SellDiscount, SellDiscountCreate, SellDiscountUpdate, SellPrice, SellPriceCreate,
            SellPriceUpdate,
        },
    },
    storage::{
        RepoCtx,
        sell_price_repo::SellPriceRepository,
        sqlite::entity::{
            SellDiscountActiveModel, SellDiscountColumn, SellDiscountEntity, SellPriceActiveModel,
            SellPriceColumn, SellPriceEntity,
        },
    },
};

/// SQLite implementation of SellPriceRepository using SeaORM.
///
/// This repository uses SeaORM's `ConnectionTrait` which allows it to work
/// with both direct database connections and transactions seamlessly.
///
/// # Example
///
/// ```rust,ignore
/// // Using with direct connection
/// let repo = SqliteSellPriceRepository::new();
/// let ctx = RepoCtx { ctx: Context::new(), db: &db_connection };
/// repo.create(&ctx, id, &sell_price).await?;
///
/// // Using within a transaction
/// let txn = db.begin().await?;
/// let ctx = RepoCtx { ctx: Context::new(), db: &txn };
/// repo.create(&ctx, id, &sell_price).await?;
/// txn.commit().await?;
/// ```
#[derive(Clone, Default)]
pub struct SqliteSellPriceRepository {}

impl SqliteSellPriceRepository {
    pub fn new() -> Self {
        SqliteSellPriceRepository {}
    }
}

#[async_trait]
impl SellPriceRepository for SqliteSellPriceRepository {
    // =========================================================================
    // SellPrice Methods
    // =========================================================================

    async fn create(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        price: &SellPriceCreate,
    ) -> DomainResult<()> {
        let metadata_str = price
            .metadata
            .as_ref()
            .map(|m| serde_json::to_string(m).unwrap_or_default());

        let model = SellPriceActiveModel {
            id: Set(id),
            branch_id: Set(price.branch_id),
            product_variant_id: Set(price.product_variant_id),
            uom_id: Set(Some(price.uom_id)),
            quantity: Set(price.quantity),
            price: Set(price.price),
            metadata: Set(metadata_str),
            ..Default::default()
        };

        model.insert(&ctx.db).await?;
        Ok(())
    }

    async fn update(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        price: &SellPriceUpdate,
    ) -> DomainResult<()> {
        use sea_orm::{UpdateMany, sea_query::Expr};

        let mut update_query: UpdateMany<SellPriceEntity> = SellPriceEntity::update_many()
            .filter(SellPriceColumn::Id.eq(id))
            .filter(SellPriceColumn::IsDeleted.eq(false));

        if let Some(uom_id) = price.uom_id {
            update_query = update_query.col_expr(SellPriceColumn::UomId, Expr::value(Some(uom_id)));
        }

        if let Some(quantity) = price.quantity {
            update_query = update_query.col_expr(SellPriceColumn::Quantity, Expr::value(quantity));
        }

        if let Some(p) = price.price {
            update_query = update_query.col_expr(SellPriceColumn::Price, Expr::value(p));
        }

        if price.metadata.should_update() {
            let metadata_json = match &price.metadata {
                crate::domain::model::Update::Set(v) => {
                    Some(serde_json::to_string(v).unwrap_or_default())
                }
                crate::domain::model::Update::Clear => None,
                crate::domain::model::Update::Unchanged => None,
            };
            update_query =
                update_query.col_expr(SellPriceColumn::Metadata, Expr::value(metadata_json));
        }

        // Always update timestamp
        update_query = update_query.col_expr(
            SellPriceColumn::UpdatedAt,
            Expr::value(
                chrono::Utc::now()
                    .format("%Y-%m-%dT%H:%M:%S%.fZ")
                    .to_string(),
            ),
        );

        let result = update_query.exec(&ctx.db).await?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!(
                "SellPrice with id {} not found",
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

        let result = SellPriceEntity::update_many()
            .filter(SellPriceColumn::Id.eq(id))
            .filter(SellPriceColumn::IsDeleted.eq(false))
            .col_expr(SellPriceColumn::IsDeleted, Expr::value(true))
            .col_expr(SellPriceColumn::DeletedAt, Expr::value(Some(now.clone())))
            .col_expr(SellPriceColumn::UpdatedAt, Expr::value(now))
            .exec(&ctx.db)
            .await?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!(
                "SellPrice with id {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn get_all_by_product_variant_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        product_variant_id: i64,
    ) -> DomainResult<Vec<SellPrice>> {
        let prices = SellPriceEntity::find()
            .filter(SellPriceColumn::ProductVariantId.eq(product_variant_id))
            .filter(SellPriceColumn::IsDeleted.eq(false))
            .all(&ctx.db)
            .await?;

        Ok(prices.into_iter().map(|p| p.to_domain()).collect())
    }

    async fn get_by_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<SellPrice>> {
        let price = SellPriceEntity::find_by_id(id)
            .filter(SellPriceColumn::IsDeleted.eq(false))
            .one(&ctx.db)
            .await?;

        Ok(price.map(|p| p.to_domain()))
    }

    // =========================================================================
    // SellDiscount Methods
    // =========================================================================

    async fn create_discount(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        discount: &SellDiscountCreate,
    ) -> DomainResult<()> {
        let metadata_str = discount
            .metadata
            .as_ref()
            .map(|m| serde_json::to_string(m).unwrap_or_default());
        let calculated_price = 0i64; // TODO: Calculate based on formula

        let model = SellDiscountActiveModel {
            id: Set(id),
            sell_price_id: Set(discount.price_id),
            quantity: Set(Some(discount.quantity)),
            discount_formula: Set(discount.discount_formula.clone()),
            calculated_price: Set(calculated_price),
            customer_level: Set(discount.customer_level),
            metadata: Set(metadata_str),
            ..Default::default()
        };

        model.insert(&ctx.db).await?;
        Ok(())
    }

    async fn update_discount(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        discount: &SellDiscountUpdate,
    ) -> DomainResult<()> {
        use sea_orm::{UpdateMany, sea_query::Expr};

        let mut update_query: UpdateMany<SellDiscountEntity> = SellDiscountEntity::update_many()
            .filter(SellDiscountColumn::Id.eq(id))
            .filter(SellDiscountColumn::IsDeleted.eq(false));

        if let Some(quantity) = discount.quantity {
            update_query =
                update_query.col_expr(SellDiscountColumn::Quantity, Expr::value(Some(quantity)));
        }

        if let Some(formula) = &discount.discount_formula {
            update_query = update_query.col_expr(
                SellDiscountColumn::DiscountFormula,
                Expr::value(formula.clone()),
            );
        }

        if discount.customer_level.should_update() {
            let customer_level_value = discount.customer_level.to_bind_value();
            update_query = update_query.col_expr(
                SellDiscountColumn::CustomerLevel,
                Expr::value(customer_level_value),
            );
        }

        if discount.metadata.should_update() {
            let metadata_json = match &discount.metadata {
                crate::domain::model::Update::Set(v) => {
                    Some(serde_json::to_string(v).unwrap_or_default())
                }
                crate::domain::model::Update::Clear => None,
                crate::domain::model::Update::Unchanged => None,
            };
            update_query =
                update_query.col_expr(SellDiscountColumn::Metadata, Expr::value(metadata_json));
        }

        // Always update timestamp
        update_query = update_query.col_expr(
            SellDiscountColumn::UpdatedAt,
            Expr::value(
                chrono::Utc::now()
                    .format("%Y-%m-%dT%H:%M:%S%.fZ")
                    .to_string(),
            ),
        );

        let result = update_query.exec(&ctx.db).await?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!(
                "SellDiscount with id {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn delete_discount(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<()> {
        use sea_orm::sea_query::Expr;

        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.fZ")
            .to_string();

        let result = SellDiscountEntity::update_many()
            .filter(SellDiscountColumn::Id.eq(id))
            .filter(SellDiscountColumn::IsDeleted.eq(false))
            .col_expr(SellDiscountColumn::IsDeleted, Expr::value(true))
            .col_expr(
                SellDiscountColumn::DeletedAt,
                Expr::value(Some(now.clone())),
            )
            .col_expr(SellDiscountColumn::UpdatedAt, Expr::value(now))
            .exec(&ctx.db)
            .await?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!(
                "SellDiscount with id {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn delete_discounts_by_sell_price_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        sell_price_id: i64,
    ) -> DomainResult<()> {
        use sea_orm::sea_query::Expr;

        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.fZ")
            .to_string();

        // Soft delete all discounts for the given sell_price_id
        SellDiscountEntity::update_many()
            .filter(SellDiscountColumn::SellPriceId.eq(sell_price_id))
            .filter(SellDiscountColumn::IsDeleted.eq(false))
            .col_expr(SellDiscountColumn::IsDeleted, Expr::value(true))
            .col_expr(
                SellDiscountColumn::DeletedAt,
                Expr::value(Some(now.clone())),
            )
            .col_expr(SellDiscountColumn::UpdatedAt, Expr::value(now))
            .exec(&ctx.db)
            .await?;

        Ok(())
    }

    async fn get_all_discount_by_price_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        sell_price_id: i64,
    ) -> DomainResult<Vec<SellDiscount>> {
        let discounts = SellDiscountEntity::find()
            .filter(SellDiscountColumn::SellPriceId.eq(sell_price_id))
            .filter(SellDiscountColumn::IsDeleted.eq(false))
            .all(&ctx.db)
            .await?;

        Ok(discounts.into_iter().map(|d| d.to_domain()).collect())
    }

    async fn get_discount_by_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<SellDiscount>> {
        let discount = SellDiscountEntity::find_by_id(id)
            .filter(SellDiscountColumn::IsDeleted.eq(false))
            .one(&ctx.db)
            .await?;

        Ok(discount.map(|d| d.to_domain()))
    }
}
