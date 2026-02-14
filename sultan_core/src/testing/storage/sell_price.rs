use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};

use crate::{
    domain::model::{
        Update,
        sell_price::{SellDiscountCreate, SellDiscountUpdate, SellPriceCreate, SellPriceUpdate},
    },
    storage::{RepoCtx, sell_price_repo::SellPriceRepository},
};

/// Run all SellPrice repository tests.
///
/// This function runs a comprehensive suite of tests for the SellPriceRepository
/// implementation. Each test receives a fresh RepoCtx from the ctx_factory.
///
/// # Arguments
///
/// * `repo` - The SellPriceRepository implementation to test
/// * `ctx_factory` - A factory function that creates new RepoCtx instances
pub async fn sell_price_test_all<C, F, Fut>(repo: &C, ctx_factory: F)
where
    C: SellPriceRepository,
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = RepoCtx<DatabaseConnection>>,
{
    sell_price_test_repo_integration(&ctx_factory().await, repo).await;
    sell_price_test_get_by_id_not_found(&ctx_factory().await, repo).await;
    sell_price_test_update_not_found(&ctx_factory().await, repo).await;
    sell_price_test_delete_not_found(&ctx_factory().await, repo).await;
    sell_price_test_get_deleted(&ctx_factory().await, repo).await;
    sell_price_test_partial_update(&ctx_factory().await, repo).await;
    sell_price_test_get_all_by_product_variant_id(&ctx_factory().await, repo).await;

    // Discount tests
    sell_discount_test_repo_integration(&ctx_factory().await, repo).await;
    sell_discount_test_get_by_id_not_found(&ctx_factory().await, repo).await;
    sell_discount_test_update_not_found(&ctx_factory().await, repo).await;
    sell_discount_test_delete_not_found(&ctx_factory().await, repo).await;
    sell_discount_test_get_deleted(&ctx_factory().await, repo).await;
    sell_discount_test_partial_update(&ctx_factory().await, repo).await;
    sell_discount_test_delete_by_sell_price_id(&ctx_factory().await, repo).await;

    // Cross-cutting tests
    sell_price_test_delete_by_product_variant_ids(&ctx_factory().await, repo).await;
    sell_price_test_delete_by_product_variant_ids_empty(&ctx_factory().await, repo).await;
    sell_price_test_delete_by_product_variant_ids_with_discounts(&ctx_factory().await, repo).await;
}

/// Creates a test product and product_variant in the database.
/// Returns the product_variant_id that can be used for sell_price tests.
async fn create_test_product_variant(ctx: &RepoCtx<DatabaseConnection>) -> i64 {
    let product_id = super::generate_test_id().await;
    let variant_id = super::generate_test_id().await;

    // Insert a product first
    ctx.db
        .execute_raw(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            "INSERT INTO products (id, name, product_type, sellable, buyable) VALUES (?, 'Test Product', 'product', 1, 1)",
            vec![product_id.into()],
        ))
        .await
        .expect("Failed to insert test product");

    // Insert a product_variant
    ctx.db
        .execute_raw(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            "INSERT INTO product_variants (id, product_id, name) VALUES (?, ?, 'Test Variant')",
            vec![variant_id.into(), product_id.into()],
        ))
        .await
        .expect("Failed to insert test product variant");

    variant_id
}

// =============================================================================
// SellPrice Tests
// =============================================================================

pub async fn sell_price_test_repo_integration<P: SellPriceRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &P,
) {
    let id = super::generate_test_id().await;
    let product_variant_id = create_test_product_variant(ctx).await;
    let uom_id = super::generate_test_id().await;

    let price = SellPriceCreate {
        branch_id: None,
        product_variant_id,
        uom_id,
        quantity: 1,
        price: 10000,
        metadata: None,
    };

    // Test Create
    repo.create(ctx, id, &price)
        .await
        .expect("Failed to create sell price");

    // Test Get By ID
    let fetched = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get sell price")
        .expect("SellPrice not found");
    assert_eq!(fetched.id, id);
    assert_eq!(fetched.product_variant_id, product_variant_id);
    assert_eq!(fetched.quantity, 1);
    assert_eq!(fetched.price, 10000);

    // Test Update
    let update_data = SellPriceUpdate {
        uom_id: None,
        quantity: Some(5),
        price: Some(15000),
        metadata: Update::Unchanged,
    };
    repo.update(ctx, id, &update_data)
        .await
        .expect("Failed to update sell price");

    let fetched_updated = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get updated sell price")
        .expect("Updated sell price not found");
    assert_eq!(fetched_updated.quantity, 5);
    assert_eq!(fetched_updated.price, 15000);

    // Test Delete
    repo.delete(ctx, id)
        .await
        .expect("Failed to delete sell price");
    let deleted = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get deleted sell price");
    assert!(deleted.is_none());
}

