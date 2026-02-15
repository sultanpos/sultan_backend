//! Integration tests for the ProductRepository implementation.
//!
//! These tests verify the SQLite implementation of the ProductRepository trait
//! using the test helpers from the testing module.

mod common;

use sultan_core::{
    storage::sqlite::{SqliteProductRepository, SqliteSellPriceRepository},
    testing::storage::product::product_test_all,
};

/// Runs all ProductRepository tests against the SQLite implementation.
///
/// This test uses the `product_test_all` function which runs a comprehensive
/// suite of tests covering:
/// - Product CRUD operations (create, read, update, delete)
/// - Product variant CRUD operations
/// - Variant lookup by barcode, ID, and product ID
/// - Variant nested data (SellPrices and Discounts)
/// - Soft delete behavior
/// - Edge cases and error handling
#[tokio::test]
async fn test_product_repo_integration() {
    let repo = SqliteProductRepository::new();
    let sell_price_repo = SqliteSellPriceRepository::new();
    product_test_all(&repo, &sell_price_repo, || async {
        common::init_sqlite_repo_ctx().await
    })
    .await;
}
