use crate::{
    domain::{
        Context,
        model::{
            Update,
            product::{UnitOfMeasureCreate, UnitOfMeasureUpdate},
        },
    },
    storage::UnitOfMeasureRepository,
};

pub async fn create_sqlite_unit_repo() -> (Context, impl UnitOfMeasureRepository) {
    let pool = super::init_sqlite_pool().await;
    (
        Context::new(),
        crate::storage::sqlite::unit::SqliteUnitOfMeasureRepository::new(pool),
    )
}

// =============================================================================
// Basic CRUD Tests
// =============================================================================

pub async fn unit_test_create<U: UnitOfMeasureRepository>(ctx: &Context, repo: U) {
    let id = super::generate_test_id().await;
    let unit = UnitOfMeasureCreate {
        name: "Kilogram".to_string(),
        description: Some("Unit of mass".to_string()),
    };
    repo.create(ctx, id, &unit)
        .await
        .expect("Failed to create unit of measure");

    let unit = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get unit of measure")
        .expect("Unit of measure not found");

    assert_eq!(unit.id, id);
    assert_eq!(unit.name, "Kilogram".to_string());
    assert_eq!(unit.description, Some("Unit of mass".to_string()));
    assert!(!unit.is_deleted);
}

pub async fn unit_test_create_without_description<U: UnitOfMeasureRepository>(
    ctx: &Context,
    repo: U,
) {
    let id = super::generate_test_id().await;
    let unit = UnitOfMeasureCreate {
        name: "Piece".to_string(),
        description: None,
    };
    repo.create(ctx, id, &unit)
        .await
        .expect("Failed to create unit of measure");

    let unit = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get unit of measure")
        .expect("Unit of measure not found");

    assert_eq!(unit.name, "Piece".to_string());
    assert_eq!(unit.description, None);
}

pub async fn unit_test_update_name<U: UnitOfMeasureRepository>(ctx: &Context, repo: U) {
    let id = super::generate_test_id().await;
    let unit = UnitOfMeasureCreate {
        name: "Original Name".to_string(),
        description: Some("Original description".to_string()),
    };
    repo.create(ctx, id, &unit)
        .await
        .expect("Failed to create unit of measure");

    let update = UnitOfMeasureUpdate {
        name: Some("Updated Name".to_string()),
        description: Update::Unchanged,
    };
    repo.update(ctx, id, &update)
        .await
        .expect("Failed to update unit of measure");

    let unit = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get unit of measure")
        .expect("Unit of measure not found");

    assert_eq!(unit.name, "Updated Name".to_string());
    // Description should remain unchanged
    assert_eq!(unit.description, Some("Original description".to_string()));
}

pub async fn unit_test_update_description<U: UnitOfMeasureRepository>(ctx: &Context, repo: U) {
    let id = super::generate_test_id().await;
    let unit = UnitOfMeasureCreate {
        name: "Liter".to_string(),
        description: Some("Old description".to_string()),
    };
    repo.create(ctx, id, &unit)
        .await
        .expect("Failed to create unit of measure");

    let update = UnitOfMeasureUpdate {
        name: None,
        description: Update::Set("New description".to_string()),
    };
    repo.update(ctx, id, &update)
        .await
        .expect("Failed to update unit of measure");

    let unit = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get unit of measure")
        .expect("Unit of measure not found");

    assert_eq!(unit.name, "Liter".to_string());
    assert_eq!(unit.description, Some("New description".to_string()));
}

pub async fn unit_test_update_clear_description<U: UnitOfMeasureRepository>(
    ctx: &Context,
    repo: U,
) {
    let id = super::generate_test_id().await;
    let unit = UnitOfMeasureCreate {
        name: "Meter".to_string(),
        description: Some("Unit of length".to_string()),
    };
    repo.create(ctx, id, &unit)
        .await
        .expect("Failed to create unit of measure");

    let update = UnitOfMeasureUpdate {
        name: None,
        description: Update::Clear,
    };
    repo.update(ctx, id, &update)
        .await
        .expect("Failed to update unit of measure");

    let unit = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get unit of measure")
        .expect("Unit of measure not found");

    assert_eq!(unit.name, "Meter".to_string());
    assert_eq!(unit.description, None);
}