pub async fn sell_price_test_get_by_id_not_found<P: SellPriceRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &P,
) {
    let non_existent_id = super::generate_test_id().await;
    let result = repo.get_by_id(ctx, non_existent_id).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

pub async fn sell_price_test_update_not_found<P: SellPriceRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &P,
) {
    let non_existent_id = super::generate_test_id().await;
    let update_data = SellPriceUpdate {
        uom_id: None,
        quantity: Some(10),
        price: None,
        metadata: Update::Unchanged,
    };
    let result = repo.update(ctx, non_existent_id, &update_data).await;
    assert!(result.is_err());
}

pub async fn sell_price_test_delete_not_found<P: SellPriceRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &P,
) {
    let non_existent_id = super::generate_test_id().await;
    let result = repo.delete(ctx, non_existent_id).await;
    assert!(result.is_err());
}

pub async fn sell_price_test_get_deleted<P: SellPriceRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &P,
) {
    let id = super::generate_test_id().await;
    let product_variant_id = create_test_product_variant(ctx).await;
    let uom_id = super::generate_test_id().await;

    let price = SellPriceCreate {
        branch_id: None,
        product_variant_id,
        uom_id,
        quantity: 1,
        price: 5000,
        metadata: None,
    };

    repo.create(ctx, id, &price)
        .await
        .expect("Failed to create sell price");
    repo.delete(ctx, id)
        .await
        .expect("Failed to delete sell price");

    // Should not find deleted price
    let result = repo.get_by_id(ctx, id).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

pub async fn sell_price_test_partial_update<P: SellPriceRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &P,
) {
    let id = super::generate_test_id().await;
    let product_variant_id = create_test_product_variant(ctx).await;
    let uom_id = super::generate_test_id().await;

    let price = SellPriceCreate {
        branch_id: Some(12345),
        product_variant_id,
        uom_id,
        quantity: 10,
        price: 50000,
        metadata: Some(serde_json::json!({"note": "original"})),
    };

    repo.create(ctx, id, &price)
        .await
        .expect("Failed to create sell price");

    // Partial update: only update price
    let partial_update = SellPriceUpdate {
        uom_id: None,
        quantity: None,
        price: Some(75000),
        metadata: Update::Unchanged,
    };
    repo.update(ctx, id, &partial_update)
        .await
        .expect("Failed to update sell price");

    let fetched = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get sell price")
        .expect("SellPrice not found");
    // Price should be updated
    assert_eq!(fetched.price, 75000);
    // Other fields should remain unchanged
    assert_eq!(fetched.quantity, 10);
}

pub async fn sell_price_test_get_all_by_product_variant_id<P: SellPriceRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &P,
) {
    let product_variant_id = create_test_product_variant(ctx).await;

    // Create multiple prices for the same product variant
    // Note: Since there's a unique constraint on (product_variant_id, COALESCE(branch_id, 0)),
    // we need to use different branch_ids for each price
    for i in 0..3 {
        let id = super::generate_test_id().await;
        let uom_id = super::generate_test_id().await;
        let price = SellPriceCreate {
            branch_id: Some(i + 1), // Different branch_id for each
            product_variant_id,
            uom_id,
            quantity: i + 1,
            price: (i + 1) * 10000,
            metadata: None,
        };
        repo.create(ctx, id, &price)
            .await
            .expect("Failed to create sell price");
    }

    let prices = repo
        .get_all_by_product_variant_id(ctx, product_variant_id)
        .await
        .expect("Failed to get all prices");
    assert_eq!(prices.len(), 3);
}

// =============================================================================
// SellDiscount Tests
// =============================================================================

async fn create_test_sell_price<P: SellPriceRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &P,
) -> i64 {
    let id = super::generate_test_id().await;
    let product_variant_id = create_test_product_variant(ctx).await;
    let uom_id = super::generate_test_id().await;

    let price = SellPriceCreate {
        branch_id: None,
        product_variant_id,
        uom_id,
        quantity: 1,
        price: 10000,
        metadata: None,
    };
    repo.create(ctx, id, &price)
        .await
        .expect("Failed to create sell price");
    id
}

