mod common;
use sultan_core::testing::storage::sell_price;

#[tokio::test]
async fn test_sell_price_repo_integration() {
    let repo = sultan_core::storage::sqlite::sell_price::SqliteSellPriceRepository::new();
    sell_price::sell_price_test_all(&repo, || async { common::init_sqlite_repo_ctx().await }).await;
}
