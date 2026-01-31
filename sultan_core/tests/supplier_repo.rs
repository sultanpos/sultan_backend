mod common;
use sultan_core::testing::storage::supplier;

#[tokio::test]
async fn test_supplier_repository() {
    let repo = sultan_core::storage::sqlite::supplier::SqliteSupplierRepository::new();
    supplier::supplier_test_all(&repo, || async { common::init_sqlite_repo_ctx().await }).await;
}