pub async fn sell_discount_test_repo_integration<P: SellPriceRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &P,
) {
    let sell_price_id = create_test_sell_price(ctx, repo).await;
    let id = super::generate_test_id().await;

    let discount = SellDiscountCreate {
        price_id: sell_price_id,
        quantity: 10,
        discount_formula: "-10%".to_string(),
        customer_level: Some(1),
        metadata: None,
    };

    // Test Create
    repo.create_discount(ctx, id, &discount)
        .await
        .expect("Failed to create discount");

    // Test Get By ID
    let fetched = repo
        .get_discount_by_id(ctx, id)
        .await
        .expect("Failed to get discount")
        .expect("Discount not found");
    assert_eq!(fetched.id, id);
    assert_eq!(fetched.sell_price_id, sell_price_id);
    assert_eq!(fetched.quantity, 10);

    // Test Update
    let update_data = SellDiscountUpdate {
        quantity: Some(20),
        discount_formula: Some("-15%".to_string()),
        customer_level: Update::Unchanged,
        metadata: Update::Unchanged,
    };
    repo.update_discount(ctx, id, &update_data)
        .await
        .expect("Failed to update discount");

    let fetched_updated = repo
        .get_discount_by_id(ctx, id)
        .await
        .expect("Failed to get updated discount")
        .expect("Updated discount not found");
    assert_eq!(fetched_updated.quantity, 20);
    assert_eq!(fetched_updated.discount_formula, Some("-15%".to_string()));

    // Test Delete
    repo.delete_discount(ctx, id)
        .await
        .expect("Failed to delete discount");
    let deleted = repo
        .get_discount_by_id(ctx, id)
        .await
        .expect("Failed to get deleted discount");
    assert!(deleted.is_none());
}

pub async fn sell_discount_test_get_by_id_not_found<P: SellPriceRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &P,
) {
    let non_existent_id = super::generate_test_id().await;
    let result = repo.get_discount_by_id(ctx, non_existent_id).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

pub async fn sell_discount_test_update_not_found<P: SellPriceRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &P,
) {
    let non_existent_id = super::generate_test_id().await;
    let update_data = SellDiscountUpdate {
        quantity: Some(10),
        discount_formula: None,
        customer_level: Update::Unchanged,
        metadata: Update::Unchanged,
    };
    let result = repo
        .update_discount(ctx, non_existent_id, &update_data)
        .await;
    assert!(result.is_err());
}

pub async fn sell_discount_test_delete_not_found<P: SellPriceRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &P,
) {
    let non_existent_id = super::generate_test_id().await;
    let result = repo.delete_discount(ctx, non_existent_id).await;
    assert!(result.is_err());
}

pub async fn sell_discount_test_get_deleted<P: SellPriceRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &P,
) {
    let sell_price_id = create_test_sell_price(ctx, repo).await;
    let id = super::generate_test_id().await;

    let discount = SellDiscountCreate {
        price_id: sell_price_id,
        quantity: 5,
        discount_formula: "-5%".to_string(),
        customer_level: None,
        metadata: None,
    };

    repo.create_discount(ctx, id, &discount)
        .await
        .expect("Failed to create discount");
    repo.delete_discount(ctx, id)
        .await
        .expect("Failed to delete discount");

    // Should not find deleted discount
    let result = repo.get_discount_by_id(ctx, id).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

pub async fn sell_discount_test_partial_update<P: SellPriceRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &P,
) {
    let sell_price_id = create_test_sell_price(ctx, repo).await;
    let id = super::generate_test_id().await;

    let discount = SellDiscountCreate {
        price_id: sell_price_id,
        quantity: 10,
        discount_formula: "-10%".to_string(),
        customer_level: Some(2),
        metadata: Some(serde_json::json!({"note": "original"})),
    };

    repo.create_discount(ctx, id, &discount)
        .await
        .expect("Failed to create discount");

    // Partial update: only update quantity
    let partial_update = SellDiscountUpdate {
        quantity: Some(25),
        discount_formula: None,
        customer_level: Update::Unchanged,
        metadata: Update::Unchanged,
    };
    repo.update_discount(ctx, id, &partial_update)
        .await
        .expect("Failed to update discount");

    let fetched = repo
        .get_discount_by_id(ctx, id)
        .await
        .expect("Failed to get discount")
        .expect("Discount not found");
    // Quantity should be updated
    assert_eq!(fetched.quantity, 25);
    // Other fields should remain unchanged
    assert_eq!(fetched.discount_formula, Some("-10%".to_string()));
    assert_eq!(fetched.customer_level, Some(2));
}

