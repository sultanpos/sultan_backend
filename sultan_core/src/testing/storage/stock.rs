use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};

use crate::{
    domain::model::{
        Update,
        stock::{StockCreate, StockUpdate},
    },
    storage::{RepoCtx, stock_repo::StockRepository},
};

/// Run all Stock repository tests.
///
/// This function runs a comprehensive suite of tests for the StockRepository
/// implementation. Each test receives a fresh RepoCtx from the ctx_factory.
///
/// # Arguments
///
/// * `repo` - The StockRepository implementation to test
/// * `ctx_factory` - A factory function that creates new RepoCtx instances
pub async fn stock_test_all<C, F, Fut>(repo: &C, ctx_factory: F)
where
    C: StockRepository,
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = RepoCtx<DatabaseConnection>>,
{
    stock_test_create(&ctx_factory().await, repo).await;
    stock_test_create_without_optional_fields(&ctx_factory().await, repo).await;
    stock_test_get_by_id(&ctx_factory().await, repo).await;
    stock_test_get_by_id_not_found(&ctx_factory().await, repo).await;
    stock_test_get_by_branch_and_variant(&ctx_factory().await, repo).await;
    stock_test_get_by_branch_and_variant_not_found(&ctx_factory().await, repo).await;
    stock_test_update(&ctx_factory().await, repo).await;
    stock_test_update_clear_optional_fields(&ctx_factory().await, repo).await;
    stock_test_update_not_found(&ctx_factory().await, repo).await;
    stock_test_delete(&ctx_factory().await, repo).await;
    stock_test_delete_not_found(&ctx_factory().await, repo).await;
    stock_test_delete_by_product_variant_ids(&ctx_factory().await, repo).await;
    stock_test_delete_by_product_variant_ids_empty(&ctx_factory().await, repo).await;
}

/// Creates a test branch and product_variant in the database.
/// Returns (branch_id, product_variant_id) that can be used for stock tests.
async fn create_test_branch_and_variant(ctx: &RepoCtx<DatabaseConnection>) -> (i64, i64) {
    let branch_id = super::generate_test_id().await;
    let product_id = super::generate_test_id().await;
    let variant_id = super::generate_test_id().await;

    // Insert a branch
    ctx.db
        .execute_raw(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            "INSERT INTO branches (id, name, code, is_main) VALUES (?, 'Test Branch', 'TST', 0)",
            vec![branch_id.into()],
        ))
        .await
        .expect("Failed to insert test branch");

    // Insert a product
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

    (branch_id, variant_id)
}

// =============================================================================
// Basic CRUD Tests
// =============================================================================

pub async fn stock_test_create<S: StockRepository>(ctx: &RepoCtx<DatabaseConnection>, repo: &S) {
    let id = super::generate_test_id().await;
    let (branch_id, variant_id) = create_test_branch_and_variant(ctx).await;

    let stock = StockCreate {
        branch_id,
        product_variant_id: variant_id,
        quantity: 100,
        min_stock: Some(10),
        max_stock: Some(500),
        last_buy_price: Some(15000),
        metadata: None,
    };

    repo.create(ctx, id, &stock)
        .await
        .expect("Failed to create stock");

    let fetched = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get stock")
        .expect("Stock not found");

    assert_eq!(fetched.id, id);
    assert_eq!(fetched.branch_id, branch_id);
    assert_eq!(fetched.product_variant_id, variant_id);
    assert_eq!(fetched.quantity, 100);
    assert_eq!(fetched.min_stock, Some(10));
    assert_eq!(fetched.max_stock, Some(500));
    assert_eq!(fetched.last_buy_price, Some(15000));
    assert_eq!(fetched.metadata, None);
}

pub async fn stock_test_create_without_optional_fields<S: StockRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    let id = super::generate_test_id().await;
    let (branch_id, variant_id) = create_test_branch_and_variant(ctx).await;

    let stock = StockCreate {
        branch_id,
        product_variant_id: variant_id,
        quantity: 50,
        min_stock: None,
        max_stock: None,
        last_buy_price: None,
        metadata: None,
    };

    repo.create(ctx, id, &stock)
        .await
        .expect("Failed to create stock");

    let fetched = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get stock")
        .expect("Stock not found");

    assert_eq!(fetched.quantity, 50);
    assert_eq!(fetched.min_stock, None);
    assert_eq!(fetched.max_stock, None);
    assert_eq!(fetched.last_buy_price, None);
}

