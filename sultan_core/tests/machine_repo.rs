mod common;
use sultan_core::storage::sqlite::SqliteMachineRepository;
use sultan_core::testing::storage::machine;

#[tokio::test]
async fn test_machine_repo_integration() {
    let repo = SqliteMachineRepository::new();
    machine::machine_test_all(&repo, || async { common::init_sqlite_repo_ctx().await }).await;
}
