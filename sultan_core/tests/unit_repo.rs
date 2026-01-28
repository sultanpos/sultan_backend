mod common;
use sqlx::SqlitePool;
use sultan_core::{
    domain::Context, storage::sqlite::SqliteUnitOfMeasureRepository, testing::storage::unit,
};

pub async fn create_sqlite_unit_repo() -> (Context, SqliteUnitOfMeasureRepository, SqlitePool) {
    let pool = common::init_sqlite_pool().await;
    let repo = SqliteUnitOfMeasureRepository::new();
    (Context::new(), repo, pool)
}

// =============================================================================
// Basic CRUD Tests
// =============================================================================

#[tokio::test]
async fn test_create_unit_of_measure() {
    let (ctx, repo, pool) = create_sqlite_unit_repo().await;
    unit::unit_test_create(&ctx, &repo, &pool).await;
}

#[tokio::test]
async fn test_create_unit_without_description() {
    let (ctx, repo, pool) = create_sqlite_unit_repo().await;
    unit::unit_test_create_without_description(&ctx, &repo, &pool).await;
}

#[tokio::test]
async fn test_update_unit_name() {
    let (ctx, repo, pool) = create_sqlite_unit_repo().await;
    unit::unit_test_update_name(&ctx, &repo, &pool).await;
}

#[tokio::test]
async fn test_update_unit_description() {
    let (ctx, repo, pool) = create_sqlite_unit_repo().await;
    unit::unit_test_update_description(&ctx, &repo, &pool).await;
}

#[tokio::test]
async fn test_update_clear_description() {
    let (ctx, repo, pool) = create_sqlite_unit_repo().await;
    unit::unit_test_update_clear_description(&ctx, &repo, &pool).await;
}

#[tokio::test]
async fn test_update_non_existent_unit() {
    let (ctx, repo, pool) = create_sqlite_unit_repo().await;
    unit::unit_test_update_non_existent(&ctx, &repo, &pool).await;
}
/*
#[tokio::test]
async fn test_delete_unit() {
    let (ctx, repo) = unit::create_sqlite_unit_repo().await;
    unit::unit_test_delete(&ctx, repo).await;
}

#[tokio::test]
async fn test_delete_non_existent_unit() {
    let (ctx, repo) = unit::create_sqlite_unit_repo().await;
    unit::unit_test_delete_non_existent(&ctx, repo).await;
}

#[tokio::test]
async fn test_get_all_units() {
    let (ctx, repo) = unit::create_sqlite_unit_repo().await;
    unit::unit_test_get_all(&ctx, repo).await;
}

#[tokio::test]
async fn test_get_all_excludes_deleted() {
    let (ctx, repo) = unit::create_sqlite_unit_repo().await;
    unit::unit_test_get_all_excludes_deleted(&ctx, repo).await;
}

#[tokio::test]
async fn test_get_by_id_non_existent() {
    let (ctx, repo) = unit::create_sqlite_unit_repo().await;
    unit::unit_test_get_by_id_non_existent(&ctx, repo).await;
}
*/