pub async fn stock_test_get_by_id<S: StockRepository>(ctx: &RepoCtx<DatabaseConnection>, repo: &S) {
    let id = super::generate_test_id().await;
    let (branch_id, variant_id) = create_test_branch_and_variant(ctx).await;

    let stock = StockCreate {
        branch_id,
        product_variant_id: variant_id,
        quantity: 75,
        min_stock: Some(5),
        max_stock: None,
        last_buy_price: None,
        metadata: None,
    };

    repo.create(ctx, id, &stock)
        .await
        .expect("Failed to create stock");

    let fetched = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get stock")
        .expect("Stock not found");

    assert_eq!(fetched.id, id);
    assert_eq!(fetched.branch_id, branch_id);
    assert_eq!(fetched.product_variant_id, variant_id);
    assert_eq!(fetched.quantity, 75);
}

pub async fn stock_test_get_by_id_not_found<S: StockRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    let non_existent_id = super::generate_test_id().await;
    let result = repo
        .get_by_id(ctx, non_existent_id)
        .await
        .expect("Query failed");
    assert!(result.is_none());
}

pub async fn stock_test_get_by_branch_and_variant<S: StockRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    let id = super::generate_test_id().await;
    let (branch_id, variant_id) = create_test_branch_and_variant(ctx).await;

    let stock = StockCreate {
        branch_id,
        product_variant_id: variant_id,
        quantity: 200,
        min_stock: Some(20),
        max_stock: Some(1000),
        last_buy_price: Some(25000),
        metadata: None,
    };

    repo.create(ctx, id, &stock)
        .await
        .expect("Failed to create stock");

    let fetched = repo
        .get_by_branch_and_variant(ctx, branch_id, variant_id)
        .await
        .expect("Failed to get stock by branch and variant")
        .expect("Stock not found");

    assert_eq!(fetched.id, id);
    assert_eq!(fetched.branch_id, branch_id);
    assert_eq!(fetched.product_variant_id, variant_id);
    assert_eq!(fetched.quantity, 200);
}

pub async fn stock_test_get_by_branch_and_variant_not_found<S: StockRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    let non_existent_branch = super::generate_test_id().await;
    let non_existent_variant = super::generate_test_id().await;
    let result = repo
        .get_by_branch_and_variant(ctx, non_existent_branch, non_existent_variant)
        .await
        .expect("Query failed");
    assert!(result.is_none());
}

pub async fn stock_test_update<S: StockRepository>(ctx: &RepoCtx<DatabaseConnection>, repo: &S) {
    let id = super::generate_test_id().await;
    let (branch_id, variant_id) = create_test_branch_and_variant(ctx).await;

    let stock = StockCreate {
        branch_id,
        product_variant_id: variant_id,
        quantity: 100,
        min_stock: Some(10),
        max_stock: Some(500),
        last_buy_price: Some(15000),
        metadata: None,
    };

    repo.create(ctx, id, &stock)
        .await
        .expect("Failed to create stock");

    let update = StockUpdate {
        min_stock: Update::Set(20),
        max_stock: Update::Set(1000),
        last_buy_price: Update::Set(18000),
        metadata: Update::Unchanged,
    };

    repo.update(ctx, branch_id, variant_id, &update)
        .await
        .expect("Failed to update stock");

    let fetched = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get stock")
        .expect("Stock not found");

    assert_eq!(fetched.min_stock, Some(20));
    assert_eq!(fetched.max_stock, Some(1000));
    assert_eq!(fetched.last_buy_price, Some(18000));
    // quantity should remain unchanged
    assert_eq!(fetched.quantity, 100);
}

pub async fn stock_test_update_clear_optional_fields<S: StockRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    let id = super::generate_test_id().await;
    let (branch_id, variant_id) = create_test_branch_and_variant(ctx).await;

    let stock = StockCreate {
        branch_id,
        product_variant_id: variant_id,
        quantity: 50,
        min_stock: Some(5),
        max_stock: Some(200),
        last_buy_price: Some(10000),
        metadata: None,
    };

    repo.create(ctx, id, &stock)
        .await
        .expect("Failed to create stock");

    let update = StockUpdate {
        min_stock: Update::Clear,
        max_stock: Update::Clear,
        last_buy_price: Update::Clear,
        metadata: Update::Unchanged,
    };

    repo.update(ctx, branch_id, variant_id, &update)
        .await
        .expect("Failed to update stock");

    let fetched = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get stock")
        .expect("Stock not found");

    assert_eq!(fetched.min_stock, None);
    assert_eq!(fetched.max_stock, None);
    assert_eq!(fetched.last_buy_price, None);
}

