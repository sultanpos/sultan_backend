mod common;

use common::init_sqlite_pool;
use serde_json::json;
use sultan_core::domain::Context;
use sultan_core::domain::model::Update;
use sultan_core::domain::model::product::{
    ProductCreate, ProductUpdate, ProductVariantCreate, ProductVariantUpdate,
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
    let pool = init_sqlite_pool().await;
    let repo: SqliteProductRepository = SqliteProductRepository::new(pool.clone());
    let tx_manager = SqliteTransactionManager::new(pool.clone());
    let ctx = Context::new();

    // Create categories
    let category_id1 = generate_test_id();
    let category_id2 = generate_test_id();

    sqlx::query("INSERT INTO categories (id, name) VALUES (?, ?), (?, ?)")
        .bind(category_id1)
        .bind("Category 1")
        .bind(category_id2)
        .bind("Category 2")
        .execute(&pool)
        .await
        .expect("Failed to create categories");

    // Create product with categories
    let product_id = generate_test_id();
    let product = ProductCreate {
        category_ids: vec![category_id1, category_id2],
        ..create_test_product()
    };

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_product(&ctx, product_id, &product, &mut tx)
        .await
        .expect("Failed to create product");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    // Update product to remove all categories
    let update = ProductUpdate {
        name: Some("Updated Product".to_string()),
        description: Update::Unchanged,
        product_type: None,
        main_image: Update::Unchanged,
        sellable: None,
        buyable: None,
        editable_price: None,
        has_variant: None,
        metadata: Update::Unchanged,
        category_ids: Some(vec![]), // Empty categories
    };

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.update_product(&ctx, product_id, &update, &mut tx)
        .await
        .expect("Failed to update product");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    // Verify categories were removed
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM product_categories WHERE product_id = ?")
            .bind(product_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count categories");

    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_update_product_replace_categories() {
    let pool = init_sqlite_pool().await;
    let repo: SqliteProductRepository = SqliteProductRepository::new(pool.clone());
    let tx_manager = SqliteTransactionManager::new(pool.clone());
    let ctx = Context::new();

    // Create categories
    let category_id1 = generate_test_id();
    let category_id2 = generate_test_id();
    let category_id3 = generate_test_id();

    sqlx::query("INSERT INTO categories (id, name) VALUES (?, ?), (?, ?), (?, ?)")
        .bind(category_id1)
        .bind("Category 1")
        .bind(category_id2)
        .bind("Category 2")
        .bind(category_id3)
        .bind("Category 3")
        .execute(&pool)
        .await
        .expect("Failed to create categories");

    // Create product with categories
    let product_id = generate_test_id();
    let product = ProductCreate {
        category_ids: vec![category_id1, category_id2],
        ..create_test_product()
    };

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_product(&ctx, product_id, &product, &mut tx)
        .await
        .expect("Failed to create product");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    // Replace with different categories
    let update = ProductUpdate {
        name: None,
        description: Update::Unchanged,
        product_type: None,
        main_image: Update::Unchanged,
        sellable: None,
        buyable: None,
        editable_price: None,
        has_variant: None,
        metadata: Update::Unchanged,
        category_ids: Some(vec![category_id3]),
    };

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.update_product(&ctx, product_id, &update, &mut tx)
        .await
        .expect("Failed to update product");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    // Verify only category_id3 is associated
    let categories: Vec<i64> = sqlx::query_scalar(
        "SELECT category_id FROM product_categories WHERE product_id = ? ORDER BY category_id",
    )
    .bind(product_id)
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch categories");

    assert_eq!(categories, vec![category_id3]);
}

#[tokio::test]
async fn test_update_product_only_metadata() {
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

    // Update only metadata
    let update = ProductUpdate {
        name: None,
        description: Update::Unchanged,
        product_type: None,
        main_image: Update::Unchanged,
        sellable: None,
        buyable: None,
        editable_price: None,
        has_variant: None,
        metadata: Update::Set(json!({"updated": true, "version": 2})),
        category_ids: None,
    };

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.update_product(&ctx, product_id, &update, &mut tx)
        .await
        .expect("Failed to update product");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    let saved = repo
        .get_by_id(&ctx, product_id)
        .await
        .expect("Failed to get product")
        .expect("Product not found");

    assert_eq!(saved.metadata, Some(json!({"updated": true, "version": 2})));
    assert_eq!(saved.name, "Test Product"); // Original name unchanged
}

#[tokio::test]
async fn test_update_product_clear_metadata() {
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

    // Clear metadata
    let update = ProductUpdate {
        name: None,
        description: Update::Unchanged,
        product_type: None,
        main_image: Update::Unchanged,
        sellable: None,
        buyable: None,
        editable_price: None,
        has_variant: None,
        metadata: Update::Clear,
        category_ids: None,
    };

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.update_product(&ctx, product_id, &update, &mut tx)
        .await
        .expect("Failed to update product");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    let saved = repo
        .get_by_id(&ctx, product_id)
        .await
        .expect("Failed to get product")
        .expect("Product not found");

    assert_eq!(saved.metadata, None);
}

#[tokio::test]
async fn test_update_variant_clear_metadata() {
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

    // Clear metadata
    let update = ProductVariantUpdate {
        barcode: Update::Unchanged,
        name: Update::Unchanged,
        metadata: Update::Clear,
    };

    repo.update_variant(&ctx, variant_id, &update)
        .await
        .expect("Failed to update variant");

    let saved = repo
        .get_variant_by_id(&ctx, variant_id)
        .await
        .expect("Failed to get variant")
        .expect("Variant not found");

    assert_eq!(saved.metadata, None);
}

#[tokio::test]
async fn test_update_variant_clear_barcode() {
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

    // Clear barcode
    let update = ProductVariantUpdate {
        barcode: Update::Clear,
        name: Update::Unchanged,
        metadata: Update::Unchanged,
    };

    repo.update_variant(&ctx, variant_id, &update)
        .await
        .expect("Failed to update variant");

    let saved = repo
        .get_variant_by_id(&ctx, variant_id)
        .await
        .expect("Failed to get variant")
        .expect("Variant not found");

    assert_eq!(saved.barcode, None);
}

#[tokio::test]
async fn test_multiple_variants_for_single_product() {
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

    // Create 5 variants
    let variant_ids: Vec<i64> = (0..5).map(|_| generate_test_id()).collect();

    for (i, &variant_id) in variant_ids.iter().enumerate() {
        let variant = ProductVariantCreate {
            product_id,
            barcode: Some(format!("BARCODE{}", i)),
            name: Some(format!("Variant {}", i)),
            metadata: Some(json!({"index": i})),
        };

        let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
        repo.create_variant(&ctx, variant_id, &variant, &mut tx)
            .await
            .expect("Failed to create variant");
        tx_manager.commit(tx).await.expect("Failed to commit tx");
    }

    // Get all variants
    let variants = repo
        .get_variant_by_product_id(&ctx, product_id)
        .await
        .expect("Failed to get variants");

    assert_eq!(variants.len(), 5);

    // Verify each variant
    for (i, variant) in variants.iter().enumerate() {
        assert_eq!(variant.product.id, product_id);
        assert_eq!(variant.name, Some(format!("Variant {}", i)));
    }
}

#[tokio::test]
async fn test_delete_variants_by_product_id_preserves_other_products() {
    let pool = init_sqlite_pool().await;
    let repo: SqliteProductRepository = SqliteProductRepository::new(pool.clone());
    let tx_manager = SqliteTransactionManager::new(pool);
    let ctx = Context::new();

    // Create two products
    let product_id1 = generate_test_id();
    let product1 = create_test_product();

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_product(&ctx, product_id1, &product1, &mut tx)
        .await
        .expect("Failed to create product1");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    let product_id2 = generate_test_id();
    let product2 = create_test_product();

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_product(&ctx, product_id2, &product2, &mut tx)
        .await
        .expect("Failed to create product2");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    // Create variants for both products
    let variant_id1 = generate_test_id();
    let variant1 = create_test_variant(product_id1);

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_variant(&ctx, variant_id1, &variant1, &mut tx)
        .await
        .expect("Failed to create variant1");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    let variant_id2 = generate_test_id();
    let variant2 = create_test_variant(product_id2);

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_variant(&ctx, variant_id2, &variant2, &mut tx)
        .await
        .expect("Failed to create variant2");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    // Delete variants for product1 only
    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.delete_variants_by_product_id(&ctx, product_id1, &mut tx)
        .await
        .expect("Failed to delete variants");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    // Verify product1 variants are deleted
    let variants1 = repo
        .get_variant_by_product_id(&ctx, product_id1)
        .await
        .expect("Failed to get variants");
    assert_eq!(variants1.len(), 0);

    // Verify product2 variants still exist
    let variants2 = repo
        .get_variant_by_product_id(&ctx, product_id2)
        .await
        .expect("Failed to get variants");
    assert_eq!(variants2.len(), 1);
    assert_eq!(variants2[0].id, variant_id2);
}

#[tokio::test]
async fn test_update_product_boolean_flags() {
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

    // Update all boolean flags
    let update = ProductUpdate {
        name: None,
        description: Update::Unchanged,
        product_type: None,
        main_image: Update::Unchanged,
        sellable: Some(false),
        buyable: Some(false),
        editable_price: Some(true),
        has_variant: Some(true),
        metadata: Update::Unchanged,
        category_ids: None,
    };

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.update_product(&ctx, product_id, &update, &mut tx)
        .await
        .expect("Failed to update product");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    let saved = repo
        .get_by_id(&ctx, product_id)
        .await
        .expect("Failed to get product")
        .expect("Product not found");

    assert!(!saved.sellable);
    assert!(!saved.buyable);
    assert!(saved.editable_price);
    assert!(saved.has_variant);
}

#[tokio::test]
async fn test_update_product_main_image() {
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

    // Update main_image
    let update = ProductUpdate {
        name: None,
        description: Update::Unchanged,
        product_type: None,
        main_image: Update::Set("https://example.com/new-image.jpg".to_string()),
        sellable: None,
        buyable: None,
        editable_price: None,
        has_variant: None,
        metadata: Update::Unchanged,
        category_ids: None,
    };

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.update_product(&ctx, product_id, &update, &mut tx)
        .await
        .expect("Failed to update product");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    let saved = repo
        .get_by_id(&ctx, product_id)
        .await
        .expect("Failed to get product")
        .expect("Product not found");

    assert_eq!(
        saved.main_image,
        Some("https://example.com/new-image.jpg".to_string())
    );
}

#[tokio::test]
async fn test_update_product_clear_main_image() {
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

    // Clear main_image
    let update = ProductUpdate {
        name: None,
        description: Update::Unchanged,
        product_type: None,
        main_image: Update::Clear,
        sellable: None,
        buyable: None,
        editable_price: None,
        has_variant: None,
        metadata: Update::Unchanged,
        category_ids: None,
    };

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.update_product(&ctx, product_id, &update, &mut tx)
        .await
        .expect("Failed to update product");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    let saved = repo
        .get_by_id(&ctx, product_id)
        .await
        .expect("Failed to get product")
        .expect("Product not found");

    assert_eq!(saved.main_image, None);
}

#[tokio::test]
async fn test_update_product_type() {
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

    // Update product_type
    let update = ProductUpdate {
        name: None,
        description: Update::Unchanged,
        product_type: Some("service".to_string()),
        main_image: Update::Unchanged,
        sellable: None,
        buyable: None,
        editable_price: None,
        has_variant: None,
        metadata: Update::Unchanged,
        category_ids: None,
    };

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.update_product(&ctx, product_id, &update, &mut tx)
        .await
        .expect("Failed to update product");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    let saved = repo
        .get_by_id(&ctx, product_id)
        .await
        .expect("Failed to get product")
        .expect("Product not found");

    assert_eq!(saved.product_type, "service");
}

#[tokio::test]
async fn test_variant_without_barcode() {
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
    let variant = ProductVariantCreate {
        product_id,
        barcode: None, // No barcode
        name: Some("No Barcode Variant".to_string()),
        metadata: None,
    };

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_variant(&ctx, variant_id, &variant, &mut tx)
        .await
        .expect("Failed to create variant");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    let saved = repo
        .get_variant_by_id(&ctx, variant_id)
        .await
        .expect("Failed to get variant")
        .expect("Variant not found");

    assert_eq!(saved.barcode, None);
    assert_eq!(saved.name, Some("No Barcode Variant".to_string()));
}

#[tokio::test]
async fn test_get_deleted_variant_returns_none() {
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

    // Delete variant
    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.delete_variant(&ctx, variant_id, &mut tx)
        .await
        .expect("Failed to delete variant");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    // Try to get by ID
    let by_id = repo
        .get_variant_by_id(&ctx, variant_id)
        .await
        .expect("Failed to get variant");
    assert!(by_id.is_none());

    // Try to get by barcode
    let by_barcode = repo
        .get_variant_by_barcode(&ctx, "1234567890")
        .await
        .expect("Failed to get variant");
    assert!(by_barcode.is_none());

    // Verify not in product variants list
    let variants = repo
        .get_variant_by_product_id(&ctx, product_id)
        .await
        .expect("Failed to get variants");
    assert_eq!(variants.len(), 0);
}

#[tokio::test]
async fn test_get_deleted_product_returns_none() {
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

    // Delete product
    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.delete_product(&ctx, product_id, &mut tx)
        .await
        .expect("Failed to delete product");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    // Try to get by ID
    let result = repo
        .get_by_id(&ctx, product_id)
        .await
        .expect("Failed to get product");
    assert!(result.is_none());
}

#[tokio::test]
async fn test_update_variant_only_name() {
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

    // Update only name
    let update = ProductVariantUpdate {
        barcode: Update::Unchanged,
        name: Update::Set("New Name".to_string()),
        metadata: Update::Unchanged,
    };

    repo.update_variant(&ctx, variant_id, &update)
        .await
        .expect("Failed to update variant");

    let saved = repo
        .get_variant_by_id(&ctx, variant_id)
        .await
        .expect("Failed to get variant")
        .expect("Variant not found");

    assert_eq!(saved.name, Some("New Name".to_string()));
    assert_eq!(saved.barcode, Some("1234567890".to_string())); // Unchanged
}

#[tokio::test]
async fn test_update_variant_only_barcode() {
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

    // Update only barcode
    let update = ProductVariantUpdate {
        barcode: Update::Set("NEW-BARCODE-123".to_string()),
        name: Update::Unchanged,
        metadata: Update::Unchanged,
    };

    repo.update_variant(&ctx, variant_id, &update)
        .await
        .expect("Failed to update variant");

    let saved = repo
        .get_variant_by_id(&ctx, variant_id)
        .await
        .expect("Failed to get variant")
        .expect("Variant not found");

    assert_eq!(saved.barcode, Some("NEW-BARCODE-123".to_string()));
    assert_eq!(saved.name, Some("Default Variant".to_string())); // Unchanged
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
