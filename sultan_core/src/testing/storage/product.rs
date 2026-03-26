use sea_orm::DatabaseConnection;
use serde_json::json;

use crate::{
    domain::{
        error::Error,
        model::{
            Update,
            category::CategoryCreate,
            product::{ProductCreate, ProductUpdate, ProductVariantCreate, ProductVariantUpdate},
            sell_price::{SellDiscountCreate, SellPriceCreate},
        },
    },
    storage::{CategoryRepository, ProductRepository, RepoCtx},
};

/// Creates a test product with default values.
fn create_test_product() -> ProductCreate {
    ProductCreate {
        name: "Test Product".to_string(),
        description: Some("A test product description".to_string()),
        product_type: "product".to_string(),
        main_image: Some("https://example.com/image.jpg".to_string()),
        sellable: true,
        buyable: true,
        editable_price: false,
        metadata: Some(json!({"key": "value"})),
        variant_count: 0,
        category_ids: vec![],
    }
}

/// Creates a test variant with default values for the given product.
fn create_test_variant(product_id: i64) -> ProductVariantCreate {
    ProductVariantCreate {
        product_id,
        barcode: Some("1234567890".to_string()),
        name: Some("Default Variant".to_string()),
        metadata: Some(json!({"sku": "SKU001"})),
    }
}

/// Runs all ProductRepository tests using the provided repository and context factory.
///
/// # Arguments
///
/// * `repo` - The repository implementation to test
/// * `ctx_factory` - A factory function that creates a fresh RepoCtx for each test
pub async fn product_test_all<C, F, Fut>(repo: &C, ctx_factory: F)
where
    C: ProductRepository,
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = RepoCtx<DatabaseConnection>>,
{
    // Product CRUD tests
    product_test_create_success(&ctx_factory().await, repo).await;
    product_test_create_without_optional_fields(&ctx_factory().await, repo).await;
    product_test_get_by_id_not_found(&ctx_factory().await, repo).await;
    product_test_update_name(&ctx_factory().await, repo).await;
    product_test_update_clear_description(&ctx_factory().await, repo).await;
    product_test_update_all_fields(&ctx_factory().await, repo).await;
    product_test_update_not_found(&ctx_factory().await, repo).await;
    product_test_delete_success(&ctx_factory().await, repo).await;
    product_test_delete_not_found(&ctx_factory().await, repo).await;
    product_test_update_deleted_fails(&ctx_factory().await, repo).await;
    product_test_delete_already_deleted_fails(&ctx_factory().await, repo).await;
    product_test_get_deleted_returns_none(&ctx_factory().await, repo).await;
    product_test_with_metadata_json(&ctx_factory().await, repo).await;
    product_test_update_only_metadata(&ctx_factory().await, repo).await;
    product_test_update_clear_metadata(&ctx_factory().await, repo).await;
    product_test_update_boolean_flags(&ctx_factory().await, repo).await;
    product_test_update_main_image(&ctx_factory().await, repo).await;
    product_test_update_clear_main_image(&ctx_factory().await, repo).await;
    product_test_update_product_type(&ctx_factory().await, repo).await;

    // Variant CRUD tests
    product_test_create_variant_success(&ctx_factory().await, repo).await;
    product_test_create_variant_without_optional_fields(&ctx_factory().await, repo).await;
    product_test_update_variant_barcode(&ctx_factory().await, repo).await;
    product_test_update_variant_clear_name(&ctx_factory().await, repo).await;
    product_test_update_variant_all_fields(&ctx_factory().await, repo).await;
    product_test_update_variant_not_found(&ctx_factory().await, repo).await;
    product_test_delete_variant_success(&ctx_factory().await, repo).await;
    product_test_delete_variant_not_found(&ctx_factory().await, repo).await;
    product_test_delete_variants_by_product_id(&ctx_factory().await, repo).await;
    product_test_update_deleted_variant_fails(&ctx_factory().await, repo).await;
    product_test_get_deleted_variant_returns_none(&ctx_factory().await, repo).await;
    product_test_update_variant_clear_metadata(&ctx_factory().await, repo).await;
    product_test_update_variant_clear_barcode(&ctx_factory().await, repo).await;
    product_test_update_variant_only_name(&ctx_factory().await, repo).await;
    product_test_update_variant_only_barcode(&ctx_factory().await, repo).await;
    product_test_update_variant_set_metadata(&ctx_factory().await, repo).await;
    product_test_variant_without_barcode(&ctx_factory().await, repo).await;
    product_test_multiple_variants_for_single_product(&ctx_factory().await, repo).await;
    product_test_delete_variants_preserves_other_products(&ctx_factory().await, repo).await;

    // Variant get tests
    product_test_get_variant_by_barcode_success(&ctx_factory().await, repo).await;
    product_test_get_variant_by_barcode_not_found(&ctx_factory().await, repo).await;
    product_test_get_variant_by_id_success(&ctx_factory().await, repo).await;
    product_test_get_variant_by_id_not_found(&ctx_factory().await, repo).await;
    product_test_get_variant_by_product_id_success(&ctx_factory().await, repo).await;
    product_test_get_variant_by_product_id_empty(&ctx_factory().await, repo).await;
    product_test_get_variant_by_product_id_product_not_found(&ctx_factory().await, repo).await;
    product_test_get_variant_by_id_when_product_deleted(&ctx_factory().await, repo).await;
    product_test_get_variant_by_barcode_when_product_deleted(&ctx_factory().await, repo).await;

    // Variant nested data tests (with SellPrices and Discounts)
    product_test_get_variant_by_id_with_nested_data(&ctx_factory().await, repo).await;
    product_test_get_variant_by_barcode_with_nested_data(&ctx_factory().await, repo).await;
    product_test_get_variant_excludes_soft_deleted_relations(&ctx_factory().await, repo).await;

    // Variant ID listing tests
    product_test_get_variant_ids_by_product_id_success(&ctx_factory().await, repo).await;
    product_test_get_variant_ids_by_product_id_empty(&ctx_factory().await, repo).await;
    product_test_get_variant_ids_by_product_id_excludes_deleted(&ctx_factory().await, repo).await;

    // Product category tests
    product_test_add_product_category_success(&ctx_factory().await, repo).await;
    product_test_add_product_category_empty_array(&ctx_factory().await, repo).await;
    product_test_add_product_category_duplicate(&ctx_factory().await, repo).await;

    // Product get_by_id full data tests
    product_test_get_by_id_with_categories(&ctx_factory().await, repo).await;
    product_test_get_by_id_with_variants_and_sell_prices(&ctx_factory().await, repo).await;
    product_test_get_by_id_with_full_data(&ctx_factory().await, repo).await;
    product_test_get_by_id_excludes_deleted_categories(&ctx_factory().await, repo).await;
    product_test_get_by_id_excludes_deleted_variants(&ctx_factory().await, repo).await;
    product_test_get_by_id_empty_categories_and_variants(&ctx_factory().await, repo).await;
}

// =============================================================================
// Product CRUD Tests
// =============================================================================

/// Test: Create a product with all fields
pub async fn product_test_create_success<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let saved = repo
        .get_by_id(ctx, product_id)
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
    assert!(!saved.is_deleted);
}

/// Test: Create a product without optional fields
pub async fn product_test_create_without_optional_fields<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = ProductCreate {
        name: "Minimal Product".to_string(),
        description: None,
        product_type: "service".to_string(),
        main_image: None,
        sellable: false,
        buyable: false,
        editable_price: true,
        metadata: None,
        variant_count: 0,
        category_ids: vec![],
    };

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let saved = repo
        .get_by_id(ctx, product_id)
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
    assert_eq!(saved.metadata, None);
}

/// Test: Get product by ID when not found
pub async fn product_test_get_by_id_not_found<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let result = repo.get_by_id(ctx, 999999).await.expect("Failed to query");

    assert!(result.is_none());
}

/// Test: Update product name
pub async fn product_test_update_name<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let update = ProductUpdate {
        name: Some("Updated Product Name".to_string()),
        description: Update::Unchanged,
        product_type: None,
        main_image: Update::Unchanged,
        sellable: None,
        buyable: None,
        editable_price: None,
        metadata: Update::Unchanged,
        category_ids: None,
    };

    repo.update_product(ctx, product_id, &update)
        .await
        .expect("Failed to update product");

    let saved = repo
        .get_by_id(ctx, product_id)
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

