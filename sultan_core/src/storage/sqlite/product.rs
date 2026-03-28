use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, ExprTrait, ModelTrait,
    QueryFilter, QuerySelect, Set, sea_query::Expr,
};

use crate::{
    domain::{
        DomainResult, Error,
        model::product::{
            CursorPage, Product, ProductCreate, ProductUpdate, ProductVariant,
            ProductVariantCreate, ProductVariantUpdate,
        },
        model::sell_price::{
            SellDiscount, SellDiscountCreate, SellDiscountUpdate, SellPrice, SellPriceCreate,
            SellPriceUpdate,
        },
    },
    storage::{
        ProductRepository, RepoCtx,
        sqlite::entity::{
            CategoryColumn, CategoryEntity, ProductActiveModel, ProductCategoryActiveModel,
            ProductCategoryColumn, ProductCategoryEntity, ProductColumn, ProductEntity,
            ProductVariantActiveModel, ProductVariantColumn, ProductVariantEntity,
            SellDiscountActiveModel, SellDiscountColumn, SellDiscountEntity, SellPriceActiveModel,
            SellPriceColumn, SellPriceEntity,
        },
    },
};

/// SQLite implementation of ProductRepository using SeaORM.
///
/// This repository uses SeaORM's `ConnectionTrait` which allows it to work
/// with both direct database connections and transactions seamlessly.
///
/// # Example
///
/// ```rust,ignore
/// // Using with direct connection
/// let repo = SqliteProductRepository::new();
/// let ctx = RepoCtx { ctx: Context::new(), db: &db_connection };
/// repo.create_product(&ctx, id, &product).await?;
///
/// // Using within a transaction
/// let txn = db.begin().await?;
/// let ctx = RepoCtx { ctx: Context::new(), db: &txn };
/// repo.create_product(&ctx, id, &product).await?;
/// repo.create_variant(&ctx, variant_id, &variant).await?;
/// txn.commit().await?;
/// ```
#[derive(Clone, Default)]
pub struct SqliteProductRepository {}

impl SqliteProductRepository {
    pub fn new() -> Self {
        SqliteProductRepository {}
    }
}

#[async_trait]
impl ProductRepository for SqliteProductRepository {
    async fn create_product(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        product: &ProductCreate,
    ) -> DomainResult<()> {
        // Serialize metadata to JSON string
        let metadata_json = product
            .metadata
            .as_ref()
            .map(|m| serde_json::to_string(m).unwrap_or_default());

        let product_model = ProductActiveModel {
            id: Set(id),
            name: Set(product.name.clone()),
            description: Set(product.description.clone()),
            product_type: Set(product.product_type.clone()),
            main_image: Set(product.main_image.clone()),
            sellable: Set(product.sellable),
            buyable: Set(product.buyable),
            editable_price: Set(product.editable_price),
            metadata: Set(metadata_json),
            ..Default::default()
        };

        product_model.insert(&ctx.db).await?;

        // Insert product categories
        for category_id in &product.category_ids {
            let category_model = ProductCategoryActiveModel {
                product_id: Set(id),
                category_id: Set(*category_id),
            };
            category_model.insert(&ctx.db).await?;
        }

        Ok(())
    }

