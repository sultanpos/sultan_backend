mod common;
use sultan_core::testing::storage::cashier_session;

#[tokio::test]
async fn test_cashier_session_repo_integration() {
    let repo = sultan_core::storage::sqlite::SqliteCashierSessionRepository::new();
    cashier_session::cashier_session_test_all(&repo, || async {
        common::init_sqlite_repo_ctx().await
    })
    .await;
}