/// Test: Update product to clear description
pub async fn product_test_update_clear_description<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let update = ProductUpdate {
        name: None,
        description: Update::Clear,
        product_type: None,
        main_image: Update::Unchanged,
        sellable: None,
        buyable: None,
        editable_price: None,
        metadata: Update::Unchanged,
        category_ids: None,
    };

    repo.update_product(ctx, product_id, &update)
        .await
        .expect("Failed to update product");

    let saved = repo
        .get_by_id(ctx, product_id)
        .await
        .expect("Failed to get product")
        .expect("Product not found");

    assert_eq!(saved.description, None);
}

/// Test: Update all product fields
pub async fn product_test_update_all_fields<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let update = ProductUpdate {
        name: Some("Fully Updated Product".to_string()),
        description: Update::Set("New description".to_string()),
        product_type: Some("service".to_string()),
        main_image: Update::Set("https://new-image.com/img.png".to_string()),
        sellable: Some(false),
        buyable: Some(false),
        editable_price: Some(true),
        metadata: Update::Set(json!({"new_key": "new_value"})),
        category_ids: None,
    };

    repo.update_product(ctx, product_id, &update)
        .await
        .expect("Failed to update product");

    let saved = repo
        .get_by_id(ctx, product_id)
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
    assert_eq!(saved.metadata, Some(json!({"new_key": "new_value"})));
}

/// Test: Update non-existent product fails
pub async fn product_test_update_not_found<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let update = ProductUpdate {
        name: Some("Updated".to_string()),
        description: Update::Unchanged,
        product_type: None,
        main_image: Update::Unchanged,
        sellable: None,
        buyable: None,
        editable_price: None,
        metadata: Update::Unchanged,
        category_ids: None,
    };

    let result = repo.update_product(ctx, 999999, &update).await;

    assert!(matches!(result, Err(Error::NotFound(_))));
}

/// Test: Delete product success
pub async fn product_test_delete_success<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    repo.delete_product(ctx, product_id)
        .await
        .expect("Failed to delete product");

    let saved = repo
        .get_by_id(ctx, product_id)
        .await
        .expect("Failed to get product");

    assert!(saved.is_none());
}

/// Test: Delete non-existent product fails
pub async fn product_test_delete_not_found<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let result = repo.delete_product(ctx, 999999).await;

    assert!(matches!(result, Err(Error::NotFound(_))));
}

/// Test: Update deleted product fails
pub async fn product_test_update_deleted_fails<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    repo.delete_product(ctx, product_id)
        .await
        .expect("Failed to delete product");

    let update = ProductUpdate {
        name: Some("Should Fail".to_string()),
        description: Update::Unchanged,
        product_type: None,
        main_image: Update::Unchanged,
        sellable: None,
        buyable: None,
        editable_price: None,
        metadata: Update::Unchanged,
        category_ids: None,
    };

    let result = repo.update_product(ctx, product_id, &update).await;

    assert!(matches!(result, Err(Error::NotFound(_))));
}

/// Test: Delete already deleted product fails
pub async fn product_test_delete_already_deleted_fails<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    repo.delete_product(ctx, product_id)
        .await
        .expect("Failed to delete product");

    let result = repo.delete_product(ctx, product_id).await;

    assert!(matches!(result, Err(Error::NotFound(_))));
}

/// Test: Get deleted product returns None
pub async fn product_test_get_deleted_returns_none<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    repo.delete_product(ctx, product_id)
        .await
        .expect("Failed to delete product");

    let result = repo
        .get_by_id(ctx, product_id)
        .await
        .expect("Failed to get product");

    assert!(result.is_none());
}

/// Test: Product with complex JSON metadata
pub async fn product_test_with_metadata_json<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
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
        variant_count: 0,
        metadata: Some(complex_metadata.clone()),
        category_ids: vec![],
    };

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let saved = repo
        .get_by_id(ctx, product_id)
        .await
        .expect("Failed to get product")
        .expect("Product not found");

    assert_eq!(saved.metadata, Some(complex_metadata));
}

/// Test: Update only metadata
pub async fn product_test_update_only_metadata<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let update = ProductUpdate {
        name: None,
        description: Update::Unchanged,
        product_type: None,
        main_image: Update::Unchanged,
        sellable: None,
        buyable: None,
        editable_price: None,
        metadata: Update::Set(json!({"updated": true, "version": 2})),
        category_ids: None,
    };

    repo.update_product(ctx, product_id, &update)
        .await
        .expect("Failed to update product");

    let saved = repo
        .get_by_id(ctx, product_id)
        .await
        .expect("Failed to get product")
        .expect("Product not found");

    assert_eq!(saved.metadata, Some(json!({"updated": true, "version": 2})));
    assert_eq!(saved.name, "Test Product"); // Original name unchanged
}

/// Test: Update to clear metadata
pub async fn product_test_update_clear_metadata<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let update = ProductUpdate {
        name: None,
        description: Update::Unchanged,
        product_type: None,
        main_image: Update::Unchanged,
        sellable: None,
        buyable: None,
        editable_price: None,
        metadata: Update::Clear,
        category_ids: None,
    };

    repo.update_product(ctx, product_id, &update)
        .await
        .expect("Failed to update product");

    let saved = repo
        .get_by_id(ctx, product_id)
        .await
        .expect("Failed to get product")
        .expect("Product not found");

    assert_eq!(saved.metadata, None);
}

/// Test: Update boolean flags
pub async fn product_test_update_boolean_flags<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let update = ProductUpdate {
        name: None,
        description: Update::Unchanged,
        product_type: None,
        main_image: Update::Unchanged,
        sellable: Some(false),
        buyable: Some(false),
        editable_price: Some(true),
        metadata: Update::Unchanged,
        category_ids: None,
    };

    repo.update_product(ctx, product_id, &update)
        .await
        .expect("Failed to update product");

    let saved = repo
        .get_by_id(ctx, product_id)
        .await
        .expect("Failed to get product")
        .expect("Product not found");

    assert!(!saved.sellable);
    assert!(!saved.buyable);
    assert!(saved.editable_price);
}

/// Test: Update main_image
pub async fn product_test_update_main_image<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let update = ProductUpdate {
        name: None,
        description: Update::Unchanged,
        product_type: None,
        main_image: Update::Set("https://example.com/new-image.jpg".to_string()),
        sellable: None,
        buyable: None,
        editable_price: None,
        metadata: Update::Unchanged,
        category_ids: None,
    };

    repo.update_product(ctx, product_id, &update)
        .await
        .expect("Failed to update product");

    let saved = repo
        .get_by_id(ctx, product_id)
        .await
        .expect("Failed to get product")
        .expect("Product not found");

    assert_eq!(
        saved.main_image,
        Some("https://example.com/new-image.jpg".to_string())
    );
}

/// Test: Update to clear main_image
pub async fn product_test_update_clear_main_image<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let update = ProductUpdate {
        name: None,
        description: Update::Unchanged,
        product_type: None,
        main_image: Update::Clear,
        sellable: None,
        buyable: None,
        editable_price: None,
        metadata: Update::Unchanged,
        category_ids: None,
    };

    repo.update_product(ctx, product_id, &update)
        .await
        .expect("Failed to update product");

    let saved = repo
        .get_by_id(ctx, product_id)
        .await
        .expect("Failed to get product")
        .expect("Product not found");

    assert_eq!(saved.main_image, None);
}

/// Test: Update product_type
pub async fn product_test_update_product_type<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let update = ProductUpdate {
        name: None,
        description: Update::Unchanged,
        product_type: Some("service".to_string()),
        main_image: Update::Unchanged,
        sellable: None,
        buyable: None,
        editable_price: None,
        metadata: Update::Unchanged,
        category_ids: None,
    };

    repo.update_product(ctx, product_id, &update)
        .await
        .expect("Failed to update product");

    let saved = repo
        .get_by_id(ctx, product_id)
        .await
        .expect("Failed to get product")
        .expect("Product not found");

    assert_eq!(saved.product_type, "service");
}

// =============================================================================
// Variant CRUD Tests
// =============================================================================

