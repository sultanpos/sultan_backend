use sea_orm::DatabaseConnection;
use serde_json::json;

use crate::{
    domain::{
        error::Error,
        model::{
            Update,
            product::{ProductCreate, ProductUpdate, ProductVariantCreate, ProductVariantUpdate},
        },
    },
    storage::{ProductRepository, RepoCtx},
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
        has_variant: false,
        metadata: Some(json!({"key": "value"})),
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
    assert!(!saved.has_variant);
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
        has_variant: true,
        metadata: None,
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
    assert!(saved.has_variant);
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
        has_variant: None,
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
        has_variant: None,
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
        has_variant: Some(true),
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
    assert!(saved.has_variant);
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
        has_variant: None,
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
        has_variant: None,
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
        has_variant: false,
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
        has_variant: None,
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
        has_variant: None,
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
        has_variant: Some(true),
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
    assert!(saved.has_variant);
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
        has_variant: None,
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
        has_variant: None,
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
        has_variant: None,
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
    assert_eq!(saved.product.id, product_id);
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

    // Verify each variant belongs to the same product
    for variant in &variants {
        assert_eq!(variant.product.id, product_id);
    }
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
    assert_eq!(saved.product.id, product_id);
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
    assert_eq!(saved_variant.product.id, product_id);
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
    for variant in &variants {
        assert_eq!(variant.product.id, product_id);
    }
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
