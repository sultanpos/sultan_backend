use async_trait::async_trait;
use serde::Serialize;
use sqlx::{QueryBuilder, Sqlite, SqlitePool, Transaction};

use super::{TableName, check_rows_affected, serialize_metadata, serialize_metadata_update};
use crate::{
    domain::{
        Context, DomainResult,
        model::sell_price::{
            SellDiscount, SellDiscountCreate, SellDiscountUpdate, SellPrice, SellPriceCreate,
            SellPriceUpdate,
        },
    },
    storage::{sell_price_repo::SellPriceRepository, sqlite::soft_delete},
};

#[derive(sqlx::FromRow, Debug, Serialize)]
struct SellPriceDbSqlite {
    pub id: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub is_deleted: bool,
    pub branch_id: Option<i64>,
    pub product_variant_id: i64,
    pub uom_id: i64,
    pub quantity: i64,
    pub price: i64,
    pub metadata: Option<String>,
}

impl From<SellPriceDbSqlite> for SellPrice {
    fn from(db: SellPriceDbSqlite) -> Self {
        SellPrice {
            id: db.id,
            created_at: super::parse_sqlite_date(&db.created_at),
            updated_at: super::parse_sqlite_date(&db.updated_at),
            deleted_at: db.deleted_at.map(|d| super::parse_sqlite_date(&d)),
            is_deleted: db.is_deleted,
            branch_id: db.branch_id,
            product_variant_id: db.product_variant_id,
            uom_id: db.uom_id,
            quantity: db.quantity,
            price: db.price,
            metadata: db.metadata.and_then(|m| serde_json::from_str(&m).ok()),
        }
    }
}

#[derive(sqlx::FromRow, Debug, Serialize)]
struct SellDiscountDbSqlite {
    pub id: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub is_deleted: bool,
    pub price_id: i64,
    pub quantity: i64,
    pub discount_formula: Option<String>,
    pub calculated_price: i64,
    pub customer_level: Option<i64>,
    pub metadata: Option<String>,
}

impl From<SellDiscountDbSqlite> for SellDiscount {
    fn from(db: SellDiscountDbSqlite) -> Self {
        SellDiscount {
            id: db.id,
            created_at: super::parse_sqlite_date(&db.created_at),
            updated_at: super::parse_sqlite_date(&db.updated_at),
            deleted_at: db.deleted_at.map(|d| super::parse_sqlite_date(&d)),
            is_deleted: db.is_deleted,
            price_id: db.price_id,
            quantity: db.quantity,
            discount_formula: db.discount_formula,
            calculated_price: db.calculated_price,
            customer_level: db.customer_level,
            metadata: db.metadata.and_then(|m| serde_json::from_str(&m).ok()),
        }
    }
}

#[derive(Clone)]
pub struct SqliteSellPriceRepository {
    pool: SqlitePool,
}