/// Test: Create variant success
pub async fn product_test_create_variant_success<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let variant_id = super::generate_test_id().await;
    let variant = create_test_variant(product_id);

    repo.create_variant(ctx, variant_id, &variant)
        .await
        .expect("Failed to create variant");

    let saved = repo
        .get_variant_by_id(ctx, variant_id)
        .await
        .expect("Failed to get variant")
        .expect("Variant not found");

    assert_eq!(saved.id, variant_id);
    assert_eq!(saved.barcode, Some("1234567890".to_string()));
    assert_eq!(saved.name, Some("Default Variant".to_string()));
    assert_eq!(saved.metadata, Some(json!({"sku": "SKU001"})));
}

/// Test: Create variant without optional fields
pub async fn product_test_create_variant_without_optional_fields<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let variant_id = super::generate_test_id().await;
    let variant = ProductVariantCreate {
        product_id,
        barcode: None,
        name: None,
        metadata: None,
    };

    repo.create_variant(ctx, variant_id, &variant)
        .await
        .expect("Failed to create variant");

    let saved = repo
        .get_variant_by_id(ctx, variant_id)
        .await
        .expect("Failed to get variant")
        .expect("Variant not found");

    assert_eq!(saved.barcode, None);
    assert_eq!(saved.name, None);
    assert_eq!(saved.metadata, None);
}

/// Test: Update variant barcode
pub async fn product_test_update_variant_barcode<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let variant_id = super::generate_test_id().await;
    let variant = create_test_variant(product_id);

    repo.create_variant(ctx, variant_id, &variant)
        .await
        .expect("Failed to create variant");

    let update = ProductVariantUpdate {
        barcode: Update::Set("9999999999".to_string()),
        name: Update::Unchanged,
        metadata: Update::Unchanged,
    };

    repo.update_variant(ctx, variant_id, &update)
        .await
        .expect("Failed to update variant");

    let saved = repo
        .get_variant_by_id(ctx, variant_id)
        .await
        .expect("Failed to get variant")
        .expect("Variant not found");

    assert_eq!(saved.barcode, Some("9999999999".to_string()));
    assert_eq!(saved.name, Some("Default Variant".to_string()));
}

/// Test: Update variant to clear name
pub async fn product_test_update_variant_clear_name<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let variant_id = super::generate_test_id().await;
    let variant = create_test_variant(product_id);

    repo.create_variant(ctx, variant_id, &variant)
        .await
        .expect("Failed to create variant");

    let update = ProductVariantUpdate {
        barcode: Update::Unchanged,
        name: Update::Clear,
        metadata: Update::Unchanged,
    };

    repo.update_variant(ctx, variant_id, &update)
        .await
        .expect("Failed to update variant");

    let saved = repo
        .get_variant_by_id(ctx, variant_id)
        .await
        .expect("Failed to get variant")
        .expect("Variant not found");

    assert_eq!(saved.name, None);
}

/// Test: Update all variant fields
pub async fn product_test_update_variant_all_fields<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let variant_id = super::generate_test_id().await;
    let variant = create_test_variant(product_id);

    repo.create_variant(ctx, variant_id, &variant)
        .await
        .expect("Failed to create variant");

    let update = ProductVariantUpdate {
        barcode: Update::Set("NEW_BARCODE".to_string()),
        name: Update::Set("New Variant Name".to_string()),
        metadata: Update::Set(json!({"new_sku": "SKU999"})),
    };

    repo.update_variant(ctx, variant_id, &update)
        .await
        .expect("Failed to update variant");

    let saved = repo
        .get_variant_by_id(ctx, variant_id)
        .await
        .expect("Failed to get variant")
        .expect("Variant not found");

    assert_eq!(saved.barcode, Some("NEW_BARCODE".to_string()));
    assert_eq!(saved.name, Some("New Variant Name".to_string()));
    assert_eq!(saved.metadata, Some(json!({"new_sku": "SKU999"})));
}

/// Test: Update non-existent variant fails
pub async fn product_test_update_variant_not_found<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let update = ProductVariantUpdate {
        barcode: Update::Set("X".to_string()),
        name: Update::Unchanged,
        metadata: Update::Unchanged,
    };

    let result = repo.update_variant(ctx, 999999, &update).await;

    assert!(matches!(result, Err(Error::NotFound(_))));
}

/// Test: Delete variant success
pub async fn product_test_delete_variant_success<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let variant_id = super::generate_test_id().await;
    let variant = create_test_variant(product_id);

    repo.create_variant(ctx, variant_id, &variant)
        .await
        .expect("Failed to create variant");

    repo.delete_variant(ctx, variant_id)
        .await
        .expect("Failed to delete variant");

    let saved = repo
        .get_variant_by_id(ctx, variant_id)
        .await
        .expect("Failed to get variant");

    assert!(saved.is_none());
}

/// Test: Delete non-existent variant fails
pub async fn product_test_delete_variant_not_found<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let result = repo.delete_variant(ctx, 999999).await;

    assert!(matches!(result, Err(Error::NotFound(_))));
}

/// Test: Delete variants by product ID
pub async fn product_test_delete_variants_by_product_id<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    // Create multiple variants
    for i in 0..3 {
        let variant_id = super::generate_test_id().await;
        let variant = ProductVariantCreate {
            product_id,
            barcode: Some(format!("V{}", i)),
            name: None,
            metadata: None,
        };
        repo.create_variant(ctx, variant_id, &variant)
            .await
            .expect("Failed to create variant");
    }

    // Verify all variants exist
    let variants = repo
        .get_variant_by_product_id(ctx, product_id)
        .await
        .expect("Failed to get variants");
    assert_eq!(variants.len(), 3);

    // Delete all variants by product_id
    repo.delete_variants_by_product_id(ctx, product_id)
        .await
        .expect("Failed to delete variants");

    // Verify all variants are deleted
    let variants = repo
        .get_variant_by_product_id(ctx, product_id)
        .await
        .expect("Failed to get variants");
    assert_eq!(variants.len(), 0);
}

/// Test: Update deleted variant fails
pub async fn product_test_update_deleted_variant_fails<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let variant_id = super::generate_test_id().await;
    let variant = create_test_variant(product_id);

    repo.create_variant(ctx, variant_id, &variant)
        .await
        .expect("Failed to create variant");

    repo.delete_variant(ctx, variant_id)
        .await
        .expect("Failed to delete variant");

    let update = ProductVariantUpdate {
        barcode: Update::Set("SHOULD_FAIL".to_string()),
        name: Update::Unchanged,
        metadata: Update::Unchanged,
    };

    let result = repo.update_variant(ctx, variant_id, &update).await;

    assert!(matches!(result, Err(Error::NotFound(_))));
}

/// Test: Get deleted variant returns None
pub async fn product_test_get_deleted_variant_returns_none<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let variant_id = super::generate_test_id().await;
    let variant = create_test_variant(product_id);

    repo.create_variant(ctx, variant_id, &variant)
        .await
        .expect("Failed to create variant");

    repo.delete_variant(ctx, variant_id)
        .await
        .expect("Failed to delete variant");

    // Try to get by ID
    let by_id = repo
        .get_variant_by_id(ctx, variant_id)
        .await
        .expect("Failed to get variant");
    assert!(by_id.is_none());

    // Try to get by barcode
    let by_barcode = repo
        .get_variant_by_barcode(ctx, "1234567890")
        .await
        .expect("Failed to get variant");
    assert!(by_barcode.is_none());

    // Verify not in product variants list
    let variants = repo
        .get_variant_by_product_id(ctx, product_id)
        .await
        .expect("Failed to get variants");
    assert_eq!(variants.len(), 0);
}

/// Test: Update variant to clear metadata
pub async fn product_test_update_variant_clear_metadata<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let variant_id = super::generate_test_id().await;
    let variant = create_test_variant(product_id);

    repo.create_variant(ctx, variant_id, &variant)
        .await
        .expect("Failed to create variant");

    let update = ProductVariantUpdate {
        barcode: Update::Unchanged,
        name: Update::Unchanged,
        metadata: Update::Clear,
    };

    repo.update_variant(ctx, variant_id, &update)
        .await
        .expect("Failed to update variant");

    let saved = repo
        .get_variant_by_id(ctx, variant_id)
        .await
        .expect("Failed to get variant")
        .expect("Variant not found");

    assert_eq!(saved.metadata, None);
}

