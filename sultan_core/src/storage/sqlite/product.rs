use async_trait::async_trait;
use serde::Serialize;
use sqlx::{QueryBuilder, Sqlite, SqlitePool, Transaction};

use super::{TableName, check_rows_affected, serialize_metadata, serialize_metadata_update};
use crate::{
    domain::{
        Context, DomainResult,
        model::product::{
            Product, ProductCreate, ProductUpdate, ProductVariant, ProductVariantCreate,
            ProductVariantUpdate,
        },
    },
    storage::{ProductRepository, sqlite::soft_delete},
};

/// SQLite implementation of the ProductRepository.
///
/// This repository manages product and product variant data in a SQLite database,
/// following the Sultan Core architecture patterns including:
/// - Context pattern for authorization and cancellation
/// - Soft delete pattern (never physically delete records)
/// - Transaction support for atomic operations
/// - Metadata storage as JSON
///
/// # Example
///
/// ```rust,ignore
/// use sultan_core::storage::sqlite::product::SqliteProductRepository;
/// use sultan_core::storage::transaction::TransactionManager;
/// use sultan_core::domain::BranchContext;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let pool = todo!();
/// # let tx_manager = todo!();
/// let repo: SqliteProductRepository<BranchContext> = SqliteProductRepository::new(pool);
/// let ctx = BranchContext::new();
///
/// // Create a product within a transaction
/// let mut tx = tx_manager.begin().await?;
/// # let product = todo!();
/// repo.create_product(&ctx, &product, &mut tx).await?;
/// tx_manager.commit(tx).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct SqliteProductRepository {
    pool: SqlitePool,
}

impl SqliteProductRepository {
    /// Creates a new SQLite product repository.
    ///
    /// # Arguments
    ///
    /// * `pool` - SQLite connection pool
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Fetches a product by its ID from the database.
    /// This is a helper method used by variant queries to fetch the associated product.
    async fn fetch_product_by_id(&self, id: i64) -> DomainResult<Option<Product>> {
        let sql = format!("{} WHERE id = ? AND is_deleted = 0", PRODUCT_SELECT_COLUMNS);
        let query = sqlx::query_as::<_, ProductDbSqlite>(&sql).bind(id);
        let product = query.fetch_optional(&self.pool).await?;
        Ok(product.map(Product::from))
    }
}

// Database model for Product - SQLite
#[derive(sqlx::FromRow, Debug, Serialize)]
struct ProductDbSqlite {
    pub id: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub is_deleted: bool,
    pub name: String,
    pub description: Option<String>,
    pub product_type: String,
    pub main_image: Option<String>,
    pub sellable: bool,
    pub buyable: bool,
    pub editable_price: bool,
    pub has_variant: bool,
    pub metadata: Option<String>,
}

impl From<ProductDbSqlite> for Product {
    fn from(db: ProductDbSqlite) -> Self {
        Product {
            id: db.id,
            created_at: super::parse_sqlite_date(&db.created_at),
            updated_at: super::parse_sqlite_date(&db.updated_at),
            deleted_at: db.deleted_at.map(|d| super::parse_sqlite_date(&d)),
            is_deleted: db.is_deleted,
            name: db.name,
            description: db.description,
            product_type: db.product_type,
            main_image: db.main_image,
            sellable: db.sellable,
            buyable: db.buyable,
            editable_price: db.editable_price,
            has_variant: db.has_variant,
            metadata: db.metadata.and_then(|m| serde_json::from_str(&m).ok()),
        }
    }
}

// Database model for ProductVariant - SQLite
#[derive(sqlx::FromRow, Debug, Serialize)]
struct ProductVariantDbSqlite {
    pub id: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub is_deleted: bool,
    pub product_id: i64,
    pub barcode: Option<String>,
    pub name: Option<String>,
    pub metadata: Option<String>,
}

impl ProductVariantDbSqlite {
    /// Converts the database model to a domain ProductVariant with the given Product.
    fn into_variant(self, product: Product) -> ProductVariant {
        ProductVariant {
            id: self.id,
            created_at: super::parse_sqlite_date(&self.created_at),
            updated_at: super::parse_sqlite_date(&self.updated_at),
            deleted_at: self.deleted_at.map(|d| super::parse_sqlite_date(&d)),
            is_deleted: self.is_deleted,
            product,
            barcode: self.barcode,
            name: self.name,
            metadata: self.metadata.and_then(|m| serde_json::from_str(&m).ok()),
        }
    }
}

// SQL query constants to reduce duplication
const PRODUCT_SELECT_COLUMNS: &str = r#"
    SELECT id, created_at, updated_at, deleted_at, is_deleted,
           name, description, product_type, main_image,
           sellable, buyable, editable_price, has_variant, metadata
    FROM products
"#;

const VARIANT_SELECT_COLUMNS: &str = r#"
    SELECT id, created_at, updated_at, deleted_at, is_deleted,
           product_id, barcode, name, metadata
    FROM product_variants