    async fn update_product(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        product: &ProductUpdate,
    ) -> DomainResult<()> {
        use sea_orm::UpdateMany;

        // Build update query with filters
        let mut update_query: UpdateMany<ProductEntity> = ProductEntity::update_many()
            .filter(ProductColumn::Id.eq(id))
            .filter(ProductColumn::IsDeleted.eq(false));

        // Update fields if provided
        if let Some(name) = &product.name {
            update_query = update_query.col_expr(ProductColumn::Name, Expr::value(name.clone()));
        }

        if product.description.should_update() {
            update_query = update_query.col_expr(
                ProductColumn::Description,
                Expr::value(product.description.to_bind_value()),
            );
        }

        if let Some(product_type) = &product.product_type {
            update_query = update_query.col_expr(
                ProductColumn::ProductType,
                Expr::value(product_type.clone()),
            );
        }

        if product.main_image.should_update() {
            update_query = update_query.col_expr(
                ProductColumn::MainImage,
                Expr::value(product.main_image.to_bind_value()),
            );
        }

        if let Some(sellable) = product.sellable {
            update_query = update_query.col_expr(ProductColumn::Sellable, Expr::value(sellable));
        }

        if let Some(buyable) = product.buyable {
            update_query = update_query.col_expr(ProductColumn::Buyable, Expr::value(buyable));
        }

        if let Some(editable_price) = product.editable_price {
            update_query =
                update_query.col_expr(ProductColumn::EditablePrice, Expr::value(editable_price));
        }

        if product.metadata.should_update() {
            let metadata_json = match &product.metadata {
                crate::domain::model::Update::Set(v) => {
                    Some(serde_json::to_string(v).unwrap_or_default())
                }
                crate::domain::model::Update::Clear => None,
                crate::domain::model::Update::Unchanged => None, // Won't be used due to should_update check
            };
            update_query =
                update_query.col_expr(ProductColumn::Metadata, Expr::value(metadata_json));
        }

        // Always update the updated_at timestamp
        update_query = update_query.col_expr(
            ProductColumn::UpdatedAt,
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
            return Err(Error::NotFound(format!("Product with id {} not found", id)));
        }

        // Update product categories if provided
        if let Some(category_ids) = &product.category_ids {
            // Delete existing categories
            ProductCategoryEntity::delete_many()
                .filter(ProductCategoryColumn::ProductId.eq(id))
                .exec(&ctx.db)
                .await?;

            // Insert new categories
            for category_id in category_ids {
                let category_model = ProductCategoryActiveModel {
                    product_id: Set(id),
                    category_id: Set(*category_id),
                };
                category_model.insert(&ctx.db).await?;
            }
        }

        Ok(())
    }

