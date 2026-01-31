mod common;
use sea_orm::DatabaseConnection;
use sultan_core::{
    storage::{BranchRepository, RepoCtx},
    testing::storage::branch,
};

async fn create_sqlite_branch_repo() -> (RepoCtx<DatabaseConnection>, impl BranchRepository) {
    let repo_ctx = common::init_sqlite_repo_ctx().await;
    (
        repo_ctx,
        sultan_core::storage::sqlite::branch::SqliteBranchRepository::new(),
    )
}

#[tokio::test]
async fn test_branch_repo_integration() {
    let (ctx, repo) = create_sqlite_branch_repo().await;
    branch::branch_test_repo_integration(&ctx, repo).await;
}

#[tokio::test]
async fn test_partial_update_branch() {
    let (ctx, repo) = create_sqlite_branch_repo().await;
    branch::branch_test_partial_update(&ctx, repo).await;
}

#[tokio::test]
async fn test_update_non_existent_branch() {
    let (ctx, repo) = create_sqlite_branch_repo().await;
    branch::branch_test_non_existent(&ctx, repo).await;
}

#[tokio::test]
async fn test_delete_non_existent_branch() {
    let (ctx, repo) = create_sqlite_branch_repo().await;
    branch::branch_test_delete_non_existent(&ctx, repo).await;
}

#[tokio::test]
async fn test_get_deleted_branch() {
    let (ctx, repo) = create_sqlite_branch_repo().await;
    branch::branch_test_get_deleted(&ctx, repo).await;
}

#[tokio::test]
async fn test_get_by_id_not_found() {
    let (ctx, repo) = create_sqlite_branch_repo().await;
    branch::branch_test_get_by_id_not_found(&ctx, repo).await;
}

#[tokio::test]
async fn test_get_all_branches() {
    let (ctx, repo) = create_sqlite_branch_repo().await;
    branch::branch_test_get_all_branches(&ctx, repo).await;
}

#[tokio::test]
async fn test_update_branch_not_found() {
    let (ctx, repo) = create_sqlite_branch_repo().await;
    branch::branch_test_update_branch_not_found(&ctx, repo).await;
}

#[tokio::test]
async fn test_create_branch_with_all_fields() {
    let (ctx, repo) = create_sqlite_branch_repo().await;
    branch::branch_test_create_branch_with_all_fields(&ctx, repo).await;
}

#[tokio::test]
async fn test_update_address_scenarios() {
    let (ctx, repo) = create_sqlite_branch_repo().await;
    branch::branch_test_update_address_scenarios(&ctx, repo).await;
}
