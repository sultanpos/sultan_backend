mod common;

use common::init_sqlite_pool;
use serde_json::json;
use sultan_core::domain::Context;
use sultan_core::domain::model::Update;
use sultan_core::domain::model::product::{
    ProductCreate, ProductVariantCreate, ProductVariantUpdate,
};
use sultan_core::snowflake::SnowflakeGenerator;
use sultan_core::storage::ProductRepository;
use sultan_core::storage::sqlite::product::SqliteProductRepository;
use sultan_core::storage::sqlite::transaction::SqliteTransactionManager;
use sultan_core::storage::transaction::TransactionManager;

use crate::common::product_share::create_sqlite_product_repo;

fn generate_test_id() -> i64 {
    thread_local! {
        static GENERATOR: SnowflakeGenerator = SnowflakeGenerator::new(1).unwrap();
    }
    GENERATOR.with(|g| g.generate().unwrap())
}

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

fn create_test_variant(product_id: i64) -> ProductVariantCreate {
    ProductVariantCreate {
        product_id,
        barcode: Some("1234567890".to_string()),
        name: Some("Default Variant".to_string()),
        metadata: Some(json!({"sku": "SKU001"})),
    }
}

// =============================================================================
// Product CRUD Tests
// =============================================================================

#[tokio::test]
async fn test_create_product_success() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::product_test_create_success(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_create_product_without_optional_fields() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_create_product_without_optional_fields(&ctx, &tx_manager, &repo)
        .await;
}

#[tokio::test]
async fn test_create_product_with_categories() {
    let (ctx, tx_manager, repo, pool) = create_sqlite_product_repo().await;
    common::product_share::test_create_product_with_categories(&ctx, &tx_manager, &repo, &pool)
        .await;
}