/// Test: Update variant to clear barcode
pub async fn product_test_update_variant_clear_barcode<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let variant_id = super::generate_test_id().await;
    let variant = create_test_variant(product_id);

    repo.create_variant(ctx, variant_id, &variant)
        .await
        .expect("Failed to create variant");

    let update = ProductVariantUpdate {
        barcode: Update::Clear,
        name: Update::Unchanged,
        metadata: Update::Unchanged,
    };

    repo.update_variant(ctx, variant_id, &update)
        .await
        .expect("Failed to update variant");

    let saved = repo
        .get_variant_by_id(ctx, variant_id)
        .await
        .expect("Failed to get variant")
        .expect("Variant not found");

    assert_eq!(saved.barcode, None);
}

/// Test: Update variant only name
pub async fn product_test_update_variant_only_name<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let variant_id = super::generate_test_id().await;
    let variant = create_test_variant(product_id);

    repo.create_variant(ctx, variant_id, &variant)
        .await
        .expect("Failed to create variant");

    let update = ProductVariantUpdate {
        barcode: Update::Unchanged,
        name: Update::Set("New Name".to_string()),
        metadata: Update::Unchanged,
    };

    repo.update_variant(ctx, variant_id, &update)
        .await
        .expect("Failed to update variant");

    let saved = repo
        .get_variant_by_id(ctx, variant_id)
        .await
        .expect("Failed to get variant")
        .expect("Variant not found");

    assert_eq!(saved.name, Some("New Name".to_string()));
    assert_eq!(saved.barcode, Some("1234567890".to_string())); // Unchanged
}

/// Test: Update variant only barcode
pub async fn product_test_update_variant_only_barcode<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let variant_id = super::generate_test_id().await;
    let variant = create_test_variant(product_id);

    repo.create_variant(ctx, variant_id, &variant)
        .await
        .expect("Failed to create variant");

    let update = ProductVariantUpdate {
        barcode: Update::Set("NEW-BARCODE-123".to_string()),
        name: Update::Unchanged,
        metadata: Update::Unchanged,
    };

    repo.update_variant(ctx, variant_id, &update)
        .await
        .expect("Failed to update variant");

    let saved = repo
        .get_variant_by_id(ctx, variant_id)
        .await
        .expect("Failed to get variant")
        .expect("Variant not found");

    assert_eq!(saved.barcode, Some("NEW-BARCODE-123".to_string()));
    assert_eq!(saved.name, Some("Default Variant".to_string())); // Unchanged
}

/// Test: Update variant set metadata
pub async fn product_test_update_variant_set_metadata<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let variant_id = super::generate_test_id().await;
    let variant = create_test_variant(product_id);

    repo.create_variant(ctx, variant_id, &variant)
        .await
        .expect("Failed to create variant");

    let update = ProductVariantUpdate {
        barcode: Update::Unchanged,
        name: Update::Unchanged,
        metadata: Update::Set(json!({"new": "data", "count": 42})),
    };

    repo.update_variant(ctx, variant_id, &update)
        .await
        .expect("Failed to update variant");

    let saved = repo
        .get_variant_by_id(ctx, variant_id)
        .await
        .expect("Failed to get variant")
        .expect("Variant not found");

    assert_eq!(saved.metadata, Some(json!({"new": "data", "count": 42})));
}

/// Test: Variant without barcode
pub async fn product_test_variant_without_barcode<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let variant_id = super::generate_test_id().await;
    let variant = ProductVariantCreate {
        product_id,
        barcode: None,
        name: Some("No Barcode Variant".to_string()),
        metadata: None,
    };

    repo.create_variant(ctx, variant_id, &variant)
        .await
        .expect("Failed to create variant");

    let saved = repo
        .get_variant_by_id(ctx, variant_id)
        .await
        .expect("Failed to get variant")
        .expect("Variant not found");

    assert_eq!(saved.barcode, None);
    assert_eq!(saved.name, Some("No Barcode Variant".to_string()));
}

/// Test: Multiple variants for single product
pub async fn product_test_multiple_variants_for_single_product<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    // Create 5 variants
    for i in 0..5 {
        let variant_id = super::generate_test_id().await;
        let variant = ProductVariantCreate {
            product_id,
            barcode: Some(format!("BARCODE{}", i)),
            name: Some(format!("Variant {}", i)),
            metadata: Some(json!({"index": i})),
        };

        repo.create_variant(ctx, variant_id, &variant)
            .await
            .expect("Failed to create variant");
    }

    // Get all variants
    let variants = repo
        .get_variant_by_product_id(ctx, product_id)
        .await
        .expect("Failed to get variants");

    assert_eq!(variants.len(), 5);
}

/// Test: Delete variants by product ID preserves other products
pub async fn product_test_delete_variants_preserves_other_products<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    // Create two products
    let product_id1 = super::generate_test_id().await;
    let product1 = create_test_product();

    repo.create_product(ctx, product_id1, &product1)
        .await
        .expect("Failed to create product1");

    let product_id2 = super::generate_test_id().await;
    let product2 = create_test_product();

    repo.create_product(ctx, product_id2, &product2)
        .await
        .expect("Failed to create product2");

    // Create variants for both products
    let variant_id1 = super::generate_test_id().await;
    let variant1 = create_test_variant(product_id1);

    repo.create_variant(ctx, variant_id1, &variant1)
        .await
        .expect("Failed to create variant1");

    let variant_id2 = super::generate_test_id().await;
    let variant2 = create_test_variant(product_id2);

    repo.create_variant(ctx, variant_id2, &variant2)
        .await
        .expect("Failed to create variant2");

    // Delete variants for product1 only
    repo.delete_variants_by_product_id(ctx, product_id1)
        .await
        .expect("Failed to delete variants");

    // Verify product1 variants are deleted
    let variants1 = repo
        .get_variant_by_product_id(ctx, product_id1)
        .await
        .expect("Failed to get variants");
    assert_eq!(variants1.len(), 0);

    // Verify product2 variants still exist
    let variants2 = repo
        .get_variant_by_product_id(ctx, product_id2)
        .await
        .expect("Failed to get variants");
    assert_eq!(variants2.len(), 1);
    assert_eq!(variants2[0].id, variant_id2);
}

// =============================================================================
// Variant Get Tests
// =============================================================================

/// Test: Get variant by barcode success
pub async fn product_test_get_variant_by_barcode_success<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let variant_id = super::generate_test_id().await;
    let unique_barcode = format!("BARCODE_{}", variant_id);
    let variant = ProductVariantCreate {
        product_id,
        barcode: Some(unique_barcode.clone()),
        name: Some("Barcode Test Variant".to_string()),
        metadata: None,
    };

    repo.create_variant(ctx, variant_id, &variant)
        .await
        .expect("Failed to create variant");

    let saved = repo
        .get_variant_by_barcode(ctx, &unique_barcode)
        .await
        .expect("Failed to get variant")
        .expect("Variant not found");

    assert_eq!(saved.id, variant_id);
    assert_eq!(saved.barcode, Some(unique_barcode));
}

/// Test: Get variant by barcode not found
pub async fn product_test_get_variant_by_barcode_not_found<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let result = repo
        .get_variant_by_barcode(ctx, "NONEXISTENT_BARCODE")
        .await
        .expect("Failed to query");

    assert!(result.is_none());
}

/// Test: Get variant by ID success
pub async fn product_test_get_variant_by_id_success<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let variant_id = super::generate_test_id().await;
    let variant = create_test_variant(product_id);

    repo.create_variant(ctx, variant_id, &variant)
        .await
        .expect("Failed to create variant");

    let result = repo
        .get_variant_by_id(ctx, variant_id)
        .await
        .expect("Failed to get variant");

    assert!(result.is_some());
    let saved_variant = result.unwrap();
    assert_eq!(saved_variant.id, variant_id);
    assert_eq!(saved_variant.barcode, Some("1234567890".to_string()));
}

/// Test: Get variant by ID not found
pub async fn product_test_get_variant_by_id_not_found<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let result = repo
        .get_variant_by_id(ctx, 999999)
        .await
        .expect("Failed to query");

    assert!(result.is_none());
}

/// Test: Get variant by product ID success
pub async fn product_test_get_variant_by_product_id_success<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    // Create multiple variants
    for i in 0..3 {
        let variant_id = super::generate_test_id().await;
        let variant = ProductVariantCreate {
            product_id,
            barcode: Some(format!("BC_{}", i)),
            name: Some(format!("Variant {}", i)),
            metadata: None,
        };
        repo.create_variant(ctx, variant_id, &variant)
            .await
            .expect("Failed to create variant");
    }

    let variants = repo
        .get_variant_by_product_id(ctx, product_id)
        .await
        .expect("Failed to get variants");

    assert_eq!(variants.len(), 3);
}