impl SqliteSellPriceRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl SqliteSellPriceRepository {
    async fn create_impl<'e, E>(
        &self,
        id: i64,
        price: &SellPriceCreate,
        executor: E,
    ) -> DomainResult<()>
    where
        E: sqlx::Executor<'e, Database = Sqlite>,
    {
        let query = r#"
            INSERT INTO sell_prices (id, branch_id, product_variant_id, uom_id, quantity, price, metadata)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        "#;
        let metadata_str = serialize_metadata(&price.metadata);
        sqlx::query(query)
            .bind(id)
            .bind(price.branch_id)
            .bind(price.product_variant_id)
            .bind(price.uom_id)
            .bind(price.quantity)
            .bind(price.price)
            .bind(metadata_str)
            .execute(executor)
            .await?;
        Ok(())
    }

    async fn update_impl<'e, E>(
        &self,
        id: i64,
        sell_price: &SellPriceUpdate,
        executor: E,
    ) -> DomainResult<()>
    where
        E: sqlx::Executor<'e, Database = Sqlite>,
    {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new("UPDATE sell_prices SET ");
        let mut separated = builder.separated(", ");

        if let Some(uom_id) = &sell_price.uom_id {
            separated.push("uom_id = ").push_bind_unseparated(uom_id);
        }
        if let Some(quantity) = &sell_price.quantity {
            separated
                .push("quantity = ")
                .push_bind_unseparated(quantity);
        }
        if let Some(price) = &sell_price.price {
            separated.push("price = ").push_bind_unseparated(price);
        }
        if sell_price.metadata.should_update() {
            let metadata_json = serialize_metadata_update(&sell_price.metadata);
            separated
                .push("metadata = ")
                .push_bind_unseparated(metadata_json);
        }

        separated.push("updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')");
        builder.push(" WHERE id = ").push_bind(id);
        builder.push(" AND is_deleted = 0");

        let query = builder.build();
        let result = query.execute(executor).await?;

        check_rows_affected(result.rows_affected(), "SellPrice", id)?;
        Ok(())
    }

    async fn create_discount_impl<'e, E>(
        &self,
        id: i64,
        price: &SellDiscountCreate,
        executor: E,
    ) -> DomainResult<()>
    where
        E: sqlx::Executor<'e, Database = Sqlite>,
    {
        let query = r#"
            INSERT INTO sell_discounts (id, sell_price_id, quantity, discount_formula, calculated_price, customer_level, metadata)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        "#;
        let metadata_str = serialize_metadata(&price.metadata);
        let calculated_price = 0; // TODO: Calculate based on formula
        sqlx::query(query)
            .bind(id)
            .bind(price.price_id)
            .bind(price.quantity)
            .bind(&price.discount_formula)
            .bind(calculated_price)
            .bind(price.customer_level)
            .bind(metadata_str)
            .execute(executor)
            .await?;
        Ok(())
    }

    async fn update_discount_impl<'e, E>(
        &self,
        id: i64,
        sell_discount: &SellDiscountUpdate,
        executor: E,
    ) -> DomainResult<()>
    where
        E: sqlx::Executor<'e, Database = Sqlite>,
    {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new("UPDATE sell_discounts SET ");
        let mut separated = builder.separated(", ");

        if let Some(quantity) = &sell_discount.quantity {
            separated
                .push("quantity = ")
                .push_bind_unseparated(quantity);
        }
        if let Some(discount_formula) = &sell_discount.discount_formula {
            separated
                .push("discount_formula = ")
                .push_bind_unseparated(discount_formula);
        }
        if sell_discount.customer_level.should_update() {
            let customer_level_value = sell_discount.customer_level.to_bind_value();
            separated
                .push("customer_level = ")
                .push_bind_unseparated(customer_level_value);
        }

        if sell_discount.metadata.should_update() {
            let metadata_json = serialize_metadata_update(&sell_discount.metadata);
            separated
                .push("metadata = ")
                .push_bind_unseparated(metadata_json);
        }

        separated.push("updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')");
        builder.push(" WHERE id = ").push_bind(id);
        builder.push(" AND is_deleted = 0");

        let query = builder.build();
        let result = query.execute(executor).await?;

        check_rows_affected(result.rows_affected(), "SellDiscount", id)?;
        Ok(())
    }
}

