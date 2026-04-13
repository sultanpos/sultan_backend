use sea_orm::DatabaseConnection;

use crate::{
    domain::model::{
        Update,
        product::{SortDirection, UnitOfMeasureCreate, UnitOfMeasureUpdate, UnitQuery, UnitSortField},
    },
    storage::{RepoCtx, unit_repo::UnitOfMeasureRepository},
};

pub fn default_query() -> UnitQuery {
    UnitQuery {
        sort_field: UnitSortField::Id,
        sort_direction: SortDirection::Asc,
        cursor: None,
        limit: 100,
    }
}

pub async fn test_unit_all<C, F, Fut>(repo: &C, ctx_factory: F)
where
    C: UnitOfMeasureRepository,
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = RepoCtx<DatabaseConnection>>,
{
    test_create(&ctx_factory().await, repo).await;
    unit_test_create_without_description(&ctx_factory().await, repo).await;
    unit_test_update_name(&ctx_factory().await, repo).await;
    unit_test_update_description(&ctx_factory().await, repo).await;
    unit_test_update_clear_description(&ctx_factory().await, repo).await;
    unit_test_update_non_existent(&ctx_factory().await, repo).await;
    unit_test_delete(&ctx_factory().await, repo).await;
    unit_test_delete_non_existent(&ctx_factory().await, repo).await;
    unit_test_get_all(&ctx_factory().await, repo).await;
    unit_test_get_all_excludes_deleted(&ctx_factory().await, repo).await;
    unit_test_get_all_cursor_pagination_asc(&ctx_factory().await, repo).await;
    unit_test_get_all_cursor_pagination_desc(&ctx_factory().await, repo).await;
    unit_test_get_all_cursor_pagination_no_next(&ctx_factory().await, repo).await;
    unit_test_get_by_id_non_existent(&ctx_factory().await, repo).await;
}

// =============================================================================
// Basic CRUD Tests
// =============================================================================

pub async fn test_create<U: UnitOfMeasureRepository>(ctx: &RepoCtx<DatabaseConnection>, repo: &U) {
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
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
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

pub async fn unit_test_update_name<U: UnitOfMeasureRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
) {
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

pub async fn unit_test_update_description<U: UnitOfMeasureRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
) {
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
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
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

pub async fn unit_test_update_non_existent<U: UnitOfMeasureRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
) {
    let non_existent_id = super::generate_test_id().await;
    let update = UnitOfMeasureUpdate {
        name: Some("New Name".to_string()),
        description: Update::Unchanged,
    };

    let result = repo.update(ctx, non_existent_id, &update).await;
    assert!(result.is_err());
}

pub async fn unit_test_delete<U: UnitOfMeasureRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
) {
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

pub async fn unit_test_delete_non_existent<U: UnitOfMeasureRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
) {
    let non_existent_id = super::generate_test_id().await;
    let result = repo.delete(ctx, non_existent_id).await;
    assert!(result.is_err());
}

pub async fn unit_test_get_all<U: UnitOfMeasureRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
) {
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

    let page = repo
        .get_all(ctx, &default_query())
        .await
        .expect("Failed to get all units");

    // Should have at least our 3 units (may have more from other tests)
    assert!(page.items.len() >= 3);
    assert!(page.items.iter().any(|u| u.id == id1 && u.name == "Kilogram"));
    assert!(page.items.iter().any(|u| u.id == id2 && u.name == "Liter"));
    assert!(page.items.iter().any(|u| u.id == id3 && u.name == "Piece"));
}

pub async fn unit_test_get_all_excludes_deleted<U: UnitOfMeasureRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
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

    let page = repo
        .get_all(ctx, &default_query())
        .await
        .expect("Failed to get all units");

    // Should contain the active unit but not the deleted one
    assert!(page.items.iter().any(|u| u.id == id1));
    assert!(!page.items.iter().any(|u| u.id == id2));
}

