mod common;
use sultan_core::testing::storage::user;

#[tokio::test]
async fn test_user_repo_integration() {
    let repo = sultan_core::storage::sqlite::user::SqliteUserRepository::new();
    user::user_test_all(&repo, || async { common::init_sqlite_repo_ctx().await }).await;
}
