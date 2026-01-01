use serde_json::json;
use sqlx::{Pool, Sqlite};
use sultan_core::{
    domain::{
        Context,
        model::{
            Update,
            category::category_create_with_name,
            product::{ProductCreate, ProductUpdate, ProductVariantCreate, ProductVariantUpdate},
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
}

pub async fn test_update_product_name<'a, T, P>(ctx: &Context, tx_manager: &'a T, repo: &'a P)
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

    let update = ProductUpdate {
        name: Some("Updated Product Name".to_string()),
        description: Update::Unchanged,
        product_type: None,
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

    assert_eq!(saved.name, "Updated Product Name");
    // Other fields should remain unchanged
    assert_eq!(
        saved.description,
        Some("A test product description".to_string())
    );
}

pub async fn test_update_product_clear_description<'a, T, P>(
    ctx: &Context,
    tx_manager: &'a T,
    repo: &'a P,
) where
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

    let update = ProductUpdate {
        name: None,
        description: Update::Clear,
        product_type: None,
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

    assert_eq!(saved.description, None);
}

pub async fn test_update_product_all_fields<'a, T, P>(ctx: &Context, tx_manager: &'a T, repo: &'a P)
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

    let update = ProductUpdate {
        name: Some("Fully Updated Product".to_string()),
        description: Update::Set("New description".to_string()),
        product_type: Some("service".to_string()),
        main_image: Update::Set("https://new-image.com/img.png".to_string()),
        sellable: Some(false),
        buyable: Some(false),
        editable_price: Some(true),
        has_variant: Some(true),
        metadata: Update::Set(json!({"new_key": "new_value"})),
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

    assert_eq!(saved.name, "Fully Updated Product");
    assert_eq!(saved.description, Some("New description".to_string()));
    assert_eq!(saved.product_type, "service");
    assert_eq!(
        saved.main_image,
        Some("https://new-image.com/img.png".to_string())
    );
    assert!(!saved.sellable);
    assert!(!saved.buyable);
    assert!(saved.editable_price);
    assert!(saved.has_variant);
    assert_eq!(saved.metadata, Some(json!({"new_key": "new_value"})));
}

pub async fn test_update_product_categories<'a, T, P>(
    ctx: &Context,
    tx_manager: &'a T,
    repo: &'a P,
    pool: &Pool<Sqlite>,
) where
    T: TransactionManager,
    P: ProductRepository<T::Transaction<'a>>,
{
    let category_repo = SqliteCategoryRepository::new(pool.clone());
    // Create categories
    let cat_id1 = super::generate_test_id().await;
    let cat_id2 = super::generate_test_id().await;
    let cat_id3 = super::generate_test_id().await;

    category_repo
        .create(ctx, cat_id1, &category_create_with_name("Cat 1"))
        .await
        .expect("Failed to create category 1");
    category_repo
        .create(ctx, cat_id2, &category_create_with_name("Cat 2"))
        .await
        .expect("Failed to create category 2");
    category_repo
        .create(ctx, cat_id3, &category_create_with_name("Cat 3"))
        .await
        .expect("Failed to create category 3");

    let product_id = super::generate_test_id().await;
    let product = ProductCreate {
        name: "Product with categories".to_string(),
        description: None,
        product_type: "product".to_string(),
        main_image: None,
        sellable: true,
        buyable: true,
        editable_price: false,
        has_variant: false,
        metadata: None,
        category_ids: vec![cat_id1, cat_id2],
    };

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_product(&ctx, product_id, &product, &mut tx)
        .await
        .expect("Failed to create product");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    // Update to new categories
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
        category_ids: Some(vec![cat_id2, cat_id3]),
    };

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.update_product(&ctx, product_id, &update, &mut tx)
        .await
        .expect("Failed to update product");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    // Verify categories were updated
    let categories = repo
        .get_product_category(&ctx, product_id)
        .await
        .expect("Failed to get product categories");

    assert_eq!(categories.len(), 2);
    assert!(categories.contains(&cat_id2));
    assert!(categories.contains(&cat_id3));
    assert!(!categories.contains(&cat_id1));
}