#[tokio::test]
async fn test_update_product_name() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_create_product_without_optional_fields(&ctx, &tx_manager, &repo)
        .await;
    common::product_share::test_update_product_name(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_product_clear_description() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_create_product_without_optional_fields(&ctx, &tx_manager, &repo)
        .await;
    common::product_share::test_update_product_clear_description(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_product_all_fields() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_create_product_without_optional_fields(&ctx, &tx_manager, &repo)
        .await;
    common::product_share::test_update_product_all_fields(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_product_categories() {
    let (ctx, tx_manager, repo, pool) = create_sqlite_product_repo().await;
    common::product_share::test_create_product_without_optional_fields(&ctx, &tx_manager, &repo)
        .await;
    common::product_share::test_update_product_categories(&ctx, &tx_manager, &repo, &pool).await;
}

#[tokio::test]
async fn test_update_product_not_found() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_update_product_not_found(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_delete_product_success() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_delete_product_success(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_delete_product_not_found() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_delete_product_not_found(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_get_product_by_id_not_found() {
    let (ctx, _, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_get_product_by_id_not_found(&ctx, &repo).await;
}

// =============================================================================
// Product Variant CRUD Tests
// =============================================================================

#[tokio::test]
async fn test_create_variant_success() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_create_variant_success(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_create_variant_without_optional_fields() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_create_variant_without_optional_fields(&ctx, &tx_manager, &repo)
        .await;
}

#[tokio::test]
async fn test_update_variant_barcode() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_update_variant_barcode(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_variant_clear_name() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_update_variant_clear_name(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_variant_all_fields() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_update_variant_all_fields(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_variant_not_found() {
    let (ctx, _, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_update_variant_not_found(&ctx, &repo).await;
}

#[tokio::test]
async fn test_delete_variant_success() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_delete_variant_success(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_delete_variant_not_found() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_delete_variant_not_found(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_delete_variants_by_product_id() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_delete_variants_by_product_id(&ctx, &tx_manager, &repo).await;
}

// =============================================================================
// Variant Query Tests
// =============================================================================

#[tokio::test]
async fn test_get_variant_by_barcode_success() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_get_variant_by_barcode_success(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_get_variant_by_barcode_not_found() {
    let (ctx, _, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_get_variant_by_barcode_not_found(&ctx, &repo).await;
}

#[tokio::test]
async fn test_get_variant_by_id_not_found() {
    let (ctx, _, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_get_variant_by_id_not_found(&ctx, &repo).await;
}

#[tokio::test]
async fn test_get_variant_by_product_id_success() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_get_variant_by_product_id_success(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_get_variant_by_product_id_empty() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_get_variant_by_product_id_empty(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_get_variant_by_product_id_product_not_found() {
    let (ctx, _, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_get_variant_by_product_id_product_not_found(&ctx, &repo).await;
}

// =============================================================================
// Transaction Tests
// =============================================================================

#[tokio::test]
async fn test_transaction_rollback_product_creation() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_transaction_rollback_product_creation(&ctx, &tx_manager, &repo)
        .await;
}

#[tokio::test]
async fn test_transaction_rollback_variant_creation() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_transaction_rollback_variant_creation(&ctx, &tx_manager, &repo)
        .await;
}

#[tokio::test]
async fn test_transaction_product_and_variant_atomic() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_transaction_product_and_variant_atomic(&ctx, &tx_manager, &repo)
        .await;
}

// =============================================================================
// Soft Delete Verification Tests
// =============================================================================

#[tokio::test]
async fn test_soft_delete_product_preserves_data() {
    let (ctx, tx_manager, repo, pool) = create_sqlite_product_repo().await;
    common::product_share::test_soft_delete_product_preserves_data(&ctx, &tx_manager, &repo, &pool)
        .await;
}

#[tokio::test]
async fn test_soft_delete_variant_preserves_data() {
    let (ctx, tx_manager, repo, pool) = create_sqlite_product_repo().await;
    common::product_share::test_soft_delete_variant_preserves_data(&ctx, &tx_manager, &repo, &pool)
        .await;
}

// =============================================================================
// Edge Cases
// =============================================================================

#[tokio::test]
async fn test_product_with_metadata_json() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_product_with_metadata_json(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_deleted_product_fails() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_update_deleted_product_fails(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_deleted_variant_fails() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_update_deleted_variant_fails(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_delete_already_deleted_product_fails() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_delete_already_deleted_product_fails(&ctx, &tx_manager, &repo)
        .await;
}

// =============================================================================
// Additional Coverage Tests
// =============================================================================

#[tokio::test]
async fn test_get_variant_by_id_success() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_get_variant_by_id_success(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_get_variant_by_id_when_product_deleted() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_get_variant_by_id_when_product_deleted(&ctx, &tx_manager, &repo)
        .await;
}

#[tokio::test]
async fn test_get_variant_by_barcode_when_product_deleted() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_get_variant_by_barcode_when_product_deleted(
        &ctx,
        &tx_manager,
        &repo,
    )
    .await;
}

#[tokio::test]
async fn test_update_product_with_empty_category_update() {
    let (ctx, tx_manager, repo, pool) = common::product_share::create_sqlite_product_repo().await;
    common::product_share::test_update_product_with_empty_category_update(
        &ctx,
        &tx_manager,
        &repo,
        &pool,
    )
    .await;
}

#[tokio::test]
async fn test_update_product_replace_categories() {
    let (ctx, tx_manager, repo, pool) = common::product_share::create_sqlite_product_repo().await;
    common::product_share::test_update_product_replace_categories(&ctx, &tx_manager, &repo, &pool)
        .await;
}

#[tokio::test]
async fn test_update_product_only_metadata() {
    let (ctx, tx_manager, repo, _pool) = common::product_share::create_sqlite_product_repo().await;
    common::product_share::test_update_product_only_metadata(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_product_clear_metadata() {
    let (ctx, tx_manager, repo, _pool) = common::product_share::create_sqlite_product_repo().await;
    common::product_share::test_update_product_clear_metadata(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_variant_clear_metadata() {
    let (ctx, tx_manager, repo, _pool) = common::product_share::create_sqlite_product_repo().await;
    common::product_share::test_update_variant_clear_metadata(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_variant_clear_barcode() {
    let (ctx, tx_manager, repo, _pool) = create_sqlite_product_repo().await;
    common::product_share::test_update_variant_clear_barcode(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_multiple_variants_for_single_product() {
    let (ctx, tx_manager, repo, _pool) = create_sqlite_product_repo().await;
    common::product_share::test_multiple_variants_for_single_product(&ctx, &tx_manager, &repo)
        .await;
}

#[tokio::test]
async fn test_delete_variants_by_product_id_preserves_other_products() {
    let (ctx, tx_manager, repo, _pool) = create_sqlite_product_repo().await;
    common::product_share::test_delete_variants_by_product_id_preserves_other_products(
        &ctx,
        &tx_manager,
        &repo,
    )
    .await;
}

#[tokio::test]
async fn test_update_product_boolean_flags() {
    let (ctx, tx_manager, repo, _pool) = create_sqlite_product_repo().await;
    common::product_share::test_update_product_boolean_flags(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_product_main_image() {
    let (ctx, tx_manager, repo, _pool) = create_sqlite_product_repo().await;
    common::product_share::test_update_product_main_image(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_product_clear_main_image() {
    let (ctx, tx_manager, repo, _pool) = create_sqlite_product_repo().await;
    common::product_share::test_update_product_clear_main_image(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_product_type() {
    let (ctx, tx_manager, repo, _pool) = create_sqlite_product_repo().await;
    common::product_share::test_update_product_type(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_variant_without_barcode() {
    let (ctx, tx_manager, repo, _pool) = create_sqlite_product_repo().await;
    common::product_share::test_variant_without_barcode(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_get_deleted_variant_returns_none() {
    let (ctx, tx_manager, repo, _pool) = create_sqlite_product_repo().await;
    common::product_share::test_get_deleted_variant_returns_none(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_get_deleted_product_returns_none() {
    let (ctx, tx_manager, repo, _pool) = create_sqlite_product_repo().await;
    common::product_share::test_get_deleted_product_returns_none(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_variant_only_name() {
    let (ctx, tx_manager, repo, _pool) = create_sqlite_product_repo().await;
    common::product_share::test_update_variant_only_name(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_variant_only_barcode() {
    let (ctx, tx_manager, repo, _pool) = create_sqlite_product_repo().await;
    common::product_share::test_update_variant_only_barcode(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_variant_set_metadata() {
    let pool = init_sqlite_pool().await;
    let repo: SqliteProductRepository = SqliteProductRepository::new(pool.clone());
    let tx_manager = SqliteTransactionManager::new(pool);
    let ctx = Context::new();

    let product_id = generate_test_id();
    let product = create_test_product();

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_product(&ctx, product_id, &product, &mut tx)
        .await
        .expect("Failed to create product");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    let variant_id = generate_test_id();
    let variant = create_test_variant(product_id);

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_variant(&ctx, variant_id, &variant, &mut tx)
        .await
        .expect("Failed to create variant");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    // Set new metadata
    let update = ProductVariantUpdate {
        barcode: Update::Unchanged,
        name: Update::Unchanged,
        metadata: Update::Set(json!({"new": "data", "count": 42})),
    };

    repo.update_variant(&ctx, variant_id, &update)
        .await
        .expect("Failed to update variant");

    let saved = repo
        .get_variant_by_id(&ctx, variant_id)
        .await
        .expect("Failed to get variant")
        .expect("Variant not found");

    assert_eq!(saved.metadata, Some(json!({"new": "data", "count": 42})));
}
