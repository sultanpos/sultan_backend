use serde_json::json;
use sultan_core::{
    domain::{
        Context,
        model::product::{ProductCreate, ProductVariantCreate},
    },
    storage::{
        ProductRepository,
        sqlite::{SqliteProductRepository, transaction::SqliteTransactionManager},
        transaction::TransactionManager,
    },
};

pub async fn create_sqlite_product_repo()
-> (Context, SqliteTransactionManager, SqliteProductRepository) {
    let pool = super::init_sqlite_pool().await;
    (
        Context::new(),
        SqliteTransactionManager::new(pool.clone()),
        SqliteProductRepository::new(pool.clone()),
    )
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

pub async fn product_test_create_success<'a, T, P>(ctx: &Context, tx_manager: &'a T, repo: &'a P)
where
    T: TransactionManager,
    P: ProductRepository<T::Transaction<'a>>,
{
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_product(&ctx, product_id, &product, &mut tx)
        .await
        .expect("Failed to create product");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    let saved = repo
        .get_by_id(&ctx, product_id)
        .await
        .expect("Failed to get product")
        .expect("Product not found");

    assert_eq!(saved.id, product_id);
    assert_eq!(saved.name, "Test Product");
    assert_eq!(
        saved.description,
        Some("A test product description".to_string())
    );
    assert_eq!(saved.product_type, "product");
    assert_eq!(
        saved.main_image,
        Some("https://example.com/image.jpg".to_string())
    );
    assert!(saved.sellable);
    assert!(saved.buyable);
    assert!(!saved.editable_price);
    assert!(!saved.has_variant);
    assert!(!saved.is_deleted);
}