pub async fn test_update_product_not_found<'a, T, P>(ctx: &Context, tx_manager: &'a T, repo: &'a P)
where
    T: TransactionManager,
    P: ProductRepository<T::Transaction<'a>>,
{
    let update = ProductUpdate {
        name: Some("Updated".to_string()),
        description: Update::Unchanged,
        product_type: None,
        main_image: Update::Unchanged,
        sellable: None,
        buyable: None,
        editable_price: None,
        has_variant: None,
        metadata: Update::Unchanged,
        category_ids: None,
    };

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    let result = repo.update_product(&ctx, 999999, &update, &mut tx).await;
    tx_manager
        .rollback(tx)
        .await
        .expect("Failed to rollback tx");

    assert!(matches!(
        result,
        Err(sultan_core::domain::Error::NotFound(_))
    ));
}

pub async fn test_delete_product_success<'a, T, P>(ctx: &Context, tx_manager: &'a T, repo: &'a P)
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

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.delete_product(&ctx, product_id, &mut tx)
        .await
        .expect("Failed to delete product");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    let saved = repo
        .get_by_id(&ctx, product_id)
        .await
        .expect("Failed to get product");

    assert!(saved.is_none());
}

pub async fn test_delete_product_not_found<'a, T, P>(ctx: &Context, tx_manager: &'a T, repo: &'a P)
where
    T: TransactionManager,
    P: ProductRepository<T::Transaction<'a>>,
{
    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    let result = repo.delete_product(&ctx, 999999, &mut tx).await;
    tx_manager
        .rollback(tx)
        .await
        .expect("Failed to rollback tx");

    assert!(matches!(
        result,
        Err(sultan_core::domain::Error::NotFound(_))
    ));
}

pub async fn test_get_product_by_id_not_found<'a, P>(ctx: &Context, repo: &'a P)
where
    P: ProductRepository<sqlx::Transaction<'a, Sqlite>>,
{
    let result = repo.get_by_id(&ctx, 999999).await.expect("Failed to query");

    assert!(result.is_none());
}

pub async fn test_create_variant_success<'a, T, P>(ctx: &Context, tx_manager: &'a T, repo: &'a P)
where
    T: TransactionManager,
    P: ProductRepository<T::Transaction<'a>>,
{
    // Create product first
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_product(&ctx, product_id, &product, &mut tx)
        .await
        .expect("Failed to create product");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    // Create variant
    let variant_id = super::generate_test_id().await;
    let variant = create_test_variant(product_id);

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

    assert_eq!(saved.id, variant_id);
    assert_eq!(saved.product.id, product_id);
    assert_eq!(saved.barcode, Some("1234567890".to_string()));
    assert_eq!(saved.name, Some("Default Variant".to_string()));
    assert_eq!(saved.metadata, Some(json!({"sku": "SKU001"})));
}

pub async fn test_create_variant_without_optional_fields<'a, T, P>(
    ctx: &Context,
    tx_manager: &'a T,
    repo: &'a P,
) where
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

    let variant_id = super::generate_test_id().await;
    let variant = ProductVariantCreate {
        product_id,
        barcode: None, // NULL barcode (no constraint)
        name: None,
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

    // Barcode is NULL (nullable column)
    assert_eq!(saved.barcode, None);
    assert_eq!(saved.name, None);
    assert_eq!(saved.metadata, None);
}

pub async fn test_update_variant_barcode<'a, T, P>(ctx: &Context, tx_manager: &'a T, repo: &'a P)
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

    let variant_id = super::generate_test_id().await;
    let variant = create_test_variant(product_id);

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_variant(&ctx, variant_id, &variant, &mut tx)
        .await
        .expect("Failed to create variant");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    let update = ProductVariantUpdate {
        barcode: Update::Set("9999999999".to_string()),
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

    assert_eq!(saved.barcode, Some("9999999999".to_string()));
    // Name should remain unchanged
    assert_eq!(saved.name, Some("Default Variant".to_string()));
}

pub async fn test_update_variant_clear_name<'a, T, P>(ctx: &Context, tx_manager: &'a T, repo: &'a P)
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

    let variant_id = super::generate_test_id().await;
    let variant = create_test_variant(product_id);

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_variant(&ctx, variant_id, &variant, &mut tx)
        .await
        .expect("Failed to create variant");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    let update = ProductVariantUpdate {
        barcode: Update::Unchanged,
        name: Update::Clear,
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

    assert_eq!(saved.name, None);
}