/// Test: Get variant by product ID returns empty list
pub async fn product_test_get_variant_by_product_id_empty<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    // Don't create any variants

    let variants = repo
        .get_variant_by_product_id(ctx, product_id)
        .await
        .expect("Failed to get variants");

    assert_eq!(variants.len(), 0);
}

/// Test: Get variant by product ID when product not found
pub async fn product_test_get_variant_by_product_id_product_not_found<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let variants = repo
        .get_variant_by_product_id(ctx, 999999)
        .await
        .expect("Failed to get variants");

    assert_eq!(variants.len(), 0);
}

/// Test: Get variant by ID when product is deleted
pub async fn product_test_get_variant_by_id_when_product_deleted<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let variant_id = super::generate_test_id().await;
    let variant = create_test_variant(product_id);

    repo.create_variant(ctx, variant_id, &variant)
        .await
        .expect("Failed to create variant");

    // Delete the product
    repo.delete_product(ctx, product_id)
        .await
        .expect("Failed to delete product");

    // Try to get variant - should return None because product is deleted
    let result = repo
        .get_variant_by_id(ctx, variant_id)
        .await
        .expect("Failed to get variant");

    assert!(result.is_none());
}

/// Test: Get variant by barcode when product is deleted
pub async fn product_test_get_variant_by_barcode_when_product_deleted<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let variant_id = super::generate_test_id().await;
    let variant = create_test_variant(product_id);

    repo.create_variant(ctx, variant_id, &variant)
        .await
        .expect("Failed to create variant");

    // Delete the product
    repo.delete_product(ctx, product_id)
        .await
        .expect("Failed to delete product");

    // Try to get variant by barcode - should return None because product is deleted
    let result = repo
        .get_variant_by_barcode(ctx, "1234567890")
        .await
        .expect("Failed to get variant");

    assert!(result.is_none());
}

// =============================================================================
// Get Variant IDs by Product ID Tests
// =============================================================================

/// Test: Get variant IDs returns correct IDs for multiple variants
pub async fn product_test_get_variant_ids_by_product_id_success<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let mut created_ids = Vec::new();
    for i in 0..3 {
        let variant_id = super::generate_test_id().await;
        created_ids.push(variant_id);
        let variant = ProductVariantCreate {
            product_id,
            barcode: Some(format!("IDS_BC_{}", i)),
            name: Some(format!("Variant {}", i)),
            metadata: None,
        };
        repo.create_variant(ctx, variant_id, &variant)
            .await
            .expect("Failed to create variant");
    }

    let ids = repo
        .get_variant_ids_by_product_id(ctx, product_id)
        .await
        .expect("Failed to get variant ids");

    assert_eq!(ids.len(), 3);
    for id in &created_ids {
        assert!(ids.contains(id));
    }
}

/// Test: Get variant IDs returns empty list when no variants exist
pub async fn product_test_get_variant_ids_by_product_id_empty<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let ids = repo
        .get_variant_ids_by_product_id(ctx, product_id)
        .await
        .expect("Failed to get variant ids");

    assert!(ids.is_empty());
}

/// Test: Get variant IDs excludes soft-deleted variants
pub async fn product_test_get_variant_ids_by_product_id_excludes_deleted<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    // Create 3 variants
    let mut variant_ids = Vec::new();
    for i in 0..3 {
        let variant_id = super::generate_test_id().await;
        variant_ids.push(variant_id);
        let variant = ProductVariantCreate {
            product_id,
            barcode: Some(format!("DEL_BC_{}", i)),
            name: Some(format!("Variant {}", i)),
            metadata: None,
        };
        repo.create_variant(ctx, variant_id, &variant)
            .await
            .expect("Failed to create variant");
    }

    // Delete the first variant
    repo.delete_variant(ctx, variant_ids[0])
        .await
        .expect("Failed to delete variant");

    let ids = repo
        .get_variant_ids_by_product_id(ctx, product_id)
        .await
        .expect("Failed to get variant ids");

    assert_eq!(ids.len(), 2);
    assert!(!ids.contains(&variant_ids[0]));
    assert!(ids.contains(&variant_ids[1]));
    assert!(ids.contains(&variant_ids[2]));
}

// =============================================================================
// Product Category Tests
// =============================================================================

/// Test: Add product categories successfully
pub async fn product_test_add_product_category_success<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    // First, create the categories that we'll link
    use crate::storage::sqlite::SqliteCategoryRepository;
    let category_repo = SqliteCategoryRepository::new();

    for category_id in 1..=3 {
        let category = CategoryCreate {
            parent_id: None,
            name: format!("Category {}", category_id),
            description: None,
        };
        category_repo
            .create(ctx, category_id, &category)
            .await
            .expect("Failed to create category");
    }

    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    // Add categories
    let category_ids = vec![1, 2, 3];
    repo.add_product_category(ctx, product_id, &category_ids)
        .await
        .expect("Failed to add product categories");

    // Verify categories were added
    let categories = repo
        .get_product_category(ctx, product_id)
        .await
        .expect("Failed to get product categories");

    assert_eq!(categories.len(), 3);
    assert!(categories.contains(&1));
    assert!(categories.contains(&2));
    assert!(categories.contains(&3));
}

/// Test: Add product categories with empty array is a no-op
pub async fn product_test_add_product_category_empty_array<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    // Add empty array
    repo.add_product_category(ctx, product_id, &[])
        .await
        .expect("Failed to add empty product categories");

    // Verify no categories were added
    let categories = repo
        .get_product_category(ctx, product_id)
        .await
        .expect("Failed to get product categories");

    assert!(categories.is_empty());
}

/// Test: Add product categories with duplicates handles gracefully
pub async fn product_test_add_product_category_duplicate<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    // First, create the categories that we'll link
    use crate::storage::sqlite::SqliteCategoryRepository;
    let category_repo = SqliteCategoryRepository::new();

    for category_id in 1..=4 {
        let category = CategoryCreate {
            parent_id: None,
            name: format!("Category {}", category_id),
            description: None,
        };
        category_repo
            .create(ctx, category_id, &category)
            .await
            .expect("Failed to create category");
    }

    let product_id = super::generate_test_id().await;
    let product = create_test_product();

    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    // Add categories first time
    let category_ids = vec![1, 2, 3];
    repo.add_product_category(ctx, product_id, &category_ids)
        .await
        .expect("Failed to add product categories");

    // Add some of the same categories again
    let duplicate_ids = vec![2, 3, 4];
    repo.add_product_category(ctx, product_id, &duplicate_ids)
        .await
        .expect("Failed to add duplicate product categories");

    // Verify we have 4 unique categories (1, 2, 3, 4)
    let categories = repo
        .get_product_category(ctx, product_id)
        .await
        .expect("Failed to get product categories");

    assert_eq!(categories.len(), 4);
    assert!(categories.contains(&1));
    assert!(categories.contains(&2));
    assert!(categories.contains(&3));
    assert!(categories.contains(&4));
}

// =============================================================================
// Variant Nested Data Tests (SellPrices and Discounts)
// =============================================================================

