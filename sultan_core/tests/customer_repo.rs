mod common;
use sultan_core::testing::storage::customer;

#[tokio::test]
async fn test_customer_repository() {
    let repo = sultan_core::storage::sqlite::customer::SqliteCustomerRepository::new();
    customer::customer_test_all(&repo, || async { common::init_sqlite_repo_ctx().await }).await;
}