pub async fn test_update_variant_all_fields<'a, T, P>(ctx: &Context, tx_manager: &'a T, repo: &'a P)
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

    let variant_id = super::generate_test_id().await;
    let variant = create_test_variant(product_id);

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_variant(&ctx, variant_id, &variant, &mut tx)
        .await
        .expect("Failed to create variant");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    let update = ProductVariantUpdate {
        barcode: Update::Set("NEW_BARCODE".to_string()),
        name: Update::Set("New Variant Name".to_string()),
        metadata: Update::Set(json!({"new_sku": "SKU999"})),
    };

    repo.update_variant(&ctx, variant_id, &update)
        .await
        .expect("Failed to update variant");

    let saved = repo
        .get_variant_by_id(&ctx, variant_id)
        .await
        .expect("Failed to get variant")
        .expect("Variant not found");

    assert_eq!(saved.barcode, Some("NEW_BARCODE".to_string()));
    assert_eq!(saved.name, Some("New Variant Name".to_string()));
    assert_eq!(saved.metadata, Some(json!({"new_sku": "SKU999"})));
}

pub async fn test_update_variant_not_found<'a, P>(ctx: &Context, repo: &'a P)
where
    P: ProductRepository<sqlx::Transaction<'a, Sqlite>>,
{
    let update = ProductVariantUpdate {
        barcode: Update::Set("X".to_string()),
        name: Update::Unchanged,
        metadata: Update::Unchanged,
    };

    let result = repo.update_variant(&ctx, 999999, &update).await;

    assert!(matches!(
        result,
        Err(sultan_core::domain::Error::NotFound(_))
    ));
}

pub async fn test_delete_variant_success<'a, T, P>(ctx: &Context, tx_manager: &'a T, repo: &'a P)
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

    let variant_id = super::generate_test_id().await;
    let variant = create_test_variant(product_id);

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_variant(&ctx, variant_id, &variant, &mut tx)
        .await
        .expect("Failed to create variant");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.delete_variant(&ctx, variant_id, &mut tx)
        .await
        .expect("Failed to delete variant");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    let saved = repo
        .get_variant_by_id(&ctx, variant_id)
        .await
        .expect("Failed to get variant");

    assert!(saved.is_none());
}

pub async fn test_delete_variant_not_found<'a, T, P>(ctx: &Context, tx_manager: &'a T, repo: &'a P)
where
    T: TransactionManager,
    P: ProductRepository<T::Transaction<'a>>,
{
    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    let result = repo.delete_variant(&ctx, 999999, &mut tx).await;
    tx_manager
        .rollback(tx)
        .await
        .expect("Failed to rollback tx");

    assert!(matches!(
        result,
        Err(sultan_core::domain::Error::NotFound(_))
    ));
}

pub async fn test_delete_variants_by_product_id<'a, T, P>(
    ctx: &Context,
    tx_manager: &'a T,
    repo: &'a P,
) where
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

    // Create multiple variants
    let variant_id1 = super::generate_test_id().await;
    let variant_id2 = super::generate_test_id().await;
    let variant_id3 = super::generate_test_id().await;

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_variant(
        &ctx,
        variant_id1,
        &ProductVariantCreate {
            product_id,
            barcode: Some("V1".to_string()),
            name: None,
            metadata: None,
        },
        &mut tx,
    )
    .await
    .expect("Failed to create variant 1");

    repo.create_variant(
        &ctx,
        variant_id2,
        &ProductVariantCreate {
            product_id,
            barcode: Some("V2".to_string()),
            name: None,
            metadata: None,
        },
        &mut tx,
    )
    .await
    .expect("Failed to create variant 2");

    repo.create_variant(
        &ctx,
        variant_id3,
        &ProductVariantCreate {
            product_id,
            barcode: Some("V3".to_string()),
            name: None,
            metadata: None,
        },
        &mut tx,
    )
    .await
    .expect("Failed to create variant 3");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    // Verify all variants exist
    let variants = repo
        .get_variant_by_product_id(&ctx, product_id)
        .await
        .expect("Failed to get variants");
    assert_eq!(variants.len(), 3);

    // Delete all variants by product_id
    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.delete_variants_by_product_id(&ctx, product_id, &mut tx)
        .await
        .expect("Failed to delete variants");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    // Verify all variants are deleted
    let variants = repo
        .get_variant_by_product_id(&ctx, product_id)
        .await
        .expect("Failed to get variants");
    assert_eq!(variants.len(), 0);
}