/// Test: get_variant_by_id fetches all nested data (Product, SellPrices, Discounts)
pub async fn product_test_get_variant_by_id_with_nested_data<R>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) where
    R: ProductRepository,
{
    // Create product
    let product_id = super::generate_test_id().await;
    let product = ProductCreate {
        name: "Test Product with Relations".to_string(),
        description: Some("Product for testing nested relations".to_string()),
        product_type: "product".to_string(),
        main_image: None,
        sellable: true,
        buyable: true,
        editable_price: false,
        metadata: None,
        variant_count: 0,
        category_ids: vec![],
    };
    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    // Create variant
    let variant_id = super::generate_test_id().await;
    let variant = ProductVariantCreate {
        product_id,
        barcode: Some("TEST-BARCODE-123".to_string()),
        name: Some("Variant with Prices".to_string()),
        metadata: None,
    };
    repo.create_variant(ctx, variant_id, &variant)
        .await
        .expect("Failed to create variant");

    // Create first sell price
    let price_id_1 = super::generate_test_id().await;
    let price_1 = SellPriceCreate {
        branch_id: Some(1),
        product_variant_id: variant_id,
        uom_id: 1,
        quantity: 1,
        price: 10000, // $100.00
        metadata: None,
    };
    repo.create_sell_price(ctx, price_id_1, &price_1)
        .await
        .expect("Failed to create first sell price");

    // Create discounts for first price
    let discount_id_1 = super::generate_test_id().await;
    let discount_1 = SellDiscountCreate {
        price_id: price_id_1,
        quantity: 10,
        discount_formula: "price * 0.9".to_string(),
        customer_level: Some(1),
        metadata: None,
    };
    repo.create_sell_discount(ctx, discount_id_1, &discount_1)
        .await
        .expect("Failed to create first discount");

    let discount_id_2 = super::generate_test_id().await;
    let discount_2 = SellDiscountCreate {
        price_id: price_id_1,
        quantity: 50,
        discount_formula: "price * 0.8".to_string(),
        customer_level: Some(2),
        metadata: None,
    };
    repo.create_sell_discount(ctx, discount_id_2, &discount_2)
        .await
        .expect("Failed to create second discount");

    // Create second sell price
    let price_id_2 = super::generate_test_id().await;
    let price_2 = SellPriceCreate {
        branch_id: Some(2),
        product_variant_id: variant_id,
        uom_id: 1,
        quantity: 1,
        price: 12000, // $120.00
        metadata: None,
    };
    repo.create_sell_price(ctx, price_id_2, &price_2)
        .await
        .expect("Failed to create second sell price");

    // Create discount for second price
    let discount_id_3 = super::generate_test_id().await;
    let discount_3 = SellDiscountCreate {
        price_id: price_id_2,
        quantity: 5,
        discount_formula: "price * 0.95".to_string(),
        customer_level: None,
        metadata: None,
    };
    repo.create_sell_discount(ctx, discount_id_3, &discount_3)
        .await
        .expect("Failed to create third discount");

    // Test get_variant_by_id
    let result = repo
        .get_variant_by_id(ctx, variant_id)
        .await
        .expect("Failed to get variant by id");

    assert!(result.is_some(), "Variant should be found");
    let fetched_variant = result.unwrap();

    // Verify variant data
    assert_eq!(fetched_variant.id, variant_id);
    assert_eq!(
        fetched_variant.barcode,
        Some("TEST-BARCODE-123".to_string())
    );
    assert_eq!(
        fetched_variant.name,
        Some("Variant with Prices".to_string())
    );

    // Verify sell prices are fetched
    assert_eq!(
        fetched_variant.sell_prices.len(),
        2,
        "Should have 2 sell prices"
    );

    // Find price 1 and verify its discounts
    let price_1_fetched = fetched_variant
        .sell_prices
        .iter()
        .find(|p| p.id == price_id_1)
        .expect("Price 1 should be present");
    assert_eq!(price_1_fetched.price, 10000);
    assert_eq!(price_1_fetched.branch_id, Some(1));
    assert_eq!(
        price_1_fetched.discounts.len(),
        2,
        "Price 1 should have 2 discounts"
    );

    // Verify discounts for price 1
    let discount_1_fetched = price_1_fetched
        .discounts
        .iter()
        .find(|d| d.id == discount_id_1)
        .expect("Discount 1 should be present");
    assert_eq!(discount_1_fetched.quantity, 10);
    assert_eq!(
        discount_1_fetched.discount_formula,
        Some("price * 0.9".to_string())
    );
    assert_eq!(discount_1_fetched.customer_level, Some(1));

    let discount_2_fetched = price_1_fetched
        .discounts
        .iter()
        .find(|d| d.id == discount_id_2)
        .expect("Discount 2 should be present");
    assert_eq!(discount_2_fetched.quantity, 50);
    assert_eq!(
        discount_2_fetched.discount_formula,
        Some("price * 0.8".to_string())
    );
    assert_eq!(discount_2_fetched.customer_level, Some(2));

    // Find price 2 and verify its discount
    let price_2_fetched = fetched_variant
        .sell_prices
        .iter()
        .find(|p| p.id == price_id_2)
        .expect("Price 2 should be present");
    assert_eq!(price_2_fetched.price, 12000);
    assert_eq!(price_2_fetched.branch_id, Some(2));
    assert_eq!(
        price_2_fetched.discounts.len(),
        1,
        "Price 2 should have 1 discount"
    );

    let discount_3_fetched = price_2_fetched
        .discounts
        .iter()
        .find(|d| d.id == discount_id_3)
        .expect("Discount 3 should be present");
    assert_eq!(discount_3_fetched.quantity, 5);
    assert_eq!(
        discount_3_fetched.discount_formula,
        Some("price * 0.95".to_string())
    );
    assert_eq!(discount_3_fetched.customer_level, None);
}

/// Test: get_variant_by_barcode fetches all nested data (Product, SellPrices, Discounts)
pub async fn product_test_get_variant_by_barcode_with_nested_data<R>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) where
    R: ProductRepository,
{
    // Create product
    let product_id = super::generate_test_id().await;
    let product = ProductCreate {
        name: "Barcode Test Product".to_string(),
        description: Some("Product for testing barcode lookup".to_string()),
        product_type: "product".to_string(),
        main_image: None,
        sellable: true,
        buyable: true,
        editable_price: false,
        metadata: None,
        variant_count: 0,
        category_ids: vec![],
    };
    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    // Create variant
    let variant_id = super::generate_test_id().await;
    let variant = ProductVariantCreate {
        product_id,
        barcode: Some("BARCODE-XYZ-789".to_string()),
        name: Some("Barcode Variant".to_string()),
        metadata: None,
    };
    repo.create_variant(ctx, variant_id, &variant)
        .await
        .expect("Failed to create variant");

    // Create sell price
    let price_id = super::generate_test_id().await;
    let price = SellPriceCreate {
        branch_id: None,
        product_variant_id: variant_id,
        uom_id: 2,
        quantity: 1,
        price: 25000, // $250.00
        metadata: None,
    };
    repo.create_sell_price(ctx, price_id, &price)
        .await
        .expect("Failed to create sell price");

    // Create discount
    let discount_id = super::generate_test_id().await;
    let discount = SellDiscountCreate {
        price_id,
        quantity: 100,
        discount_formula: "price * 0.7".to_string(),
        customer_level: Some(3),
        metadata: None,
    };
    repo.create_sell_discount(ctx, discount_id, &discount)
        .await
        .expect("Failed to create discount");

    // Test get_variant_by_barcode
    let result = repo
        .get_variant_by_barcode(ctx, "BARCODE-XYZ-789")
        .await
        .expect("Failed to get variant by barcode");

    assert!(result.is_some(), "Variant should be found by barcode");
    let fetched_variant = result.unwrap();

    // Verify variant data
    assert_eq!(fetched_variant.id, variant_id);
    assert_eq!(fetched_variant.barcode, Some("BARCODE-XYZ-789".to_string()));
    assert_eq!(fetched_variant.name, Some("Barcode Variant".to_string()));

    // Verify sell prices are fetched
    assert_eq!(
        fetched_variant.sell_prices.len(),
        1,
        "Should have 1 sell price"
    );

    let fetched_price = &fetched_variant.sell_prices[0];
    assert_eq!(fetched_price.id, price_id);
    assert_eq!(fetched_price.price, 25000);
    assert_eq!(fetched_price.branch_id, None);
    assert_eq!(fetched_price.uom_id, 2);

    // Verify discounts are fetched
    assert_eq!(fetched_price.discounts.len(), 1, "Should have 1 discount");

    let fetched_discount = &fetched_price.discounts[0];
    assert_eq!(fetched_discount.id, discount_id);
    assert_eq!(fetched_discount.quantity, 100);
    assert_eq!(
        fetched_discount.discount_formula,
        Some("price * 0.7".to_string())
    );
    assert_eq!(fetched_discount.customer_level, Some(3));
}

