use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set, sea_query::Expr,
};

use crate::{
    domain::{
        DomainResult, Error,
        model::product::{
            Product, ProductCreate, ProductUpdate, ProductVariant, ProductVariantCreate,
            ProductVariantUpdate,
        },
    },
    storage::{
        ProductRepository, RepoCtx,
        sqlite::entity::{
            ProductActiveModel, ProductCategoryActiveModel, ProductCategoryColumn,
            ProductCategoryEntity, ProductColumn, ProductEntity, ProductVariantActiveModel,
            ProductVariantColumn, ProductVariantEntity,
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

    /// Fetches a product by its ID from the database.
    /// This is a helper method used by variant queries to fetch the associated product.
    async fn fetch_product_by_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<Product>> {
        let product = ProductEntity::find_by_id(id)
            .filter(ProductColumn::IsDeleted.eq(false))
            .one(&ctx.db)
            .await?;

        Ok(product.map(|p| p.to_domain()))
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
            has_variant: Set(product.has_variant),
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

        if let Some(has_variant) = product.has_variant {
            update_query =
                update_query.col_expr(ProductColumn::HasVariant, Expr::value(has_variant));
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
        self.fetch_product_by_id(ctx, id).await
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

        variant_model.insert(&ctx.db).await?;
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
        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.fZ")
            .to_string();

        // Soft delete: mark as deleted with a single UPDATE query
        let result = ProductVariantEntity::update_many()
            .filter(ProductVariantColumn::Id.eq(id))
            .filter(ProductVariantColumn::IsDeleted.eq(false))
            .col_expr(ProductVariantColumn::IsDeleted, Expr::value(true))
            .col_expr(
                ProductVariantColumn::DeletedAt,
                Expr::value(Some(now.clone())),
            )
            .col_expr(ProductVariantColumn::UpdatedAt, Expr::value(now))
            .exec(&ctx.db)
            .await?;

        // Check if any rows were affected
        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!(
                "ProductVariant with id {} not found",
                id
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
            .col_expr(ProductVariantColumn::UpdatedAt, Expr::value(now))
            .exec(&ctx.db)
            .await?;

        Ok(())
    }

    async fn get_variant_by_barcode(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        barcode: &str,
    ) -> DomainResult<Option<ProductVariant>> {
        let variant = ProductVariantEntity::find()
            .filter(ProductVariantColumn::Barcode.eq(barcode))
            .filter(ProductVariantColumn::IsDeleted.eq(false))
            .one(&ctx.db)
            .await?;

        match variant {
            Some(variant_model) => {
                let product = self
                    .fetch_product_by_id(ctx, variant_model.product_id)
                    .await?;
                Ok(product.map(|p| variant_model.to_domain(p)))
            }
            None => Ok(None),
        }
    }

    async fn get_variant_by_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<ProductVariant>> {
        let variant = ProductVariantEntity::find_by_id(id)
            .filter(ProductVariantColumn::IsDeleted.eq(false))
            .one(&ctx.db)
            .await?;

        match variant {
            Some(variant_model) => {
                let product = self
                    .fetch_product_by_id(ctx, variant_model.product_id)
                    .await?;
                Ok(product.map(|p| variant_model.to_domain(p)))
            }
            None => Ok(None),
        }
    }

    async fn get_variant_by_product_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        product_id: i64,
    ) -> DomainResult<Vec<ProductVariant>> {
        let variants = ProductVariantEntity::find()
            .filter(ProductVariantColumn::ProductId.eq(product_id))
            .filter(ProductVariantColumn::IsDeleted.eq(false))
            .all(&ctx.db)
            .await?;

        if variants.is_empty() {
            return Ok(Vec::new());
        }

        let product = self.fetch_product_by_id(ctx, product_id).await?;

        match product {
            Some(product) => Ok(variants
                .into_iter()
                .map(|v| v.to_domain(product.clone()))
                .collect()),
            None => Ok(Vec::new()),
        }
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
}