pub async fn test_get_variant_by_barcode_success<'a, T, P>(
    ctx: &Context,
    tx_manager: &'a T,
    repo: &'a P,
) where
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

    let variant_id = super::generate_test_id().await;
    let unique_barcode = format!("BARCODE_{}", variant_id);
    let variant = ProductVariantCreate {
        product_id,
        barcode: Some(unique_barcode.clone()),
        name: Some("Barcode Test Variant".to_string()),
        metadata: None,
    };

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_variant(&ctx, variant_id, &variant, &mut tx)
        .await
        .expect("Failed to create variant");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    let saved = repo
        .get_variant_by_barcode(&ctx, &unique_barcode)
        .await
        .expect("Failed to get variant")
        .expect("Variant not found");

    assert_eq!(saved.id, variant_id);
    assert_eq!(saved.barcode, Some(unique_barcode));
    assert_eq!(saved.product.id, product_id);
}

pub async fn test_get_variant_by_barcode_not_found<'a, P>(ctx: &Context, repo: &'a P)
where
    P: ProductRepository<sqlx::Transaction<'a, Sqlite>>,
{
    let result = repo
        .get_variant_by_barcode(&ctx, "NONEXISTENT_BARCODE")
        .await
        .expect("Failed to query");

    assert!(result.is_none());
}

pub async fn test_get_variant_by_id_not_found<'a, P>(ctx: &Context, repo: &'a P)
where
    P: ProductRepository<sqlx::Transaction<'a, Sqlite>>,
{
    let result = repo
        .get_variant_by_id(&ctx, 999999)
        .await
        .expect("Failed to query");

    assert!(result.is_none());
}

pub async fn test_get_variant_by_product_id_success<'a, T, P>(
    ctx: &Context,
    tx_manager: &'a T,
    repo: &'a P,
) where
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

    // Create multiple variants
    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    for i in 0..3 {
        let variant_id = super::generate_test_id().await;
        let variant = ProductVariantCreate {
            product_id,
            barcode: Some(format!("BC_{}", i)),
            name: Some(format!("Variant {}", i)),
            metadata: None,
        };
        repo.create_variant(&ctx, variant_id, &variant, &mut tx)
            .await
            .expect("Failed to create variant");
    }
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    let variants = repo
        .get_variant_by_product_id(&ctx, product_id)
        .await
        .expect("Failed to get variants");

    assert_eq!(variants.len(), 3);
    for variant in &variants {
        assert_eq!(variant.product.id, product_id);
    }
}

pub async fn test_get_variant_by_product_id_empty<'a, T, P>(
    ctx: &Context,
    tx_manager: &'a T,
    repo: &'a P,
) where
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

    // Don't create any variants

    let variants = repo
        .get_variant_by_product_id(&ctx, product_id)
        .await
        .expect("Failed to get variants");

    assert_eq!(variants.len(), 0);
}

pub async fn test_get_variant_by_product_id_product_not_found<'a, P>(ctx: &Context, repo: &'a P)
where
    P: ProductRepository<sqlx::Transaction<'a, Sqlite>>,
{
    let variants = repo
        .get_variant_by_product_id(&ctx, 999999)
        .await
        .expect("Failed to get variants");

    assert_eq!(variants.len(), 0);
}

pub async fn test_transaction_rollback_product_creation<'a, T, P>(
    ctx: &Context,
    tx_manager: &'a T,
    repo: &'a P,
) where
    T: TransactionManager,
    P: ProductRepository<T::Transaction<'a>>,
{
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_product(&ctx, product_id, &product, &mut tx)
        .await
        .expect("Failed to create product");
    // Rollback instead of commit
    tx_manager
        .rollback(tx)
        .await
        .expect("Failed to rollback tx");

    // Product should NOT exist
    let saved = repo
        .get_by_id(&ctx, product_id)
        .await
        .expect("Failed to query");
    assert!(saved.is_none());
}

pub async fn test_transaction_rollback_variant_creation<'a, T, P>(
    ctx: &Context,
    tx_manager: &'a T,
    repo: &'a P,
) where
    T: TransactionManager,
    P: ProductRepository<T::Transaction<'a>>,
{
    // Create product first (committed)
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_product(&ctx, product_id, &product, &mut tx)
        .await
        .expect("Failed to create product");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    // Create variant but rollback
    let variant_id = super::generate_test_id().await;
    let variant = create_test_variant(product_id);

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_variant(&ctx, variant_id, &variant, &mut tx)
        .await
        .expect("Failed to create variant");
    tx_manager
        .rollback(tx)
        .await
        .expect("Failed to rollback tx");

    // Variant should NOT exist
    let saved = repo
        .get_variant_by_id(&ctx, variant_id)
        .await
        .expect("Failed to query");
    assert!(saved.is_none());
}