pub async fn stock_test_update_not_found<S: StockRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    let non_existent_branch = super::generate_test_id().await;
    let non_existent_variant = super::generate_test_id().await;

    let update = StockUpdate {
        min_stock: Update::Set(10),
        max_stock: Update::Unchanged,
        last_buy_price: Update::Unchanged,
        metadata: Update::Unchanged,
    };

    let result = repo
        .update(ctx, non_existent_branch, non_existent_variant, &update)
        .await;
    assert!(result.is_err());
}

pub async fn stock_test_delete<S: StockRepository>(ctx: &RepoCtx<DatabaseConnection>, repo: &S) {
    let id = super::generate_test_id().await;
    let (branch_id, variant_id) = create_test_branch_and_variant(ctx).await;

    let stock = StockCreate {
        branch_id,
        product_variant_id: variant_id,
        quantity: 30,
        min_stock: None,
        max_stock: None,
        last_buy_price: None,
        metadata: None,
    };

    repo.create(ctx, id, &stock)
        .await
        .expect("Failed to create stock");

    repo.delete(ctx, id).await.expect("Failed to delete stock");

    let result = repo.get_by_id(ctx, id).await.expect("Query failed");
    assert!(result.is_none());
}

pub async fn stock_test_delete_not_found<S: StockRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    let non_existent_id = super::generate_test_id().await;
    let result = repo.delete(ctx, non_existent_id).await;
    assert!(result.is_err());
}

/// Tests that `delete_by_product_variant_ids` deletes stocks for given variant IDs.
pub async fn stock_test_delete_by_product_variant_ids<S: StockRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    let (branch_id, variant_id_1) = create_test_branch_and_variant(ctx).await;
    let variant_id_2 = super::generate_test_id().await;
    let variant_id_other = super::generate_test_id().await;

    // Create product_variants for variant_id_2 and variant_id_other
    let product_id = super::generate_test_id().await;
    ctx.db
        .execute_raw(sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            "INSERT INTO products (id, name, product_type, sellable, buyable) VALUES (?, 'Test Product', 'product', 1, 1)",
            vec![product_id.into()],
        ))
        .await
        .expect("Failed to insert test product");

    for variant_id in [variant_id_2, variant_id_other] {
        ctx.db
            .execute_raw(sea_orm::Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Sqlite,
                "INSERT INTO product_variants (id, product_id, name) VALUES (?, ?, 'Test Variant')",
                vec![variant_id.into(), product_id.into()],
            ))
            .await
            .expect("Failed to insert test product variant");
    }

    // Create stocks for variant 1 and 2
    for variant_id in [variant_id_1, variant_id_2] {
        let id = super::generate_test_id().await;
        let stock = StockCreate {
            branch_id,
            product_variant_id: variant_id,
            quantity: 50,
            min_stock: None,
            max_stock: None,
            last_buy_price: None,
            metadata: None,
        };
        repo.create(ctx, id, &stock)
            .await
            .expect("Failed to create stock");
    }

    // Create a stock for a different variant (should not be affected)
    let other_id = super::generate_test_id().await;
    let other_stock = StockCreate {
        branch_id,
        product_variant_id: variant_id_other,
        quantity: 100,
        min_stock: None,
        max_stock: None,
        last_buy_price: None,
        metadata: None,
    };
    repo.create(ctx, other_id, &other_stock)
        .await
        .expect("Failed to create other stock");

    // Delete by variant IDs
    repo.delete_by_product_variant_ids(ctx, &[variant_id_1, variant_id_2])
        .await
        .expect("Failed to delete by product variant ids");

    // Stocks for variant 1 and 2 should be gone
    let stock_1 = repo
        .get_by_branch_and_variant(ctx, branch_id, variant_id_1)
        .await
        .expect("Failed to get stock");
    assert!(stock_1.is_none());

    let stock_2 = repo
        .get_by_branch_and_variant(ctx, branch_id, variant_id_2)
        .await
        .expect("Failed to get stock");
    assert!(stock_2.is_none());

    // Stock for other variant should still exist
    let stock_other = repo
        .get_by_branch_and_variant(ctx, branch_id, variant_id_other)
        .await
        .expect("Failed to get stock");
    assert!(stock_other.is_some());
}

/// Tests that `delete_by_product_variant_ids` with an empty slice is a no-op.
pub async fn stock_test_delete_by_product_variant_ids_empty<S: StockRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    // Should not error on empty slice
    repo.delete_by_product_variant_ids(ctx, &[])
        .await
        .expect("Failed to delete by empty product variant ids");
}