/// Test: Soft-deleted prices and discounts are excluded from variant results
pub async fn product_test_get_variant_excludes_soft_deleted_relations<R>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) where
    R: ProductRepository,
{
    // Create product and variant
    let product_id = super::generate_test_id().await;
    let product = ProductCreate {
        name: "Soft Delete Test Product".to_string(),
        description: None,
        product_type: "product".to_string(),
        main_image: None,
        sellable: true,
        buyable: true,
        editable_price: false,
        metadata: None,
        variant_count: 0,
        category_ids: vec![],
    };
    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let variant_id = super::generate_test_id().await;
    let variant = ProductVariantCreate {
        product_id,
        barcode: Some("SOFT-DELETE-TEST".to_string()),
        name: Some("Soft Delete Variant".to_string()),
        metadata: None,
    };
    repo.create_variant(ctx, variant_id, &variant)
        .await
        .expect("Failed to create variant");

    // Create two prices with different branches
    let price_id_1 = super::generate_test_id().await;
    let price_1 = SellPriceCreate {
        branch_id: Some(1), // Branch 1
        product_variant_id: variant_id,
        uom_id: 1,
        quantity: 1,
        price: 1000,
        metadata: None,
    };
    repo.create_sell_price(ctx, price_id_1, &price_1)
        .await
        .expect("Failed to create price 1");

    let price_id_2 = super::generate_test_id().await;
    let price_2 = SellPriceCreate {
        branch_id: Some(2), // Branch 2
        product_variant_id: variant_id,
        uom_id: 1,
        quantity: 1,
        price: 2000,
        metadata: None,
    };
    repo.create_sell_price(ctx, price_id_2, &price_2)
        .await
        .expect("Failed to create price 2");

    // Create discounts for both prices
    let discount_id_1 = super::generate_test_id().await;
    let discount_1 = SellDiscountCreate {
        price_id: price_id_1,
        quantity: 10,
        discount_formula: "formula1".to_string(),
        customer_level: None,
        metadata: None,
    };
    repo.create_sell_discount(ctx, discount_id_1, &discount_1)
        .await
        .expect("Failed to create discount 1");

    let discount_id_2 = super::generate_test_id().await;
    let discount_2 = SellDiscountCreate {
        price_id: price_id_2,
        quantity: 20,
        discount_formula: "formula2".to_string(),
        customer_level: None,
        metadata: None,
    };
    repo.create_sell_discount(ctx, discount_id_2, &discount_2)
        .await
        .expect("Failed to create discount 2");

    // Fetch and verify all data is present
    let result = repo
        .get_variant_by_id(ctx, variant_id)
        .await
        .expect("Failed to get variant");
    let variant_before = result.unwrap();
    assert_eq!(variant_before.sell_prices.len(), 2);
    assert_eq!(variant_before.sell_prices[0].discounts.len(), 1);
    assert_eq!(variant_before.sell_prices[1].discounts.len(), 1);

    // Soft delete price 1
    repo.delete_sell_price(ctx, price_id_1)
        .await
        .expect("Failed to delete price 1");

    // Soft delete discount 2 (price 2 is still active)
    repo.delete_sell_discount(ctx, discount_id_2)
        .await
        .expect("Failed to delete discount 2");

    // Fetch again and verify deleted items are excluded
    let result = repo
        .get_variant_by_id(ctx, variant_id)
        .await
        .expect("Failed to get variant after deletions");
    let variant_after = result.unwrap();

    // Should only have price 2 now (price 1 was deleted)
    assert_eq!(
        variant_after.sell_prices.len(),
        1,
        "Should only have 1 active price"
    );
    assert_eq!(
        variant_after.sell_prices[0].id, price_id_2,
        "Should be price 2"
    );

    // Price 2 should have no discounts (discount 2 was deleted)
    assert_eq!(
        variant_after.sell_prices[0].discounts.len(),
        0,
        "Price 2 should have no active discounts"
    );
}

// =============================================================================
// Product get_by_id Full Data Tests
// =============================================================================

/// Test: get_by_id returns product with populated categories
pub async fn product_test_get_by_id_with_categories<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    use crate::storage::sqlite::SqliteCategoryRepository;
    let category_repo = SqliteCategoryRepository::new();

    // Create categories
    let cat_id_1 = super::generate_test_id().await;
    let cat_id_2 = super::generate_test_id().await;
    category_repo
        .create(
            ctx,
            cat_id_1,
            &CategoryCreate {
                parent_id: None,
                name: "Electronics".to_string(),
                description: Some("Electronic devices".to_string()),
            },
        )
        .await
        .expect("Failed to create category 1");
    category_repo
        .create(
            ctx,
            cat_id_2,
            &CategoryCreate {
                parent_id: None,
                name: "Accessories".to_string(),
                description: None,
            },
        )
        .await
        .expect("Failed to create category 2");

    // Create product and link categories
    let product_id = super::generate_test_id().await;
    let product = create_test_product();
    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");
    repo.add_product_category(ctx, product_id, &[cat_id_1, cat_id_2])
        .await
        .expect("Failed to add categories");

    // Fetch product
    let saved = repo
        .get_by_id(ctx, product_id)
        .await
        .expect("Failed to get product")
        .expect("Product not found");

    assert_eq!(saved.categories.len(), 2, "Should have 2 categories");

    let cat_1 = saved
        .categories
        .iter()
        .find(|c| c.id == cat_id_1)
        .expect("Category 1 should be present");
    assert_eq!(cat_1.name, "Electronics");
    assert_eq!(cat_1.description, Some("Electronic devices".to_string()));

    let cat_2 = saved
        .categories
        .iter()
        .find(|c| c.id == cat_id_2)
        .expect("Category 2 should be present");
    assert_eq!(cat_2.name, "Accessories");
    assert_eq!(cat_2.description, None);
}

/// Test: get_by_id returns product with populated variants including sell prices and discounts
pub async fn product_test_get_by_id_with_variants_and_sell_prices<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    // Create product
    let product_id = super::generate_test_id().await;
    let product = ProductCreate {
        name: "Product with Variants".to_string(),
        description: None,
        product_type: "product".to_string(),
        main_image: None,
        sellable: true,
        buyable: true,
        editable_price: false,
        metadata: None,
        variant_count: 0,
        category_ids: vec![],
    };
    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    // Create variant 1
    let variant_id_1 = super::generate_test_id().await;
    let variant_1 = ProductVariantCreate {
        product_id,
        barcode: Some("VAR-001".to_string()),
        name: Some("Small".to_string()),
        metadata: None,
    };
    repo.create_variant(ctx, variant_id_1, &variant_1)
        .await
        .expect("Failed to create variant 1");

    // Create variant 2
    let variant_id_2 = super::generate_test_id().await;
    let variant_2 = ProductVariantCreate {
        product_id,
        barcode: Some("VAR-002".to_string()),
        name: Some("Large".to_string()),
        metadata: None,
    };
    repo.create_variant(ctx, variant_id_2, &variant_2)
        .await
        .expect("Failed to create variant 2");

    // Create sell price for variant 1
    let price_id_1 = super::generate_test_id().await;
    let price_1 = SellPriceCreate {
        branch_id: None,
        product_variant_id: variant_id_1,
        uom_id: 1,
        quantity: 1,
        price: 5000,
        metadata: None,
    };
    repo.create_sell_price(ctx, price_id_1, &price_1)
        .await
        .expect("Failed to create sell price 1");

    // Create discount for sell price 1
    let discount_id = super::generate_test_id().await;
    let discount = SellDiscountCreate {
        price_id: price_id_1,
        quantity: 10,
        discount_formula: "price * 0.9".to_string(),
        customer_level: Some(1),
        metadata: None,
    };
    repo.create_sell_discount(ctx, discount_id, &discount)
        .await
        .expect("Failed to create discount");

    // Create sell price for variant 2
    let price_id_2 = super::generate_test_id().await;
    let price_2 = SellPriceCreate {
        branch_id: None,
        product_variant_id: variant_id_2,
        uom_id: 1,
        quantity: 1,
        price: 8000,
        metadata: None,
    };
    repo.create_sell_price(ctx, price_id_2, &price_2)
        .await
        .expect("Failed to create sell price 2");

    // Fetch product by ID
    let saved = repo
        .get_by_id(ctx, product_id)
        .await
        .expect("Failed to get product")
        .expect("Product not found");

    assert_eq!(saved.variants.len(), 2, "Should have 2 variants");

    // Verify variant 1
    let v1 = saved
        .variants
        .iter()
        .find(|v| v.id == variant_id_1)
        .expect("Variant 1 should be present");
    assert_eq!(v1.barcode, Some("VAR-001".to_string()));
    assert_eq!(v1.name, Some("Small".to_string()));
    assert_eq!(
        v1.sell_prices.len(),
        1,
        "Variant 1 should have 1 sell price"
    );
    assert_eq!(v1.sell_prices[0].id, price_id_1);
    assert_eq!(v1.sell_prices[0].price, 5000);
    assert_eq!(
        v1.sell_prices[0].discounts.len(),
        1,
        "Sell price 1 should have 1 discount"
    );
    assert_eq!(v1.sell_prices[0].discounts[0].id, discount_id);
    assert_eq!(v1.sell_prices[0].discounts[0].quantity, 10);

    // Verify variant 2
    let v2 = saved
        .variants
        .iter()
        .find(|v| v.id == variant_id_2)
        .expect("Variant 2 should be present");
    assert_eq!(v2.barcode, Some("VAR-002".to_string()));
    assert_eq!(v2.name, Some("Large".to_string()));
    assert_eq!(
        v2.sell_prices.len(),
        1,
        "Variant 2 should have 1 sell price"
    );
    assert_eq!(v2.sell_prices[0].id, price_id_2);
    assert_eq!(v2.sell_prices[0].price, 8000);
    assert!(
        v2.sell_prices[0].discounts.is_empty(),
        "Sell price 2 should have no discounts"
    );
}