pub async fn test_transaction_product_and_variant_atomic<'a, T, P>(
    ctx: &Context,
    tx_manager: &'a T,
    repo: &'a P,
) where
    T: TransactionManager,
    P: ProductRepository<T::Transaction<'a>>,
{
    let product_id = super::generate_test_id().await;
    let variant_id = super::generate_test_id().await;

    // Create both in same transaction
    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");

    let product = create_test_product();
    repo.create_product(&ctx, product_id, &product, &mut tx)
        .await
        .expect("Failed to create product");

    let variant = create_test_variant(product_id);
    repo.create_variant(&ctx, variant_id, &variant, &mut tx)
        .await
        .expect("Failed to create variant");

    tx_manager.commit(tx).await.expect("Failed to commit tx");

    // Both should exist
    let saved_product = repo
        .get_by_id(&ctx, product_id)
        .await
        .expect("Failed to get product");
    assert!(saved_product.is_some());

    let saved_variant = repo
        .get_variant_by_id(&ctx, variant_id)
        .await
        .expect("Failed to get variant");
    assert!(saved_variant.is_some());
}

pub async fn test_soft_delete_product_preserves_data(
    ctx: &Context,
    tx_manager: &SqliteTransactionManager,
    repo: &SqliteProductRepository,
    pool: &Pool<Sqlite>,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_product(&ctx, product_id, &product, &mut tx)
        .await
        .expect("Failed to create product");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.delete_product(&ctx, product_id, &mut tx)
        .await
        .expect("Failed to delete product");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    // Verify data still exists in database (soft deleted)
    let row: Option<(i64, bool)> =
        sqlx::query_as("SELECT id, is_deleted FROM products WHERE id = ?")
            .bind(product_id)
            .fetch_optional(pool)
            .await
            .expect("Failed to query");

    assert!(row.is_some());
    let (id, is_deleted) = row.unwrap();
    assert_eq!(id, product_id);
    assert!(is_deleted);
}

pub async fn test_soft_delete_variant_preserves_data(
    ctx: &Context,
    tx_manager: &SqliteTransactionManager,
    repo: &SqliteProductRepository,
    pool: &Pool<Sqlite>,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_product(&ctx, product_id, &product, &mut tx)
        .await
        .expect("Failed to create product");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    let variant_id = super::generate_test_id().await;
    let variant = create_test_variant(product_id);

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_variant(&ctx, variant_id, &variant, &mut tx)
        .await
        .expect("Failed to create variant");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.delete_variant(&ctx, variant_id, &mut tx)
        .await
        .expect("Failed to delete variant");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    // Verify data still exists in database (soft deleted)
    let row: Option<(i64, bool)> =
        sqlx::query_as("SELECT id, is_deleted FROM product_variants WHERE id = ?")
            .bind(variant_id)
            .fetch_optional(pool)
            .await
            .expect("Failed to query");

    assert!(row.is_some());
    let (id, is_deleted) = row.unwrap();
    assert_eq!(id, variant_id);
    assert!(is_deleted);
}

pub async fn test_product_with_metadata_json<'a, T, P>(
    ctx: &Context,
    tx_manager: &'a T,
    repo: &'a P,
) where
    T: TransactionManager,
    P: ProductRepository<T::Transaction<'a>>,
{
    let product_id = super::generate_test_id().await;
    let complex_metadata = json!({
        "tags": ["electronics", "gadget"],
        "specs": {
            "weight": 100,
            "dimensions": {"width": 10, "height": 20, "depth": 5}
        },
        "featured": true
    });

    let product = ProductCreate {
        name: "Complex Metadata Product".to_string(),
        description: None,
        product_type: "product".to_string(),
        main_image: None,
        sellable: true,
        buyable: true,
        editable_price: false,
        has_variant: false,
        metadata: Some(complex_metadata.clone()),
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

    assert_eq!(saved.metadata, Some(complex_metadata));
}

pub async fn test_update_deleted_product_fails<'a, T, P>(
    ctx: &Context,
    tx_manager: &'a T,
    repo: &'a P,
) where
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

    // Delete the product
    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.delete_product(&ctx, product_id, &mut tx)
        .await
        .expect("Failed to delete product");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    // Try to update deleted product
    let update = ProductUpdate {
        name: Some("Should Fail".to_string()),
        description: Update::Unchanged,
        product_type: None,
        main_image: Update::Unchanged,
        sellable: None,
        buyable: None,
        editable_price: None,
        has_variant: None,
        metadata: Update::Unchanged,
        category_ids: None,
    };

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    let result = repo
        .update_product(&ctx, product_id, &update, &mut tx)
        .await;
    tx_manager
        .rollback(tx)
        .await
        .expect("Failed to rollback tx");

    assert!(matches!(
        result,
        Err(sultan_core::domain::Error::NotFound(_))
    ));
}

