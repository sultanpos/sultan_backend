mod common;

use sultan_core::domain::Context;
use sultan_core::domain::model::Update;
use sultan_core::domain::model::product::{UnitOfMeasureCreate, UnitOfMeasureUpdate};
use sultan_core::snowflake::SnowflakeGenerator;
use sultan_core::storage::UnitOfMeasureRepository;
use sultan_core::storage::sqlite::unit::SqliteUnitOfMeasureRepository;

use common::init_sqlite_pool;

fn generate_test_id() -> i64 {
    thread_local! {
        static GENERATOR: SnowflakeGenerator = SnowflakeGenerator::new(1).unwrap();
    }
    GENERATOR.with(|g| g.generate().unwrap())
}

// =============================================================================
// Basic CRUD Tests
// =============================================================================

#[tokio::test]
async fn test_create_unit_of_measure() {
    let pool = init_sqlite_pool().await;
    let repo: SqliteUnitOfMeasureRepository = SqliteUnitOfMeasureRepository::new(pool);
    let ctx = Context::new();

    let id = generate_test_id();
    let unit = UnitOfMeasureCreate {
        name: "Kilogram".to_string(),
        description: Some("Unit of mass".to_string()),
    };
    repo.create(&ctx, id, &unit)
        .await
        .expect("Failed to create unit of measure");

    let unit = repo
        .get_by_id(&ctx, id)
        .await
        .expect("Failed to get unit of measure")
        .expect("Unit of measure not found");

    assert_eq!(unit.id, id);
    assert_eq!(unit.name, "Kilogram".to_string());
    assert_eq!(unit.description, Some("Unit of mass".to_string()));
    assert!(!unit.is_deleted);
}

#[tokio::test]
async fn test_create_unit_without_description() {
    let pool = init_sqlite_pool().await;
    let repo: SqliteUnitOfMeasureRepository = SqliteUnitOfMeasureRepository::new(pool);
    let ctx = Context::new();

    let id = generate_test_id();
    let unit = UnitOfMeasureCreate {
        name: "Piece".to_string(),
        description: None,
    };
    repo.create(&ctx, id, &unit)
        .await
        .expect("Failed to create unit of measure");

    let unit = repo
        .get_by_id(&ctx, id)
        .await
        .expect("Failed to get unit of measure")
        .expect("Unit of measure not found");

    assert_eq!(unit.name, "Piece".to_string());
    assert_eq!(unit.description, None);
}

#[tokio::test]
async fn test_update_unit_name() {
    let pool = init_sqlite_pool().await;
    let repo: SqliteUnitOfMeasureRepository = SqliteUnitOfMeasureRepository::new(pool);
    let ctx = Context::new();

    let id = generate_test_id();
    let unit = UnitOfMeasureCreate {
        name: "Original Name".to_string(),
        description: Some("Original description".to_string()),
    };
    repo.create(&ctx, id, &unit)
        .await
        .expect("Failed to create unit of measure");

    let update = UnitOfMeasureUpdate {
        name: Some("Updated Name".to_string()),
        description: Update::Unchanged,
    };
    repo.update(&ctx, id, &update)
        .await
        .expect("Failed to update unit of measure");

    let unit = repo
        .get_by_id(&ctx, id)
        .await
        .expect("Failed to get unit of measure")
        .expect("Unit of measure not found");

    assert_eq!(unit.name, "Updated Name".to_string());
    // Description should remain unchanged
    assert_eq!(unit.description, Some("Original description".to_string()));
}

#[tokio::test]
async fn test_update_unit_description() {
    let pool = init_sqlite_pool().await;
    let repo: SqliteUnitOfMeasureRepository = SqliteUnitOfMeasureRepository::new(pool);
    let ctx = Context::new();

    let id = generate_test_id();
    let unit = UnitOfMeasureCreate {
        name: "Liter".to_string(),
        description: Some("Old description".to_string()),
    };
    repo.create(&ctx, id, &unit)
        .await
        .expect("Failed to create unit of measure");

    let update = UnitOfMeasureUpdate {
        name: None,
        description: Update::Set("New description".to_string()),
    };
    repo.update(&ctx, id, &update)
        .await
        .expect("Failed to update unit of measure");

    let unit = repo
        .get_by_id(&ctx, id)
        .await
        .expect("Failed to get unit of measure")
        .expect("Unit of measure not found");

    assert_eq!(unit.name, "Liter".to_string());
    assert_eq!(unit.description, Some("New description".to_string()));
}