"#;

#[async_trait]
impl<'a> ProductRepository<Transaction<'a, Sqlite>> for SqliteProductRepository {
    async fn create_product(
        &self,
        _: &Context,
        id: i64,
        product: &ProductCreate,
        tx: &mut Transaction<'a, Sqlite>,
    ) -> DomainResult<()> {
        let metadata_json = serialize_metadata(&product.metadata);

        let query = sqlx::query(
            r#"
            INSERT INTO products (
                id, name, description, product_type, main_image,
                sellable, buyable, editable_price, has_variant, metadata
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(&product.name)
        .bind(&product.description)
        .bind(&product.product_type)
        .bind(&product.main_image)
        .bind(product.sellable)
        .bind(product.buyable)
        .bind(product.editable_price)
        .bind(product.has_variant)
        .bind(&metadata_json);

        query.execute(&mut **tx).await?;

        // Insert product categories
        if !product.category_ids.is_empty() {
            let mut builder: QueryBuilder<Sqlite> =
                QueryBuilder::new("INSERT INTO product_categories (product_id, category_id) ");

            builder.push_values(&product.category_ids, |mut b, category_id| {
                b.push_bind(id).push_bind(category_id);
            });

            let query = builder.build();

            query.execute(&mut **tx).await?;
        }

        Ok(())
    }

    async fn update_product(
        &self,
        _: &Context,
        id: i64,
        product: &ProductUpdate,
        tx: &mut Transaction<'a, Sqlite>,
    ) -> DomainResult<()> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new("UPDATE products SET ");
        let mut separated = builder.separated(", ");

        if let Some(name) = &product.name {
            separated.push("name = ").push_bind_unseparated(name);
        }
        if product.description.should_update() {
            separated
                .push("description = ")
                .push_bind_unseparated(product.description.to_bind_value());
        }
        if let Some(product_type) = &product.product_type {
            separated
                .push("product_type = ")
                .push_bind_unseparated(product_type);
        }
        if product.main_image.should_update() {
            separated
                .push("main_image = ")
                .push_bind_unseparated(product.main_image.to_bind_value());
        }
        if let Some(sellable) = product.sellable {
            separated
                .push("sellable = ")
                .push_bind_unseparated(sellable);
        }
        if let Some(buyable) = product.buyable {
            separated.push("buyable = ").push_bind_unseparated(buyable);
        }
        if let Some(editable_price) = product.editable_price {
            separated
                .push("editable_price = ")
                .push_bind_unseparated(editable_price);
        }
        if let Some(has_variant) = product.has_variant {
            separated
                .push("has_variant = ")
                .push_bind_unseparated(has_variant);
        }
        if product.metadata.should_update() {
            let metadata_json = serialize_metadata_update(&product.metadata);
            separated
                .push("metadata = ")
                .push_bind_unseparated(metadata_json);
        }

        separated.push("updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')");
        builder.push(" WHERE id = ").push_bind(id);
        builder.push(" AND is_deleted = 0");

        let query = builder.build();
        let result = query.execute(&mut **tx).await?;

        check_rows_affected(result.rows_affected(), "Product", id)?;

        // Update product categories if provided
        if let Some(category_ids) = &product.category_ids {
            // Delete existing categories
            let delete_query =
                sqlx::query("DELETE FROM product_categories WHERE product_id = ?").bind(id);

            delete_query.execute(&mut **tx).await?;

            // Insert new categories
            if !category_ids.is_empty() {
                let mut builder: QueryBuilder<Sqlite> =
                    QueryBuilder::new("INSERT INTO product_categories (product_id, category_id) ");

                builder.push_values(category_ids, |mut b, category_id| {
                    b.push_bind(id).push_bind(category_id);
                });

                let query = builder.build();

                query.execute(&mut **tx).await?;
            }
        }

        Ok(())
    }

    async fn delete_product(
        &self,
        _: &Context,
        id: i64,
        tx: &mut Transaction<'a, Sqlite>,
    ) -> DomainResult<()> {
        let query = soft_delete(&mut **tx, TableName::Products, id);
        let result = query.await?;
        check_rows_affected(result.rows_affected(), "Product", id)
    }

    async fn get_by_id(&self, _: &Context, id: i64) -> DomainResult<Option<Product>> {
        let sql = format!("{} WHERE id = ? AND is_deleted = 0", PRODUCT_SELECT_COLUMNS);
        let query = sqlx::query_as::<_, ProductDbSqlite>(&sql).bind(id);

        let product = query.fetch_optional(&self.pool).await?;
        Ok(product.map(Product::from))
    }

    async fn create_variant(
        &self,
        _: &Context,
        id: i64,
        variant: &ProductVariantCreate,
        tx: &mut Transaction<'a, Sqlite>,
    ) -> DomainResult<()> {
        let metadata_json = serialize_metadata(&variant.metadata);

        let query = sqlx::query(
            r#"
            INSERT INTO product_variants (
                id, product_id, barcode, name, metadata
            ) VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(variant.product_id)
        .bind(&variant.barcode)
        .bind(&variant.name)
        .bind(&metadata_json);

        query.execute(&mut **tx).await?;
        Ok(())
    }

    async fn update_variant(
        &self,
        _: &Context,
        id: i64,
        variant: &ProductVariantUpdate,
    ) -> DomainResult<()> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new("UPDATE product_variants SET ");
        let mut separated = builder.separated(", ");

        if variant.barcode.should_update() {
            separated
                .push("barcode = ")
                .push_bind_unseparated(variant.barcode.to_bind_value());
        }
        if variant.name.should_update() {
            separated
                .push("name = ")
                .push_bind_unseparated(variant.name.to_bind_value());
        }
        if variant.metadata.should_update() {
            let metadata_json = serialize_metadata_update(&variant.metadata);
            separated
                .push("metadata = ")
                .push_bind_unseparated(metadata_json);
        }

        separated.push("updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')");
        builder.push(" WHERE id = ").push_bind(id);
        builder.push(" AND is_deleted = 0");

        let query = builder.build();
        let result = query.execute(&self.pool).await?;
        check_rows_affected(result.rows_affected(), "ProductVariant", id)
    }

    async fn delete_variant(
        &self,
        _: &Context,
        id: i64,
        tx: &mut Transaction<'a, Sqlite>,
    ) -> DomainResult<()> {
        let query = soft_delete(&mut **tx, TableName::ProductVariants, id);
        let result = query.await?;
        check_rows_affected(result.rows_affected(), "ProductVariant", id)
    }

    async fn delete_variants_by_product_id(
        &self,
        _: &Context,
        product_id: i64,
        tx: &mut Transaction<'a, Sqlite>,
    ) -> DomainResult<()> {
        let query = sqlx::query(
            r#"
            UPDATE product_variants SET
                is_deleted = 1,
                deleted_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE product_id = ? AND is_deleted = 0
            "#,
        )
        .bind(product_id);

        query.execute(&mut **tx).await?;
        Ok(())
    }

    async fn get_variant_by_barcode(
        &self,
        _: &Context,
        barcode: &str,
    ) -> DomainResult<Option<ProductVariant>> {
        let variant_sql = format!(
            "{} WHERE barcode = ? AND is_deleted = 0",
            VARIANT_SELECT_COLUMNS
        );
        let variant_query = sqlx::query_as::<_, ProductVariantDbSqlite>(&variant_sql).bind(barcode);

        let variant_db = variant_query.fetch_optional(&self.pool).await?;

        match variant_db {
            Some(variant_db) => {
                let product = self.fetch_product_by_id(variant_db.product_id).await?;
                Ok(product.map(|p| variant_db.into_variant(p)))
            }
            None => Ok(None),
        }
    }

    async fn get_variant_by_id(
        &self,
        _: &Context,
        id: i64,
    ) -> DomainResult<Option<ProductVariant>> {
        let variant_sql = format!("{} WHERE id = ? AND is_deleted = 0", VARIANT_SELECT_COLUMNS);
        let variant_query = sqlx::query_as::<_, ProductVariantDbSqlite>(&variant_sql).bind(id);

        let variant_db = variant_query.fetch_optional(&self.pool).await?;

        match variant_db {
            Some(variant_db) => {
                let product = self.fetch_product_by_id(variant_db.product_id).await?;
                Ok(product.map(|p| variant_db.into_variant(p)))
            }
            None => Ok(None),
        }
    }

    async fn get_variant_by_product_id(
        &self,
        _: &Context,
        product_id: i64,
    ) -> DomainResult<Vec<ProductVariant>> {
        let variant_sql = format!(
            "{} WHERE product_id = ? AND is_deleted = 0",
            VARIANT_SELECT_COLUMNS
        );
        let variants_query =
            sqlx::query_as::<_, ProductVariantDbSqlite>(&variant_sql).bind(product_id);

        let variants_db = variants_query.fetch_all(&self.pool).await?;

        if variants_db.is_empty() {
            return Ok(Vec::new());
        }

        let product = self.fetch_product_by_id(product_id).await?;

        match product {
            Some(product) => Ok(variants_db
                .into_iter()
                .map(|v| v.into_variant(product.clone()))
                .collect()),
            None => Ok(Vec::new()),
        }
    }

    async fn get_product_category(&self, _: &Context, product_id: i64) -> DomainResult<Vec<i64>> {
        let query = sqlx::query_as::<_, (i64,)>(
            "SELECT category_id FROM product_categories WHERE product_id = ?",
        )
        .bind(product_id);

        let categories = query.fetch_all(&self.pool).await?;

        let category_ids = categories.into_iter().map(|(id,)| id).collect();

        Ok(category_ids)
    }
}
