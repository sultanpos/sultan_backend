use serde_json::json;
use sqlx::{Pool, Sqlite, pool};
use sultan_core::{
    domain::{
        Context,
        model::{
            category::category_create_with_name,
            product::{ProductCreate, ProductVariantCreate},
        },
    },
    storage::{
        CategoryRepository, ProductRepository,
        sqlite::{
            SqliteCategoryRepository, SqliteProductRepository,
            transaction::SqliteTransactionManager,
        },
        transaction::TransactionManager,
    },
};

pub async fn create_sqlite_product_repo() -> (
    Context,
    SqliteTransactionManager,
    SqliteProductRepository,
    Pool<Sqlite>,
) {
    let pool = super::init_sqlite_pool().await;
    (
        Context::new(),
        SqliteTransactionManager::new(pool.clone()),
        SqliteProductRepository::new(pool.clone()),
        pool,
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

pub async fn test_create_product_without_optional_fields<'a, T, P>(
    ctx: &Context,
    tx_manager: &'a T,
    repo: &'a P,
) where
    T: TransactionManager,
    P: ProductRepository<T::Transaction<'a>>,
{
    let product_id = super::generate_test_id().await;
    let product = ProductCreate {
        name: "Minimal Product".to_string(),
        description: None,
        product_type: "service".to_string(),
        main_image: None,
        sellable: false,
        buyable: false,
        editable_price: true,
        has_variant: true,
        metadata: None,
        category_ids: vec![],
    };

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

    assert_eq!(saved.name, "Minimal Product");
    assert_eq!(saved.description, None);
    assert_eq!(saved.product_type, "service");
    assert_eq!(saved.main_image, None);
    assert!(!saved.sellable);
    assert!(!saved.buyable);
    assert!(saved.editable_price);
    assert!(saved.has_variant);
    assert_eq!(saved.metadata, None);
}

pub async fn test_create_product_with_categories<'a, T, P>(
    ctx: &Context,
    tx_manager: &'a T,
    repo: &'a P,
    pool: &Pool<Sqlite>,
) where
    T: TransactionManager,
    P: ProductRepository<T::Transaction<'a>>,
{
    let category_repo = SqliteCategoryRepository::new(pool.clone());
    // First create categories
    let category_id1 = super::generate_test_id().await;
    let category_id2 = super::generate_test_id().await;

    category_repo
        .create(ctx, category_id1, &category_create_with_name("Category 1"))
        .await
        .expect("Failed to create category 1");
    category_repo
        .create(ctx, category_id2, &category_create_with_name("Category 2"))
        .await
        .expect("Failed to create category 2");

    let product_id = super::generate_test_id().await;
    let product = ProductCreate {
        name: "Categorized Product".to_string(),
        description: None,
        product_type: "product".to_string(),
        main_image: None,
        sellable: true,
        buyable: true,
        editable_price: false,
        has_variant: false,
        metadata: None,
        category_ids: vec![category_id1, category_id2],
    };

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_product(&ctx, product_id, &product, &mut tx)
        .await
        .expect("Failed to create product");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    let categories = repo
        .get_product_category(&ctx, product_id)
        .await
        .expect("Failed to get product categories");
    assert_eq!(categories.len(), 2);
    assert!(categories.contains(&category_id1));
    assert!(categories.contains(&category_id2));

    // Verify categories were linked
    /*let categories: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT product_id, category_id FROM product_categories WHERE product_id = ?",
    )
    .bind(product_id)
    .fetch_all(&pool)
    .await
    .expect("Failed to get categories");

    assert_eq!(categories.len(), 2);*/
}
