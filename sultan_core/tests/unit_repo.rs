mod common;
use sea_orm::DatabaseConnection;
use sultan_core::storage::{RepoCtx, sqlite::SqliteUnitOfMeasureRepository};
use sultan_core::testing::storage::unit;

pub async fn create_sqlite_unit_repo()
-> (RepoCtx<DatabaseConnection>, SqliteUnitOfMeasureRepository) {
    let repo_ctx = common::init_sqlite_repo_ctx().await;
    let repo = SqliteUnitOfMeasureRepository::new();
    (repo_ctx, repo)
}

#[tokio::test]
async fn test_create_unit_of_measure() {
    let (ctx, repo) = create_sqlite_unit_repo().await;
    unit::test_unit_all(&ctx, &repo).await;
}
