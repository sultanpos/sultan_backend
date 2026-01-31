mod common;
use sultan_core::testing::storage::number;

#[tokio::test]
async fn test_number_repo_integration() {
    let repo = sultan_core::storage::sqlite::number::SqliteNumberRepository::new();
    number::number_test_all(&repo, || async { common::init_sqlite_repo_ctx().await }).await;
}
