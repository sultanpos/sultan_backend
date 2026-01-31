mod common;
use sea_orm::{DatabaseConnection, TransactionTrait};
use sultan_core::domain::Context;
use sultan_core::domain::model::product::UnitOfMeasureCreate;
use sultan_core::storage::{
    RepoCtx, UnitOfMeasureRepository, sqlite::SqliteUnitOfMeasureRepository,
};
use sultan_core::testing::storage::unit;

async fn create_sqlite_unit_repo() -> (RepoCtx<DatabaseConnection>, SqliteUnitOfMeasureRepository) {
    let repo_ctx = common::init_sqlite_repo_ctx().await;
    let repo = SqliteUnitOfMeasureRepository::new();
    (repo_ctx, repo)
}

#[tokio::test]
async fn test_unit_of_measure_repository() {
    let (ctx, repo) = create_sqlite_unit_repo().await;
    unit::test_unit_all(&ctx, &repo).await;
}

#[tokio::test]
pub async fn test_unit_of_measure_repository_with_transaction() {
    let db = common::init_sqlite_db().await;
    let tx = db.begin().await.unwrap();
    let id = common::generate_test_id().await;
    let unit = UnitOfMeasureCreate {
        name: "Kilogram".to_string(),
        description: Some("Unit of mass".to_string()),
    };
    let ctx = RepoCtx {
        ctx: Context::new(),
        db: tx,
    };
    let repo = SqliteUnitOfMeasureRepository::new();
    repo.create(&ctx, id, &unit)
        .await
        .expect("Failed to create unit of measure");

    let ctx_non_tx = RepoCtx {
        ctx: Context::new(),
        db: db.clone(),
    };
    let unit = repo
        .get_by_id(&ctx_non_tx, id)
        .await
        .expect("Failed to get unit of measure");
    assert!(unit.is_none());

    ctx.db.commit().await.unwrap();

    let unit = repo
        .get_by_id(&ctx_non_tx, id)
        .await
        .expect("Failed to get unit of measure")
        .expect("Unit of measure not found");

    assert_eq!(unit.id, id);
    assert_eq!(unit.name, "Kilogram".to_string());
    assert_eq!(unit.description, Some("Unit of mass".to_string()));
    assert!(!unit.is_deleted);
}