pub async fn unit_test_update_non_existent<U: UnitOfMeasureRepository>(ctx: &Context, repo: U) {
    let non_existent_id = super::generate_test_id().await;
    let update = UnitOfMeasureUpdate {
        name: Some("New Name".to_string()),
        description: Update::Unchanged,
    };

    let result = repo.update(ctx, non_existent_id, &update).await;
    assert!(result.is_err());
}

pub async fn unit_test_delete<U: UnitOfMeasureRepository>(ctx: &Context, repo: U) {
    let id = super::generate_test_id().await;
    let unit = UnitOfMeasureCreate {
        name: "Gram".to_string(),
        description: Some("Unit of mass".to_string()),
    };
    repo.create(ctx, id, &unit)
        .await
        .expect("Failed to create unit of measure");

    repo.delete(ctx, id)
        .await
        .expect("Failed to delete unit of measure");

    let result = repo.get_by_id(ctx, id).await.expect("Query failed");
    assert!(result.is_none());
}

pub async fn unit_test_delete_non_existent<U: UnitOfMeasureRepository>(ctx: &Context, repo: U) {
    let non_existent_id = super::generate_test_id().await;
    let result = repo.delete(ctx, non_existent_id).await;
    assert!(result.is_err());
}

pub async fn unit_test_get_all<U: UnitOfMeasureRepository>(ctx: &Context, repo: U) {
    let id1 = super::generate_test_id().await;
    let id2 = super::generate_test_id().await;
    let id3 = super::generate_test_id().await;

    repo.create(
        ctx,
        id1,
        &UnitOfMeasureCreate {
            name: "Kilogram".to_string(),
            description: Some("Unit of mass".to_string()),
        },
    )
    .await
    .unwrap();

    repo.create(
        ctx,
        id2,
        &UnitOfMeasureCreate {
            name: "Liter".to_string(),
            description: Some("Unit of volume".to_string()),
        },
    )
    .await
    .unwrap();

    repo.create(
        ctx,
        id3,
        &UnitOfMeasureCreate {
            name: "Piece".to_string(),
            description: None,
        },
    )
    .await
    .unwrap();

    let units = repo.get_all(ctx).await.expect("Failed to get all units");

    // Should have at least our 3 units (may have more from other tests)
    assert!(units.len() >= 3);
    assert!(units.iter().any(|u| u.id == id1 && u.name == "Kilogram"));
    assert!(units.iter().any(|u| u.id == id2 && u.name == "Liter"));
    assert!(units.iter().any(|u| u.id == id3 && u.name == "Piece"));
}

pub async fn unit_test_get_all_excludes_deleted<U: UnitOfMeasureRepository>(
    ctx: &Context,
    repo: U,
) {
    let id1 = super::generate_test_id().await;
    let id2 = super::generate_test_id().await;

    repo.create(
        ctx,
        id1,
        &UnitOfMeasureCreate {
            name: "Active Unit".to_string(),
            description: None,
        },
    )
    .await
    .unwrap();

    repo.create(
        ctx,
        id2,
        &UnitOfMeasureCreate {
            name: "Deleted Unit".to_string(),
            description: None,
        },
    )
    .await
    .unwrap();

    // Delete the second unit
    repo.delete(ctx, id2).await.unwrap();

    let units = repo.get_all(ctx).await.expect("Failed to get all units");

    // Should contain the active unit but not the deleted one
    assert!(units.iter().any(|u| u.id == id1));
    assert!(!units.iter().any(|u| u.id == id2));
}

pub async fn unit_test_get_by_id_non_existent<U: UnitOfMeasureRepository>(ctx: &Context, repo: U) {
    let non_existent_id = super::generate_test_id().await;
    let result = repo
        .get_by_id(ctx, non_existent_id)
        .await
        .expect("Query failed");

    assert!(result.is_none());
}
