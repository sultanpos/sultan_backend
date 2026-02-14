mod common;
use sultan_core::storage::sqlite::SqliteStockRepository;
use sultan_core::testing::storage::stock;

#[tokio::test]
async fn test_stock_repository() {
    let repo = SqliteStockRepository::new();
    stock::stock_test_all(&repo, || async { common::init_sqlite_repo_ctx().await }).await;
}