#[tokio::test]
async fn test_update_clear_description() {
    let pool = init_sqlite_pool().await;
    let repo: SqliteUnitOfMeasureRepository = SqliteUnitOfMeasureRepository::new(pool);
    let ctx = Context::new();

    let id = generate_test_id();
    let unit = UnitOfMeasureCreate {
        name: "Meter".to_string(),
        description: Some("Unit of length".to_string()),
    };
    repo.create(&ctx, id, &unit)
        .await
        .expect("Failed to create unit of measure");

    let update = UnitOfMeasureUpdate {
        name: None,
        description: Update::Clear,
    };
    repo.update(&ctx, id, &update)
        .await
        .expect("Failed to update unit of measure");

    let unit = repo
        .get_by_id(&ctx, id)
        .await
        .expect("Failed to get unit of measure")
        .expect("Unit of measure not found");

    assert_eq!(unit.name, "Meter".to_string());
    assert_eq!(unit.description, None);
}

#[tokio::test]
async fn test_update_non_existent_unit() {
    let pool = init_sqlite_pool().await;
    let repo: SqliteUnitOfMeasureRepository = SqliteUnitOfMeasureRepository::new(pool);
    let ctx = Context::new();

    let non_existent_id = generate_test_id();
    let update = UnitOfMeasureUpdate {
        name: Some("New Name".to_string()),
        description: Update::Unchanged,
    };

    let result = repo.update(&ctx, non_existent_id, &update).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_unit() {
    let pool = init_sqlite_pool().await;
    let repo: SqliteUnitOfMeasureRepository = SqliteUnitOfMeasureRepository::new(pool);
    let ctx = Context::new();

    let id = generate_test_id();
    let unit = UnitOfMeasureCreate {
        name: "Gram".to_string(),
        description: Some("Unit of mass".to_string()),
    };
    repo.create(&ctx, id, &unit)
        .await
        .expect("Failed to create unit of measure");

    repo.delete(&ctx, id)
        .await
        .expect("Failed to delete unit of measure");

    let result = repo.get_by_id(&ctx, id).await.expect("Query failed");
    assert!(result.is_none());
}

#[tokio::test]
async fn test_delete_non_existent_unit() {
    let pool = init_sqlite_pool().await;
    let repo: SqliteUnitOfMeasureRepository = SqliteUnitOfMeasureRepository::new(pool);
    let ctx = Context::new();

    let non_existent_id = generate_test_id();
    let result = repo.delete(&ctx, non_existent_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_all_units() {
    let pool = init_sqlite_pool().await;
    let repo: SqliteUnitOfMeasureRepository = SqliteUnitOfMeasureRepository::new(pool);
    let ctx = Context::new();

    let id1 = generate_test_id();
    let id2 = generate_test_id();
    let id3 = generate_test_id();

    repo.create(
        &ctx,
        id1,
        &UnitOfMeasureCreate {
            name: "Kilogram".to_string(),
            description: Some("Unit of mass".to_string()),
        },
    )
    .await
    .unwrap();

    repo.create(
        &ctx,
        id2,
        &UnitOfMeasureCreate {
            name: "Liter".to_string(),
            description: Some("Unit of volume".to_string()),
        },
    )
    .await
    .unwrap();

    repo.create(
        &ctx,
        id3,
        &UnitOfMeasureCreate {
            name: "Piece".to_string(),
            description: None,
        },
    )
    .await
    .unwrap();

    let units = repo.get_all(&ctx).await.expect("Failed to get all units");

    // Should have at least our 3 units (may have more from other tests)
    assert!(units.len() >= 3);
    assert!(units.iter().any(|u| u.id == id1 && u.name == "Kilogram"));
    assert!(units.iter().any(|u| u.id == id2 && u.name == "Liter"));
    assert!(units.iter().any(|u| u.id == id3 && u.name == "Piece"));
}

#[tokio::test]
async fn test_get_all_excludes_deleted() {
    let pool = init_sqlite_pool().await;
    let repo: SqliteUnitOfMeasureRepository = SqliteUnitOfMeasureRepository::new(pool);
    let ctx = Context::new();

    let id1 = generate_test_id();
    let id2 = generate_test_id();

    repo.create(
        &ctx,
        id1,
        &UnitOfMeasureCreate {
            name: "Active Unit".to_string(),
            description: None,
        },
    )
    .await
    .unwrap();

    repo.create(
        &ctx,
        id2,
        &UnitOfMeasureCreate {
            name: "Deleted Unit".to_string(),
            description: None,
        },
    )
    .await
    .unwrap();

    // Delete the second unit
    repo.delete(&ctx, id2).await.unwrap();

    let units = repo.get_all(&ctx).await.expect("Failed to get all units");

    // Should contain the active unit but not the deleted one
    assert!(units.iter().any(|u| u.id == id1));
    assert!(!units.iter().any(|u| u.id == id2));
}

#[tokio::test]
async fn test_get_by_id_non_existent() {
    let pool = init_sqlite_pool().await;
    let repo: SqliteUnitOfMeasureRepository = SqliteUnitOfMeasureRepository::new(pool);
    let ctx = Context::new();

    let non_existent_id = generate_test_id();
    let result = repo
        .get_by_id(&ctx, non_existent_id)
        .await
        .expect("Query failed");

    assert!(result.is_none());
}
