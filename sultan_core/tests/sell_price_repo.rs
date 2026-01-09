pub mod common;
use serde_json::json;
use sultan_core::domain::model::product::UnitOfMeasureCreate;
use sultan_core::storage::UnitOfMeasureRepository;
use sultan_core::storage::sqlite::transaction::SqliteTransactionManager;
use sultan_core::storage::sqlite::{
    SqliteProductRepository, SqliteSellPriceRepository, SqliteUnitOfMeasureRepository,
};
use sultan_core::testing::storage::sell_price::{
    SellPriceTestData, sell_price_test_repo_integration,
};
use sultan_core::{
    domain::model::product::{ProductCreate, ProductVariantCreate},
    storage::{ProductRepository, transaction::TransactionManager},
};

fn create_test_product() -> ProductCreate {
    ProductCreate {
        name: "Test Product".to_string(),
        description: Some("A test product description".to_string()),
        product_type: "product".to_string(),
        main_image: Some("https://example.com/image.jpg".to_string()),
        sellable: true,
        buyable: true,
        editable_price: false,
        has_variant: false,
        metadata: Some(json!({"key": "value"})),
        category_ids: vec![],
    }
}

fn create_test_variant(product_id: i64, barcode: &str) -> ProductVariantCreate {
    ProductVariantCreate {
        product_id,
        barcode: Some(barcode.to_string()),
        name: Some("Default Variant".to_string()),
        metadata: Some(json!({"sku": "SKU001"})),
    }
}

async fn create_sell_price_test_data() -> SellPriceTestData<'static, SqliteTransactionManager> {
    let pool = common::init_sqlite_pool().await;
    let ctx = sultan_core::domain::Context::new();
    let tx_manager = SqliteTransactionManager::new(pool.clone());
    let sell_price_repo = SqliteSellPriceRepository::new(pool.clone());
    let product_repo = SqliteProductRepository::new(pool.clone());

    let product = create_test_product();
    let unit_repo = SqliteUnitOfMeasureRepository::new(pool.clone());
    let mut tx = tx_manager.begin().await.unwrap();

    let unit_id = sultan_core::testing::storage::generate_test_id().await;
    unit_repo
        .create(
            &ctx,
            unit_id,
            &UnitOfMeasureCreate {
                name: "Piece".to_string(),
                description: Some("Unit for pieces".to_string()),
            },
        )
        .await
        .expect("unable to create unit");

    let product_id = sultan_core::testing::storage::generate_test_id().await;
    product_repo
        .create_product(&ctx, product_id, &product, &mut tx)
        .await
        .expect("Failed to create test product");

    let variant_id = sultan_core::testing::storage::generate_test_id().await;
    let variant_id2 = sultan_core::testing::storage::generate_test_id().await;

    product_repo
        .create_variant(
            &ctx,
            variant_id,
            &create_test_variant(product_id, "123"),
            &mut tx,
        )
        .await
        .expect("Failed to create test variant");
    product_repo
        .create_variant(
            &ctx,
            variant_id2,
            &create_test_variant(product_id, "124"),
            &mut tx,
        )
        .await
        .expect("Failed to create test variant");

    tx_manager.commit(tx).await.unwrap();

    SellPriceTestData {
        ctx,
        product_id: product_id,
        unit_id,
        variant_id: vec![variant_id, variant_id2],
        tx_manager: Box::new(tx_manager),
        sell_price_repo: Box::new(sell_price_repo),
    }
}

// =============================================================================
// Basic CRUD Tests
// =============================================================================

#[tokio::test]
async fn test_create_and_get_sell_price() {
    sell_price_test_repo_integration(&create_sell_price_test_data().await).await;
}

#[tokio::test]
async fn test_delete_sell_price() {
    sultan_core::testing::storage::sell_price::sell_price_test_delete(
        &create_sell_price_test_data().await,
    )
    .await;
}

#[tokio::test]
async fn test_delete_non_existent_sell_price() {
    sultan_core::testing::storage::sell_price::sell_price_test_delete_non_existent(
        &create_sell_price_test_data().await,
    )
    .await;
}

#[tokio::test]
async fn test_get_by_id_not_found() {
    sultan_core::testing::storage::sell_price::sell_price_test_get_by_id_not_found(
        &create_sell_price_test_data().await,
    )
    .await;
}

#[tokio::test]
async fn test_get_all_by_variant_empty() {
    sultan_core::testing::storage::sell_price::sell_price_test_get_all_by_variant_empty(
        &create_sell_price_test_data().await,
    )
    .await;
}

#[tokio::test]
async fn test_get_all_by_variant_multiple() {
    sultan_core::testing::storage::sell_price::sell_price_test_get_all_by_variant_multiple(
        &create_sell_price_test_data().await,
    )
    .await;
}

// =============================================================================
// Update Tests
// =============================================================================

#[tokio::test]
async fn test_update_price_only() {
    sultan_core::testing::storage::sell_price::sell_price_test_update_price_only(
        &create_sell_price_test_data().await,
    )
    .await;
}

#[tokio::test]
async fn test_update_quantity_only() {
    sultan_core::testing::storage::sell_price::sell_price_test_update_quantity_only(
        &create_sell_price_test_data().await,
    )
    .await;
}

#[tokio::test]
async fn test_update_metadata() {
    sultan_core::testing::storage::sell_price::sell_price_test_update_metadata(
        &create_sell_price_test_data().await,
    )
    .await;
}

// =============================================================================
// Multiple Variants Tests
// =============================================================================

#[tokio::test]
async fn test_different_prices_for_different_variants() {
    sultan_core::testing::storage::sell_price::sell_price_test_different_variants(
        &create_sell_price_test_data().await,
    )
    .await;
}

// =============================================================================
// Discount Tests
// =============================================================================

#[tokio::test]
async fn test_create_discount() {
    sultan_core::testing::storage::sell_price::sell_price_test_create_discount(
        &create_sell_price_test_data().await,
    )
    .await;
}

#[tokio::test]
async fn test_update_discount() {
    sultan_core::testing::storage::sell_price::sell_price_test_update_discount(
        &create_sell_price_test_data().await,
    )
    .await;
}

#[tokio::test]
async fn test_delete_discount() {
    sultan_core::testing::storage::sell_price::sell_price_test_delete_discount(
        &create_sell_price_test_data().await,
    )
    .await;
}

#[tokio::test]
async fn test_get_all_discounts_by_price_id() {
    sultan_core::testing::storage::sell_price::sell_price_test_get_all_discounts(
        &create_sell_price_test_data().await,
    )
    .await;
}

#[tokio::test]
async fn test_get_discount_by_id_not_found() {
    sultan_core::testing::storage::sell_price::sell_price_test_get_discount_not_found(
        &create_sell_price_test_data().await,
    )
    .await;
}
