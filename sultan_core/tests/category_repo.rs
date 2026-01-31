mod common;
use sultan_core::testing::storage::category;

#[tokio::test]
async fn test_category_repository() {
    let repo = sultan_core::storage::sqlite::category::SqliteCategoryRepository::new();

    category::category_test_all(&repo, || async { common::init_sqlite_repo_ctx().await }).await;
}