pub async fn test_update_deleted_variant_fails<'a, T, P>(
    ctx: &Context,
    tx_manager: &'a T,
    repo: &'a P,
) where
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

    let variant_id = super::generate_test_id().await;
    let variant = create_test_variant(product_id);

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_variant(&ctx, variant_id, &variant, &mut tx)
        .await
        .expect("Failed to create variant");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    // Delete the variant
    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.delete_variant(&ctx, variant_id, &mut tx)
        .await
        .expect("Failed to delete variant");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    // Try to update deleted variant
    let update = ProductVariantUpdate {
        barcode: Update::Set("SHOULD_FAIL".to_string()),
        name: Update::Unchanged,
        metadata: Update::Unchanged,
    };

    let result = repo.update_variant(&ctx, variant_id, &update).await;

    assert!(matches!(
        result,
        Err(sultan_core::domain::Error::NotFound(_))
    ));
}

pub async fn test_delete_already_deleted_product_fails<'a, T, P>(
    ctx: &Context,
    tx_manager: &'a T,
    repo: &'a P,
) where
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

    // Delete once
    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.delete_product(&ctx, product_id, &mut tx)
        .await
        .expect("Failed to delete product");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    // Try to delete again
    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    let result = repo.delete_product(&ctx, product_id, &mut tx).await;
    tx_manager
        .rollback(tx)
        .await
        .expect("Failed to rollback tx");

    assert!(matches!(
        result,
        Err(sultan_core::domain::Error::NotFound(_))
    ));
}

pub async fn test_get_variant_by_id_success<'a, T, P>(ctx: &Context, tx_manager: &'a T, repo: &'a P)
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

    let variant_id = super::generate_test_id().await;
    let variant = create_test_variant(product_id);

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_variant(&ctx, variant_id, &variant, &mut tx)
        .await
        .expect("Failed to create variant");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    let result = repo
        .get_variant_by_id(&ctx, variant_id)
        .await
        .expect("Failed to get variant");

    assert!(result.is_some());
    let saved_variant = result.unwrap();
    assert_eq!(saved_variant.id, variant_id);
    assert_eq!(saved_variant.product.id, product_id);
    assert_eq!(saved_variant.barcode, Some("1234567890".to_string()));
}

pub async fn test_get_variant_by_id_when_product_deleted<'a, T, P>(
    ctx: &Context,
    tx_manager: &'a T,
    repo: &'a P,
) where
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

    let variant_id = super::generate_test_id().await;
    let variant = create_test_variant(product_id);

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_variant(&ctx, variant_id, &variant, &mut tx)
        .await
        .expect("Failed to create variant");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    // Delete the product
    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.delete_product(&ctx, product_id, &mut tx)
        .await
        .expect("Failed to delete product");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    // Try to get variant - should return None because product is deleted
    let result = repo
        .get_variant_by_id(&ctx, variant_id)
        .await
        .expect("Failed to get variant");

    assert!(result.is_none());
}

pub async fn test_get_variant_by_barcode_when_product_deleted<'a, T, P>(
    ctx: &Context,
    tx_manager: &'a T,
    repo: &'a P,
) where
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

    let variant_id = super::generate_test_id().await;
    let variant = create_test_variant(product_id);

    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.create_variant(&ctx, variant_id, &variant, &mut tx)
        .await
        .expect("Failed to create variant");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    // Delete the product
    let mut tx = tx_manager.begin().await.expect("Failed to begin tx");
    repo.delete_product(&ctx, product_id, &mut tx)
        .await
        .expect("Failed to delete product");
    tx_manager.commit(tx).await.expect("Failed to commit tx");

    // Try to get variant by barcode - should return None because product is deleted
    let result = repo
        .get_variant_by_barcode(&ctx, "1234567890")
        .await
        .expect("Failed to get variant");

    assert!(result.is_none());
}