/// Test: get_by_id returns product with both categories and variants fully populated
pub async fn product_test_get_by_id_with_full_data<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    use crate::storage::sqlite::SqliteCategoryRepository;
    let category_repo = SqliteCategoryRepository::new();

    // Create category
    let cat_id = super::generate_test_id().await;
    category_repo
        .create(
            ctx,
            cat_id,
            &CategoryCreate {
                parent_id: None,
                name: "Full Data Category".to_string(),
                description: None,
            },
        )
        .await
        .expect("Failed to create category");

    // Create product
    let product_id = super::generate_test_id().await;
    let product = ProductCreate {
        name: "Full Data Product".to_string(),
        description: Some("Product with everything".to_string()),
        product_type: "product".to_string(),
        main_image: None,
        sellable: true,
        buyable: true,
        editable_price: false,
        metadata: None,
        variant_count: 0,
        category_ids: vec![],
    };
    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    // Link category
    repo.add_product_category(ctx, product_id, &[cat_id])
        .await
        .expect("Failed to add category");

    // Create variant
    let variant_id = super::generate_test_id().await;
    let variant = ProductVariantCreate {
        product_id,
        barcode: Some("FULL-001".to_string()),
        name: Some("Full Variant".to_string()),
        metadata: None,
    };
    repo.create_variant(ctx, variant_id, &variant)
        .await
        .expect("Failed to create variant");

    // Create sell price with discount
    let price_id = super::generate_test_id().await;
    let price = SellPriceCreate {
        branch_id: Some(1),
        product_variant_id: variant_id,
        uom_id: 1,
        quantity: 1,
        price: 15000,
        metadata: None,
    };
    repo.create_sell_price(ctx, price_id, &price)
        .await
        .expect("Failed to create sell price");

    let discount_id = super::generate_test_id().await;
    let discount = SellDiscountCreate {
        price_id,
        quantity: 5,
        discount_formula: "price * 0.85".to_string(),
        customer_level: None,
        metadata: None,
    };
    repo.create_sell_discount(ctx, discount_id, &discount)
        .await
        .expect("Failed to create discount");

    // Fetch product by ID
    let saved = repo
        .get_by_id(ctx, product_id)
        .await
        .expect("Failed to get product")
        .expect("Product not found");

    // Verify product fields
    assert_eq!(saved.id, product_id);
    assert_eq!(saved.name, "Full Data Product");

    // Verify categories
    assert_eq!(saved.categories.len(), 1);
    assert_eq!(saved.categories[0].id, cat_id);
    assert_eq!(saved.categories[0].name, "Full Data Category");

    // Verify variants
    assert_eq!(saved.variants.len(), 1);
    assert_eq!(saved.variants[0].id, variant_id);
    assert_eq!(saved.variants[0].barcode, Some("FULL-001".to_string()));

    // Verify sell prices
    assert_eq!(saved.variants[0].sell_prices.len(), 1);
    assert_eq!(saved.variants[0].sell_prices[0].price, 15000);
    assert_eq!(saved.variants[0].sell_prices[0].branch_id, Some(1));

    // Verify discounts
    assert_eq!(saved.variants[0].sell_prices[0].discounts.len(), 1);
    assert_eq!(
        saved.variants[0].sell_prices[0].discounts[0].id,
        discount_id
    );
    assert_eq!(saved.variants[0].sell_prices[0].discounts[0].quantity, 5);
}

/// Test: get_by_id excludes soft-deleted categories
pub async fn product_test_get_by_id_excludes_deleted_categories<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    use crate::storage::CategoryRepository;
    use crate::storage::sqlite::SqliteCategoryRepository;
    let category_repo = SqliteCategoryRepository::new();

    // Create 2 categories
    let cat_id_1 = super::generate_test_id().await;
    let cat_id_2 = super::generate_test_id().await;
    category_repo
        .create(
            ctx,
            cat_id_1,
            &CategoryCreate {
                parent_id: None,
                name: "Active Category".to_string(),
                description: None,
            },
        )
        .await
        .expect("Failed to create category 1");
    category_repo
        .create(
            ctx,
            cat_id_2,
            &CategoryCreate {
                parent_id: None,
                name: "Deleted Category".to_string(),
                description: None,
            },
        )
        .await
        .expect("Failed to create category 2");

    // Create product and link both categories
    let product_id = super::generate_test_id().await;
    let product = create_test_product();
    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");
    repo.add_product_category(ctx, product_id, &[cat_id_1, cat_id_2])
        .await
        .expect("Failed to add categories");

    // Soft-delete category 2
    category_repo
        .delete(ctx, cat_id_2)
        .await
        .expect("Failed to delete category 2");

    // Fetch product
    let saved = repo
        .get_by_id(ctx, product_id)
        .await
        .expect("Failed to get product")
        .expect("Product not found");

    // Only active category should be returned
    assert_eq!(
        saved.categories.len(),
        1,
        "Should only have 1 active category"
    );
    assert_eq!(saved.categories[0].id, cat_id_1);
    assert_eq!(saved.categories[0].name, "Active Category");
}

/// Test: get_by_id excludes soft-deleted variants
pub async fn product_test_get_by_id_excludes_deleted_variants<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    // Create product
    let product_id = super::generate_test_id().await;
    let product = create_test_product();
    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    // Create 2 variants
    let variant_id_1 = super::generate_test_id().await;
    let variant_id_2 = super::generate_test_id().await;
    repo.create_variant(
        ctx,
        variant_id_1,
        &ProductVariantCreate {
            product_id,
            barcode: Some("ACTIVE-VAR".to_string()),
            name: Some("Active Variant".to_string()),
            metadata: None,
        },
    )
    .await
    .expect("Failed to create variant 1");
    repo.create_variant(
        ctx,
        variant_id_2,
        &ProductVariantCreate {
            product_id,
            barcode: Some("DELETED-VAR".to_string()),
            name: Some("Deleted Variant".to_string()),
            metadata: None,
        },
    )
    .await
    .expect("Failed to create variant 2");

    // Verify both variants are returned initially
    let saved = repo
        .get_by_id(ctx, product_id)
        .await
        .expect("Failed to get product")
        .expect("Product not found");
    assert_eq!(
        saved.variants.len(),
        2,
        "Should have 2 variants before delete"
    );

    // Soft-delete variant 2
    repo.delete_variant(ctx, variant_id_2)
        .await
        .expect("Failed to delete variant 2");

    // Fetch product again
    let saved = repo
        .get_by_id(ctx, product_id)
        .await
        .expect("Failed to get product")
        .expect("Product not found");

    // Only active variant should be returned
    assert_eq!(saved.variants.len(), 1, "Should only have 1 active variant");
    assert_eq!(saved.variants[0].id, variant_id_1);
    assert_eq!(saved.variants[0].name, Some("Active Variant".to_string()));
}

/// Test: get_by_id returns empty categories and variants when none are linked
pub async fn product_test_get_by_id_empty_categories_and_variants<R: ProductRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let product_id = super::generate_test_id().await;
    let product = create_test_product();
    repo.create_product(ctx, product_id, &product)
        .await
        .expect("Failed to create product");

    let saved = repo
        .get_by_id(ctx, product_id)
        .await
        .expect("Failed to get product")
        .expect("Product not found");

    assert!(saved.categories.is_empty(), "Categories should be empty");
    assert!(saved.variants.is_empty(), "Variants should be empty");
}