#[async_trait]
impl<'a> SellPriceRepository<Transaction<'a, Sqlite>> for SqliteSellPriceRepository {
    async fn create(&self, _: &Context, id: i64, price: &SellPriceCreate) -> DomainResult<()> {
        self.create_impl(id, price, &self.pool).await
    }
    async fn create_tx(
        &self,
        _: &Context,
        id: i64,
        price: &SellPriceCreate,
        tx: &mut Transaction<'a, Sqlite>,
    ) -> DomainResult<()> {
        self.create_impl(id, price, &mut **tx).await
    }
    async fn update(&self, _: &Context, id: i64, sell_price: &SellPriceUpdate) -> DomainResult<()> {
        self.update_impl(id, sell_price, &self.pool).await
    }
    async fn update_tx(
        &self,
        _: &Context,
        id: i64,
        sell_price: &SellPriceUpdate,
        tx: &mut Transaction<'a, Sqlite>,
    ) -> DomainResult<()> {
        self.update_impl(id, sell_price, &mut **tx).await
    }
    async fn delete(&self, _: &Context, id: i64) -> DomainResult<()> {
        let result = soft_delete(&self.pool, TableName::SellPrices, id).await?;
        check_rows_affected(result.rows_affected(), "Product", id)
    }
    async fn delete_tx(
        &self,
        _: &Context,
        id: i64,
        tx: &mut Transaction<'a, Sqlite>,
    ) -> DomainResult<()> {
        let result = soft_delete(&mut **tx, TableName::SellPrices, id).await?;
        check_rows_affected(result.rows_affected(), "Product", id)
    }
    async fn get_all_by_product_variant_id(
        &self,
        _: &Context,
        id: i64,
    ) -> DomainResult<Vec<SellPrice>> {
        let query = r#"
            SELECT id, created_at, updated_at, deleted_at, is_deleted, branch_id, product_variant_id, uom_id, quantity, price, metadata
            FROM sell_prices
            WHERE product_variant_id = ? AND is_deleted = 0
        "#;
        let rows: Vec<SellPriceDbSqlite> =
            sqlx::query_as(query).bind(id).fetch_all(&self.pool).await?;
        Ok(super::map_results(rows))
    }
    async fn get_by_id(&self, _: &Context, id: i64) -> DomainResult<Option<SellPrice>> {
        let query = r#"
            SELECT id, created_at, updated_at, deleted_at, is_deleted, branch_id, product_variant_id, uom_id, quantity, price, metadata
            FROM sell_prices
            WHERE id = ? AND is_deleted = 0
        "#;
        let row: Option<SellPriceDbSqlite> = sqlx::query_as(query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| r.into()))
    }

    async fn create_discount(
        &self,
        _: &Context,
        id: i64,
        price: &SellDiscountCreate,
    ) -> DomainResult<()> {
        self.create_discount_impl(id, price, &self.pool).await
    }
    async fn create_discount_tx(
        &self,
        _: &Context,
        id: i64,
        price: &SellDiscountCreate,
        tx: &mut Transaction<'a, Sqlite>,
    ) -> DomainResult<()> {
        self.create_discount_impl(id, price, &mut **tx).await
    }
    async fn update_discount(
        &self,
        _: &Context,
        id: i64,
        sell_discount: &SellDiscountUpdate,
    ) -> DomainResult<()> {
        self.update_discount_impl(id, sell_discount, &self.pool)
            .await
    }
    async fn update_discount_tx(
        &self,
        _: &Context,
        id: i64,
        sell_discount: &SellDiscountUpdate,
        tx: &mut Transaction<'a, Sqlite>,
    ) -> DomainResult<()> {
        self.update_discount_impl(id, sell_discount, &mut **tx)
            .await
    }
    async fn delete_discount(&self, _: &Context, id: i64) -> DomainResult<()> {
        let result = soft_delete(&self.pool, TableName::SellDiscounts, id).await?;
        check_rows_affected(result.rows_affected(), "SellDiscount", id)
    }
    async fn delete_discount_by_sell_price_id_tx(
        &self,
        _: &Context,
        sell_price_id: i64,
        tx: &mut Transaction<'a, Sqlite>,
    ) -> DomainResult<()> {
        let sql = format!(
            r#"
        UPDATE sell_discounts SET
            is_deleted = 1,
            deleted_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
        WHERE sell_price_id = ? AND is_deleted = 0
        "#,
        );
        sqlx::query(&sql)
            .bind(sell_price_id)
            .execute(&mut **tx)
            .await?;
        let result = soft_delete(&mut **tx, TableName::SellDiscounts, sell_price_id).await?;
        check_rows_affected(result.rows_affected(), "SellDiscount", sell_price_id)
    }
    async fn get_all_discount_by_price_id(
        &self,
        _: &Context,
        id: i64,
    ) -> DomainResult<Vec<SellDiscount>> {
        let query = r#"
            SELECT id, created_at, updated_at, deleted_at, is_deleted, sell_price_id as price_id, quantity, discount_formula, calculated_price, customer_level, metadata
            FROM sell_discounts
            WHERE sell_price_id = ? AND is_deleted = 0
        "#;
        let rows: Vec<SellDiscountDbSqlite> =
            sqlx::query_as(query).bind(id).fetch_all(&self.pool).await?;
        Ok(super::map_results(rows))
    }
    async fn get_discount_by_id(&self, _: &Context, id: i64) -> DomainResult<Option<SellDiscount>> {
        let query = r#"
            SELECT id, created_at, updated_at, deleted_at, is_deleted, sell_price_id as price_id, quantity, discount_formula, calculated_price, customer_level, metadata
            FROM sell_discounts
            WHERE id = ? AND is_deleted = 0
        "#;
        let row: Option<SellDiscountDbSqlite> = sqlx::query_as(query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| r.into()))
    }
}