pub async fn unit_test_get_by_id_non_existent<U: UnitOfMeasureRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
) {
    let non_existent_id = super::generate_test_id().await;
    let result = repo
        .get_by_id(ctx, non_existent_id)
        .await
        .expect("Query failed");
    assert!(result.is_none());
}

pub async fn unit_test_get_all_cursor_pagination_asc<U: UnitOfMeasureRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
) {
    // Create 5 units
    for i in 0..5 {
        let id = super::generate_test_id().await;
        repo.create(
            ctx,
            id,
            &UnitOfMeasureCreate {
                name: format!("Pag Unit {}", i),
                description: None,
            },
        )
        .await
        .expect("Failed to create unit");
    }

    // Get first page (2 items), sorted by id asc
    let query_page1 = UnitQuery {
        sort_field: UnitSortField::Id,
        sort_direction: SortDirection::Asc,
        cursor: None,
        limit: 2,
    };
    let page1 = repo
        .get_all(ctx, &query_page1)
        .await
        .expect("Failed to get page 1");
    assert_eq!(page1.items.len(), 2);
    assert!(page1.next_cursor.is_some());

    // Get second page using the cursor
    let query_page2 = UnitQuery {
        sort_field: UnitSortField::Id,
        sort_direction: SortDirection::Asc,
        cursor: page1.next_cursor.clone(),
        limit: 2,
    };
    let page2 = repo
        .get_all(ctx, &query_page2)
        .await
        .expect("Failed to get page 2");
    assert_eq!(page2.items.len(), 2);

    // Verify pages don't overlap
    for u1 in &page1.items {
        assert!(!page2.items.iter().any(|u2| u2.id == u1.id));
    }
}

pub async fn unit_test_get_all_cursor_pagination_desc<U: UnitOfMeasureRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
) {
    // Create 3 units
    for i in 0..3 {
        let id = super::generate_test_id().await;
        repo.create(
            ctx,
            id,
            &UnitOfMeasureCreate {
                name: format!("Desc Unit {}", i),
                description: None,
            },
        )
        .await
        .expect("Failed to create unit");
    }

    // Get first page in descending order by name
    let query_page1 = UnitQuery {
        sort_field: UnitSortField::Name,
        sort_direction: SortDirection::Desc,
        cursor: None,
        limit: 2,
    };
    let page1 = repo
        .get_all(ctx, &query_page1)
        .await
        .expect("Failed to get page 1 desc");
    assert_eq!(page1.items.len(), 2);
    assert!(page1.next_cursor.is_some());

    // Get second page using the cursor
    let query_page2 = UnitQuery {
        sort_field: UnitSortField::Name,
        sort_direction: SortDirection::Desc,
        cursor: page1.next_cursor.clone(),
        limit: 2,
    };
    let page2 = repo
        .get_all(ctx, &query_page2)
        .await
        .expect("Failed to get page 2 desc");
    assert_eq!(page2.items.len(), 1);
    assert!(page2.next_cursor.is_none());

    // Verify pages don't overlap
    for u1 in &page1.items {
        assert!(!page2.items.iter().any(|u2| u2.id == u1.id));
    }
}

pub async fn unit_test_get_all_cursor_pagination_no_next<U: UnitOfMeasureRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
) {
    // Create exactly 2 units
    for i in 0..2 {
        let id = super::generate_test_id().await;
        repo.create(
            ctx,
            id,
            &UnitOfMeasureCreate {
                name: format!("No Next Unit {}", i),
                description: None,
            },
        )
        .await
        .expect("Failed to create unit");
    }

    // Get page of 5, only 2 exist — no next cursor
    let query = UnitQuery {
        sort_field: UnitSortField::UpdatedAt,
        sort_direction: SortDirection::Asc,
        cursor: None,
        limit: 5,
    };
    let page = repo.get_all(ctx, &query).await.expect("Failed to get page");
    assert_eq!(page.items.len(), 2);
    assert!(page.next_cursor.is_none());
}