    async fn delete_product(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<()> {
        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.fZ")
            .to_string();

        // Soft delete: mark as deleted with a single UPDATE query
        let result = ProductEntity::update_many()
            .filter(ProductColumn::Id.eq(id))
            .filter(ProductColumn::IsDeleted.eq(false))
            .col_expr(ProductColumn::IsDeleted, Expr::value(true))
            .col_expr(ProductColumn::DeletedAt, Expr::value(Some(now.clone())))
            .col_expr(ProductColumn::UpdatedAt, Expr::value(now))
            .exec(&ctx.db)
            .await?;

        // Check if any rows were affected
        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!("Product with id {} not found", id)));
        }

        Ok(())
    }

    async fn get_by_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<Product>> {
        let product_model = match ProductEntity::find_by_id(id)
            .filter(ProductColumn::IsDeleted.eq(false))
            .one(&ctx.db)
            .await?
        {
            Some(p) => p,
            None => return Ok(None),
        };

        // Fetch categories via Product → ProductCategory → Category join (single query)
        let categories = product_model
            .find_linked(crate::storage::sqlite::entity::product::ProductToCategories)
            .filter(CategoryColumn::IsDeleted.eq(false))
            .all(&ctx.db)
            .await?
            .into_iter()
            .map(|c| c.to_domain())
            .collect();

        // Fetch variants
        let variant_models = ProductVariantEntity::find()
            .filter(ProductVariantColumn::ProductId.eq(id))
            .filter(ProductVariantColumn::IsDeleted.eq(false))
            .all(&ctx.db)
            .await?;

        // Bulk-fetch all sell prices (with discounts) for every variant in one query
        let variant_ids: Vec<i64> = variant_models.iter().map(|v| v.id).collect();

        let mut prices_by_variant: std::collections::HashMap<i64, Vec<_>> =
            std::collections::HashMap::new();

        if !variant_ids.is_empty() {
            let all_prices_with_discounts = SellPriceEntity::find()
                .filter(SellPriceColumn::ProductVariantId.is_in(variant_ids))
                .filter(SellPriceColumn::IsDeleted.eq(false))
                .find_with_related(SellDiscountEntity)
                .all(&ctx.db)
                .await?;

            for (price_model, discount_models) in all_prices_with_discounts {
                let variant_id = price_model.product_variant_id;
                let mut sell_price = price_model.to_domain();
                sell_price.discounts = discount_models
                    .into_iter()
                    .filter(|d| !d.is_deleted)
                    .map(|d| d.to_domain())
                    .collect();
                prices_by_variant
                    .entry(variant_id)
                    .or_default()
                    .push(sell_price);
            }
        }

        let variants: Vec<_> = variant_models
            .into_iter()
            .map(|vm| {
                let sell_prices = prices_by_variant.remove(&vm.id).unwrap_or_default();
                let mut variant = vm.to_domain();
                variant.sell_prices = sell_prices;
                variant
            })
            .collect();

        let mut product = product_model.to_domain();
        product.categories = categories;
        product.variants = variants;

        Ok(Some(product))
    }

    async fn get_all(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        query: &crate::domain::model::product::ProductQuery,
    ) -> DomainResult<CursorPage<Product>> {
        use crate::domain::model::product::{ProductCursor, ProductSortField, SortDirection};
        use sea_orm::{Condition, Order, QueryOrder};

        let mut select = ProductEntity::find().filter(ProductColumn::IsDeleted.eq(false));

        // ── Filters ─────────────────────────────────────────────────────
        if let Some(name) = &query.filter.name {
            select = select.filter(ProductColumn::Name.contains(name));
        }
        if let Some(product_type) = &query.filter.product_type {
            select = select.filter(ProductColumn::ProductType.eq(product_type.clone()));
        }
        if let Some(category_id) = query.filter.category_id {
            select = select.filter(
                ProductColumn::Id.in_subquery(
                    sea_orm::sea_query::Query::select()
                        .column((ProductCategoryEntity, ProductCategoryColumn::ProductId))
                        .from(ProductCategoryEntity)
                        .inner_join(
                            CategoryEntity,
                            sea_orm::sea_query::Expr::col((CategoryEntity, CategoryColumn::Id))
                                .equals((ProductCategoryEntity, ProductCategoryColumn::CategoryId)),
                        )
                        .and_where(
                            sea_orm::sea_query::Expr::col((
                                ProductCategoryEntity,
                                ProductCategoryColumn::CategoryId,
                            ))
                            .eq(category_id),
                        )
                        .and_where(
                            sea_orm::sea_query::Expr::col((
                                CategoryEntity,
                                CategoryColumn::IsDeleted,
                            ))
                            .eq(false),
                        )
                        .to_owned(),
                ),
            );
        }

        // ── Map sort field to column ────────────────────────────────────
        let sort_col = match query.sort_field {
            ProductSortField::Name => ProductColumn::Name,
            ProductSortField::CreatedAt => ProductColumn::CreatedAt,
            ProductSortField::UpdatedAt => ProductColumn::UpdatedAt,
        };

        let order = match query.sort_direction {
            SortDirection::Asc => Order::Asc,
            SortDirection::Desc => Order::Desc,
        };

        // ── Cursor condition ────────────────────────────────────────────
        // WHERE (field > val) OR (field = val AND id > cursor_id)
        // (reversed comparisons for Desc)
        if let Some(cursor) = &query.cursor {
            let cond = match query.sort_direction {
                SortDirection::Asc => Condition::any()
                    .add(Expr::col(sort_col).gt(cursor.field_value.clone()))
                    .add(
                        Condition::all()
                            .add(Expr::col(sort_col).eq(cursor.field_value.clone()))
                            .add(Expr::col(ProductColumn::Id).gt(cursor.id)),
                    ),
                SortDirection::Desc => Condition::any()
                    .add(Expr::col(sort_col).lt(cursor.field_value.clone()))
                    .add(
                        Condition::all()
                            .add(Expr::col(sort_col).eq(cursor.field_value.clone()))
                            .add(Expr::col(ProductColumn::Id).lt(cursor.id)),
                    ),
            };
            select = select.filter(cond);
        }

        // ── Ordering: (sort_field, id) ──────────────────────────────────
        select = select
            .order_by(sort_col, order.clone())
            .order_by(ProductColumn::Id, order);

        // Fetch limit + 1 to detect whether there is a next page
        let fetch_limit = query.limit + 1;
        let rows = select.limit(fetch_limit).all(&ctx.db).await?;

        let has_next = rows.len() as u64 > query.limit;
        let product_models: Vec<_> = rows.into_iter().take(query.limit as usize).collect();

        // ── Build next_cursor from the last item ────────────────────────
        let next_cursor = if has_next {
            product_models.last().map(|last| {
                let field_value = match query.sort_field {
                    ProductSortField::Name => last.name.clone(),
                    ProductSortField::CreatedAt => last.created_at.clone(),
                    ProductSortField::UpdatedAt => last.updated_at.clone(),
                };
                ProductCursor {
                    field_value,
                    id: last.id,
                }
            })
        } else {
            None
        };

        // ── Convert to domain models (lightweight — no relations) ───────
        let items = product_models.into_iter().map(|m| m.to_domain()).collect();

        Ok(CursorPage { items, next_cursor })
    }

    async fn create_variant(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        variant: &ProductVariantCreate,
    ) -> DomainResult<()> {
        // Serialize metadata to JSON string
        let metadata_json = variant
            .metadata
            .as_ref()
            .map(|m| serde_json::to_string(m).unwrap_or_default());

        let variant_model = ProductVariantActiveModel {
            id: Set(id),
            product_id: Set(variant.product_id),
            barcode: Set(variant.barcode.clone()),
            name: Set(variant.name.clone()),
            metadata: Set(metadata_json),
            ..Default::default()
        };

        // Verify the parent product exists and is not deleted before inserting the variant
        let parent_exists = ProductEntity::find_by_id(variant.product_id)
            .filter(ProductColumn::IsDeleted.eq(false))
            .one(&ctx.db)
            .await?;

        if parent_exists.is_none() {
            return Err(Error::NotFound(format!(
                "Product with id {} not found",
                variant.product_id
            )));
        }

        variant_model.insert(&ctx.db).await?;

        // Increment variant_count and update updated_at on the parent product
        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.fZ")
            .to_string();
        let result = ProductEntity::update_many()
            .filter(ProductColumn::Id.eq(variant.product_id))
            .filter(ProductColumn::IsDeleted.eq(false))
            .col_expr(
                ProductColumn::VariantCount,
                Expr::col(ProductColumn::VariantCount).add(1),
            )
            .col_expr(ProductColumn::UpdatedAt, Expr::value(now))
            .exec(&ctx.db)
            .await?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!(
                "Product with id {} not found",
                variant.product_id
            )));
        }

        Ok(())
    }

    async fn update_variant(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        variant: &ProductVariantUpdate,
    ) -> DomainResult<()> {
        use sea_orm::UpdateMany;

        // Build update query with filters
        let mut update_query: UpdateMany<ProductVariantEntity> =
            ProductVariantEntity::update_many()
                .filter(ProductVariantColumn::Id.eq(id))
                .filter(ProductVariantColumn::IsDeleted.eq(false));

        // Update fields if provided
        if variant.barcode.should_update() {
            update_query = update_query.col_expr(
                ProductVariantColumn::Barcode,
                Expr::value(variant.barcode.to_bind_value()),
            );
        }

        if variant.name.should_update() {
            update_query = update_query.col_expr(
                ProductVariantColumn::Name,
                Expr::value(variant.name.to_bind_value()),
            );
        }

        if variant.metadata.should_update() {
            let metadata_json = match &variant.metadata {
                crate::domain::model::Update::Set(v) => {
                    Some(serde_json::to_string(v).unwrap_or_default())
                }
                crate::domain::model::Update::Clear => None,
                crate::domain::model::Update::Unchanged => None,
            };
            update_query =
                update_query.col_expr(ProductVariantColumn::Metadata, Expr::value(metadata_json));
        }

        // Always update the updated_at timestamp
        update_query = update_query.col_expr(
            ProductVariantColumn::UpdatedAt,
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
            return Err(Error::NotFound(format!(
                "ProductVariant with id {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn delete_variant(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<()> {
        // Look up the product_id before deleting
        let variant = ProductVariantEntity::find_by_id(id)
            .filter(ProductVariantColumn::IsDeleted.eq(false))
            .one(&ctx.db)
            .await?;

        let variant = variant
            .ok_or_else(|| Error::NotFound(format!("ProductVariant with id {} not found", id)))?;

        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.fZ")
            .to_string();

        // Soft delete: mark as deleted with a single UPDATE query
        ProductVariantEntity::update_many()
            .filter(ProductVariantColumn::Id.eq(id))
            .filter(ProductVariantColumn::IsDeleted.eq(false))
            .col_expr(ProductVariantColumn::IsDeleted, Expr::value(true))
            .col_expr(
                ProductVariantColumn::DeletedAt,
                Expr::value(Some(now.clone())),
            )
            .col_expr(ProductVariantColumn::UpdatedAt, Expr::value(now.clone()))
            .exec(&ctx.db)
            .await?;

        // Decrement variant_count and update updated_at on the parent product
        let result = ProductEntity::update_many()
            .filter(ProductColumn::Id.eq(variant.product_id))
            .filter(ProductColumn::IsDeleted.eq(false))
            .col_expr(
                ProductColumn::VariantCount,
                Expr::col(ProductColumn::VariantCount).sub(1),
            )
            .col_expr(ProductColumn::UpdatedAt, Expr::value(now))
            .exec(&ctx.db)
            .await?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!(
                "Product with id {} not found",
                variant.product_id
            )));
        }

        Ok(())
    }

    async fn delete_variants_by_product_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        product_id: i64,
    ) -> DomainResult<()> {
        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.fZ")
            .to_string();

        // Soft delete all variants for this product
        ProductVariantEntity::update_many()
            .filter(ProductVariantColumn::ProductId.eq(product_id))
            .filter(ProductVariantColumn::IsDeleted.eq(false))
            .col_expr(ProductVariantColumn::IsDeleted, Expr::value(true))
            .col_expr(
                ProductVariantColumn::DeletedAt,
                Expr::value(Some(now.clone())),
            )
            .col_expr(ProductVariantColumn::UpdatedAt, Expr::value(now.clone()))
            .exec(&ctx.db)
            .await?;

        // Reset variant_count to 0 and update updated_at on the parent product
        let result = ProductEntity::update_many()
            .filter(ProductColumn::Id.eq(product_id))
            .filter(ProductColumn::IsDeleted.eq(false))
            .col_expr(ProductColumn::VariantCount, Expr::value(0i32))
            .col_expr(ProductColumn::UpdatedAt, Expr::value(now))
            .exec(&ctx.db)
            .await?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!(
                "Product with id {} not found",
                product_id
            )));
        }

        Ok(())
    }

    async fn get_variant_by_barcode(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        barcode: &str,
    ) -> DomainResult<Option<ProductVariant>> {
        // Query 1: Variant JOIN Product — fetch variant with parent product
        let result = ProductVariantEntity::find()
            .filter(ProductVariantColumn::Barcode.eq(barcode))
            .filter(ProductVariantColumn::IsDeleted.eq(false))
            .find_also_related(ProductEntity)
            .one(&ctx.db)
            .await?;

        let (variant_model, _) = match result {
            Some((v, Some(p))) if !p.is_deleted => (v, p),
            _ => return Ok(None),
        };

        // Query 2: SellPrice LEFT JOIN SellDiscount — fetch prices with nested discounts
        let prices_with_discounts = SellPriceEntity::find()
            .filter(SellPriceColumn::ProductVariantId.eq(variant_model.id))
            .filter(SellPriceColumn::IsDeleted.eq(false))
            .find_with_related(SellDiscountEntity)
            .all(&ctx.db)
            .await?;

        // Build nested sell_prices with discounts (filter soft-deleted discounts in Rust)
        let sell_prices = prices_with_discounts
            .into_iter()
            .map(|(price_model, discount_models)| {
                let mut sell_price = price_model.to_domain();
                sell_price.discounts = discount_models
                    .into_iter()
                    .filter(|d| !d.is_deleted)
                    .map(|d| d.to_domain())
                    .collect();
                sell_price
            })
            .collect();

        // Assemble the full ProductVariant
        let mut variant = variant_model.to_domain();
        variant.sell_prices = sell_prices;

        Ok(Some(variant))
    }

    async fn get_variant_by_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<ProductVariant>> {
        // Query 1: Variant JOIN Product — fetch variant with parent product
        let result = ProductVariantEntity::find_by_id(id)
            .filter(ProductVariantColumn::IsDeleted.eq(false))
            .find_also_related(ProductEntity)
            .one(&ctx.db)
            .await?;

        let (variant_model, _) = match result {
            Some((v, Some(p))) if !p.is_deleted => (v, p),
            _ => return Ok(None),
        };

        // Query 2: SellPrice LEFT JOIN SellDiscount — fetch prices with nested discounts
        let prices_with_discounts = SellPriceEntity::find()
            .filter(SellPriceColumn::ProductVariantId.eq(id))
            .filter(SellPriceColumn::IsDeleted.eq(false))
            .find_with_related(SellDiscountEntity)
            .all(&ctx.db)
            .await?;

        // Build nested sell_prices with discounts (filter soft-deleted discounts in Rust)
        let sell_prices = prices_with_discounts
            .into_iter()
            .map(|(price_model, discount_models)| {
                let mut sell_price = price_model.to_domain();
                sell_price.discounts = discount_models
                    .into_iter()
                    .filter(|d| !d.is_deleted)
                    .map(|d| d.to_domain())
                    .collect();
                sell_price
            })
            .collect();

        // Assemble the full ProductVariant
        let mut variant = variant_model.to_domain();
        variant.sell_prices = sell_prices;

        Ok(Some(variant))
    }

    async fn get_variant_by_product_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        product_id: i64,
    ) -> DomainResult<Vec<ProductVariant>> {
        // Single query: Variants JOIN Product — fetch all variants with parent product
        let results = ProductVariantEntity::find()
            .filter(ProductVariantColumn::ProductId.eq(product_id))
            .filter(ProductVariantColumn::IsDeleted.eq(false))
            .find_also_related(ProductEntity)
            .all(&ctx.db)
            .await?;

        // Filter out variants whose parent product is deleted and map to domain
        let variants: Vec<ProductVariant> = results
            .into_iter()
            .filter_map(|(variant_model, product_model)| {
                product_model.and_then(|p| {
                    if !p.is_deleted {
                        Some(variant_model.to_domain())
                    } else {
                        None
                    }
                })
            })
            .collect();

        Ok(variants)
    }

    async fn get_product_category(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        product_id: i64,
    ) -> DomainResult<Vec<i64>> {
        let categories = ProductCategoryEntity::find()
            .filter(ProductCategoryColumn::ProductId.eq(product_id))
            .all(&ctx.db)
            .await?;

        Ok(categories.into_iter().map(|c| c.category_id).collect())
    }

    async fn get_variant_ids_by_product_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        product_id: i64,
    ) -> DomainResult<Vec<i64>> {
        let variant_ids: Vec<i64> = ProductVariantEntity::find()
            .select_only()
            .column(ProductVariantColumn::Id)
            .filter(ProductVariantColumn::ProductId.eq(product_id))
            .filter(ProductVariantColumn::IsDeleted.eq(false))
            .into_tuple::<i64>()
            .all(&ctx.db)
            .await?;

        Ok(variant_ids)
    }

    async fn add_product_category(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        product_id: i64,
        category_ids: &[i64],
    ) -> DomainResult<()> {
        if category_ids.is_empty() {
            return Ok(());
        }

        // Insert product categories
        for category_id in category_ids {
            let category_model = ProductCategoryActiveModel {
                product_id: Set(product_id),
                category_id: Set(*category_id),
            };
            // Ignore duplicate key errors (association already exists)
            match category_model.insert(&ctx.db).await {
                Ok(_) => {}
                Err(e) => {
                    // Only ignore unique constraint violations (duplicates)
                    // Foreign key violations and other errors should propagate
                    let err_str = e.to_string();
                    if !err_str.contains("UNIQUE constraint") && !err_str.contains("duplicate") {
                        return Err(e.into());
                    }
                }
            }
        }

        Ok(())
    }

    // =========================================================================
    // SellPrice Methods
    // =========================================================================

    async fn create_sell_price(
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

    async fn update_sell_price(
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

    async fn delete_sell_price(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<()> {
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

    async fn delete_sell_prices_by_product_variant_ids(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        product_variant_ids: &[i64],
    ) -> DomainResult<()> {
        use sea_orm::{QueryTrait, sea_query::Expr};

        if product_variant_ids.is_empty() {
            return Ok(());
        }

        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.fZ")
            .to_string();

        // Build subquery to get sell_price IDs for the given product variant IDs
        let subquery = SellPriceEntity::find()
            .select_only()
            .column(SellPriceColumn::Id)
            .filter(SellPriceColumn::ProductVariantId.is_in(product_variant_ids.to_vec()))
            .filter(SellPriceColumn::IsDeleted.eq(false))
            .into_query();

        // Soft-delete all discounts associated with those sell prices using subquery
        SellDiscountEntity::update_many()
            .filter(SellDiscountColumn::SellPriceId.in_subquery(subquery))
            .filter(SellDiscountColumn::IsDeleted.eq(false))
            .col_expr(SellDiscountColumn::IsDeleted, Expr::value(true))
            .col_expr(
                SellDiscountColumn::DeletedAt,
                Expr::value(Some(now.clone())),
            )
            .col_expr(SellDiscountColumn::UpdatedAt, Expr::value(now.clone()))
            .exec(&ctx.db)
            .await?;

        // Soft-delete all sell prices for the given product variant IDs
        SellPriceEntity::update_many()
            .filter(SellPriceColumn::ProductVariantId.is_in(product_variant_ids.to_vec()))
            .filter(SellPriceColumn::IsDeleted.eq(false))
            .col_expr(SellPriceColumn::IsDeleted, Expr::value(true))
            .col_expr(SellPriceColumn::DeletedAt, Expr::value(Some(now.clone())))
            .col_expr(SellPriceColumn::UpdatedAt, Expr::value(now))
            .exec(&ctx.db)
            .await?;

        Ok(())
    }

    async fn get_all_sell_prices_by_product_variant_id(
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

    async fn get_sell_price_by_id(
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

    async fn create_sell_discount(
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

    async fn update_sell_discount(
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

    async fn delete_sell_discount(
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

    async fn delete_sell_discounts_by_sell_price_id(
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

    async fn get_all_sell_discount_by_price_id(
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

    async fn get_sell_discount_by_id(
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
