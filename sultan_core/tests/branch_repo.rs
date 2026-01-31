mod common;
use sultan_core::testing::storage::branch;

#[tokio::test]
async fn test_branch_repo_integration() {
    let repo = sultan_core::storage::sqlite::branch::SqliteBranchRepository::new();
    branch::branch_test_all(&repo, || async { common::init_sqlite_repo_ctx().await }).await;
}