/// Tests that `delete_by_product_variant_ids` soft-deletes sell prices for given variant IDs.
pub async fn sell_price_test_delete_by_product_variant_ids<P: SellPriceRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &P,
) {
    let variant_id_1 = create_test_product_variant(ctx).await;
    let variant_id_2 = create_test_product_variant(ctx).await;
    let variant_id_other = create_test_product_variant(ctx).await;

    // Create prices for variant 1 and 2
    for variant_id in [variant_id_1, variant_id_2] {
        let id = super::generate_test_id().await;
        let uom_id = super::generate_test_id().await;
        let price = SellPriceCreate {
            branch_id: None,
            product_variant_id: variant_id,
            uom_id,
            quantity: 1,
            price: 10000,
            metadata: None,
        };
        repo.create(ctx, id, &price)
            .await
            .expect("Failed to create sell price");
    }

    // Create a price for a different variant (should not be affected)
    let other_id = super::generate_test_id().await;
    let uom_id = super::generate_test_id().await;
    let other_price = SellPriceCreate {
        branch_id: None,
        product_variant_id: variant_id_other,
        uom_id,
        quantity: 1,
        price: 20000,
        metadata: None,
    };
    repo.create(ctx, other_id, &other_price)
        .await
        .expect("Failed to create other sell price");

    // Delete by variant IDs
    repo.delete_by_product_variant_ids(ctx, &[variant_id_1, variant_id_2])
        .await
        .expect("Failed to delete by product variant ids");

    // Prices for variant 1 and 2 should be gone
    let prices_1 = repo
        .get_all_by_product_variant_id(ctx, variant_id_1)
        .await
        .expect("Failed to get prices");
    assert!(prices_1.is_empty());

    let prices_2 = repo
        .get_all_by_product_variant_id(ctx, variant_id_2)
        .await
        .expect("Failed to get prices");
    assert!(prices_2.is_empty());

    // Price for other variant should still exist
    let prices_other = repo
        .get_all_by_product_variant_id(ctx, variant_id_other)
        .await
        .expect("Failed to get prices");
    assert_eq!(prices_other.len(), 1);
}

/// Tests that `delete_by_product_variant_ids` with an empty slice is a no-op.
pub async fn sell_price_test_delete_by_product_variant_ids_empty<P: SellPriceRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &P,
) {
    // Should not error on empty slice
    repo.delete_by_product_variant_ids(ctx, &[])
        .await
        .expect("Failed to delete by empty product variant ids");
}

/// Tests that `delete_by_product_variant_ids` also soft-deletes associated discounts.
pub async fn sell_price_test_delete_by_product_variant_ids_with_discounts<
    P: SellPriceRepository,
>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &P,
) {
    let variant_id = create_test_product_variant(ctx).await;

    // Create a sell price
    let price_id = super::generate_test_id().await;
    let uom_id = super::generate_test_id().await;
    let price = SellPriceCreate {
        branch_id: None,
        product_variant_id: variant_id,
        uom_id,
        quantity: 1,
        price: 10000,
        metadata: None,
    };
    repo.create(ctx, price_id, &price)
        .await
        .expect("Failed to create sell price");

    // Create discounts for that price
    for i in 0..3 {
        let discount_id = super::generate_test_id().await;
        let discount = SellDiscountCreate {
            price_id,
            quantity: (i + 1) * 5,
            discount_formula: format!("-{}%", (i + 1) * 5),
            customer_level: None,
            metadata: None,
        };
        repo.create_discount(ctx, discount_id, &discount)
            .await
            .expect("Failed to create discount");
    }

    // Verify discounts exist
    let discounts_before = repo
        .get_all_discount_by_price_id(ctx, price_id)
        .await
        .expect("Failed to get discounts");
    assert_eq!(discounts_before.len(), 3);

    // Delete by variant IDs — should cascade to discounts
    repo.delete_by_product_variant_ids(ctx, &[variant_id])
        .await
        .expect("Failed to delete by product variant ids");

    // Sell price should be gone
    let prices_after = repo
        .get_all_by_product_variant_id(ctx, variant_id)
        .await
        .expect("Failed to get prices");
    assert!(prices_after.is_empty());

    // Discounts should also be gone
    let discounts_after = repo
        .get_all_discount_by_price_id(ctx, price_id)
        .await
        .expect("Failed to get discounts");
    assert!(discounts_after.is_empty());
}

pub async fn sell_discount_test_delete_by_sell_price_id<P: SellPriceRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &P,
) {
    let sell_price_id = create_test_sell_price(ctx, repo).await;

    // Create multiple discounts for the same price
    for i in 0..3 {
        let id = super::generate_test_id().await;
        let discount = SellDiscountCreate {
            price_id: sell_price_id,
            quantity: (i + 1) * 5,
            discount_formula: format!("-{}%", (i + 1) * 5),
            customer_level: None,
            metadata: None,
        };
        repo.create_discount(ctx, id, &discount)
            .await
            .expect("Failed to create discount");
    }

    // Verify discounts were created
    let discounts = repo
        .get_all_discount_by_price_id(ctx, sell_price_id)
        .await
        .expect("Failed to get all discounts");
    assert_eq!(discounts.len(), 3);

    // Delete all discounts by sell_price_id
    repo.delete_discounts_by_sell_price_id(ctx, sell_price_id)
        .await
        .expect("Failed to delete discounts by sell_price_id");

    // Verify discounts were deleted
    let discounts_after = repo
        .get_all_discount_by_price_id(ctx, sell_price_id)
        .await
        .expect("Failed to get all discounts");
    assert!(discounts_after.is_empty());
}
