use sea_orm::DatabaseConnection;

use crate::{
    domain::model::{
        Update,
        branch::{BranchCreate, BranchFilter, BranchQuery, BranchSortField, BranchUpdate},
        product::SortDirection,
    },
    storage::{BranchRepository, RepoCtx},
};

fn default_query() -> BranchQuery {
    BranchQuery {
        filter: BranchFilter::default(),
        sort_field: BranchSortField::CreatedAt,
        sort_direction: SortDirection::Desc,
        cursor: None,
        limit: 20,
    }
}

pub async fn branch_test_all<C, F, Fut>(repo: &C, ctx_factory: F)
where
    C: BranchRepository,
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = RepoCtx<DatabaseConnection>>,
{
    branch_test_repo_integration(&ctx_factory().await, repo).await;
    branch_test_partial_update(&ctx_factory().await, repo).await;
    branch_test_non_existent(&ctx_factory().await, repo).await;
    branch_test_delete_non_existent(&ctx_factory().await, repo).await;
    branch_test_get_deleted(&ctx_factory().await, repo).await;
    branch_test_get_by_id_not_found(&ctx_factory().await, repo).await;
    branch_test_get_all_branches(&ctx_factory().await, repo).await;
    branch_test_update_branch_not_found(&ctx_factory().await, repo).await;
    branch_test_create_branch_with_all_fields(&ctx_factory().await, repo).await;
    branch_test_update_address_scenarios(&ctx_factory().await, repo).await;
    branch_test_set_all_is_main_false(&ctx_factory().await, repo).await;
    branch_test_cursor_pagination_asc(&ctx_factory().await, repo).await;
    branch_test_cursor_pagination_desc(&ctx_factory().await, repo).await;
    branch_test_cursor_pagination_no_next_on_last_page(&ctx_factory().await, repo).await;
}

pub async fn branch_test_repo_integration<B: BranchRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &B,
) {
    let id = super::generate_test_id().await;
    let branch = BranchCreate {
        is_main: true,
        name: "Main Branch".to_string(),
        code: "MAIN".to_string(),
        address: Some("123 Main St".to_string()),
        phone: Some("555-1234".to_string()),
        npwp: None,
        image: None,
    };

    // Test Create
    repo.create(ctx, id, &branch)
        .await
        .expect("Failed to create branch");

    // Test Get By ID
    let fetched_branch = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get branch")
        .expect("Branch not found");
    assert_eq!(fetched_branch.name, branch.name);
    assert_eq!(fetched_branch.is_main, branch.is_main);

    // Test Update
    let update_data = BranchUpdate {
        name: Some("Updated Branch".to_string()),
        ..Default::default()
    };
    repo.update(ctx, id, &update_data)
        .await
        .expect("Failed to update branch");

    let fetched_updated = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get updated branch")
        .expect("Updated branch not found");
    assert_eq!(fetched_updated.name, "Updated Branch");

    // Test Get All
    let page = repo
        .get_all(ctx, &default_query())
        .await
        .expect("Failed to get all branches");
    // Note: Other tests might have added branches, so we check if it contains at least our branch
    assert!(page.items.iter().any(|b| b.id == id));

    // Test Delete
    repo.delete(ctx, id).await.expect("Failed to delete branch");
    let deleted_branch = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get deleted branch");
    assert!(deleted_branch.is_none());
}

pub async fn branch_test_partial_update<B: BranchRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &B,
) {
    let id = super::generate_test_id().await;
    let branch = BranchCreate {
        is_main: false,
        name: "Original Branch".to_string(),
        code: "ORIG".to_string(),
        address: Some("456 Elm St".to_string()),
        phone: Some("555-5678".to_string()),
        npwp: Some("98765432109876".to_string()),
        image: Some("original.png".to_string()),
    };

    // Create the branch
    repo.create(ctx, id, &branch)
        .await
        .expect("Failed to create branch");

    // Partial update: only update name
    let partial_update = BranchUpdate {
        name: Some("Updated Name".to_string()),
        ..Default::default()
    };
    repo.update(ctx, id, &partial_update)
        .await
        .expect("Failed to update branch");

    let fetched = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get branch")
        .expect("Branch not found");

    // Name should be updated
    assert_eq!(fetched.name, "Updated Name");
    // Other fields should remain unchanged
    assert_eq!(fetched.code, "ORIG");
    assert!(!fetched.is_main);
    assert_eq!(fetched.address, Some("456 Elm St".to_string()));
    assert_eq!(fetched.phone, Some("555-5678".to_string()));
    assert_eq!(fetched.npwp, Some("98765432109876".to_string()));
    assert_eq!(fetched.image, Some("original.png".to_string()));

    // Partial update: only update code
    let partial_update2 = BranchUpdate {
        code: Some("NEW".to_string()),
        ..Default::default()
    };
    repo.update(ctx, id, &partial_update2)
        .await
        .expect("Failed to update branch");

    let fetched2 = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get branch")
        .expect("Branch not found");

    // Code should be updated
    assert_eq!(fetched2.name, "Updated Name"); // Should remain from previous update
    assert_eq!(fetched2.code, "NEW");
    assert_eq!(fetched2.address, Some("456 Elm St".to_string())); // Should remain unchanged
    assert_eq!(fetched2.phone, Some("555-5678".to_string())); // Should remain unchanged
    assert_eq!(fetched2.npwp, Some("98765432109876".to_string())); // Should remain unchanged
    assert_eq!(fetched2.image, Some("original.png".to_string())); // Should remain unchanged
}

pub async fn branch_test_non_existent<B: BranchRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &B,
) {
    let update_data = BranchUpdate {
        name: Some("Non-existent".to_string()),
        ..Default::default()
    };

    let result = repo.update(ctx, 999, &update_data).await;
    assert!(matches!(result, Err(crate::domain::Error::NotFound(_))));
}

pub async fn branch_test_delete_non_existent<B: BranchRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &B,
) {
    let result = repo.delete(ctx, 999).await;
    assert!(matches!(result, Err(crate::domain::Error::NotFound(_))));
}

pub async fn branch_test_get_deleted<B: BranchRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &B,
) {
    let id = super::generate_test_id().await;
    let branch = BranchCreate {
        is_main: false,
        name: "To Delete".to_string(),
        code: "DEL".to_string(),
        address: None,
        phone: None,
        npwp: None,
        image: None,
    };

    repo.create(ctx, id, &branch)
        .await
        .expect("Failed to create branch");
    repo.delete(ctx, id).await.expect("Failed to delete branch");

    let result = repo.get_by_id(ctx, id).await.expect("Failed to get branch");
    assert!(result.is_none());
}

pub async fn branch_test_get_by_id_not_found<B: BranchRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &B,
) {
    let result = repo
        .get_by_id(ctx, 9999)
        .await
        .expect("Failed to get branch");
    assert!(result.is_none());
}

pub async fn branch_test_get_all_branches<B: BranchRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &B,
) {
    // Create multiple branches
    for i in 0..3 {
        let id = super::generate_test_id().await;
        let branch = BranchCreate {
            is_main: i == 0,
            name: format!("Branch {}", i),
            code: format!("BR{}", i),
            address: None,
            phone: None,
            npwp: None,
            image: None,
        };
        repo.create(ctx, id, &branch)
            .await
            .expect("Failed to create branch");
    }

    let page = repo
        .get_all(ctx, &default_query())
        .await
        .expect("Failed to get all branches");
    assert!(page.items.len() >= 3);
}

pub async fn branch_test_update_branch_not_found<B: BranchRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &B,
) {
    let update_data = BranchUpdate {
        name: Some("Non-existent".to_string()),
        ..Default::default()
    };

    let result = repo.update(ctx, 9999, &update_data).await;
    assert!(matches!(result, Err(crate::domain::Error::NotFound(_))));
}

pub async fn branch_test_create_branch_with_all_fields<B: BranchRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &B,
) {
    let id = super::generate_test_id().await;
    let branch = BranchCreate {
        is_main: false,
        name: "Complete Branch".to_string(),
        code: "COMP".to_string(),
        address: Some("456 Complete Ave".to_string()),
        phone: Some("555-9999".to_string()),
        npwp: Some("12345678901234".to_string()),
        image: Some("branch.png".to_string()),
    };

    repo.create(ctx, id, &branch)
        .await
        .expect("Failed to create branch");

    let fetched = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get branch")
        .expect("Branch not found");

    assert_eq!(fetched.name, "Complete Branch");
    assert_eq!(fetched.code, "COMP");
    assert_eq!(fetched.address, Some("456 Complete Ave".to_string()));
    assert_eq!(fetched.phone, Some("555-9999".to_string()));
    assert_eq!(fetched.npwp, Some("12345678901234".to_string()));
    assert_eq!(fetched.image, Some("branch.png".to_string()));
}

pub async fn branch_test_update_address_scenarios<B: BranchRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &B,
) {
    let id = super::generate_test_id().await;
    let branch = BranchCreate {
        is_main: false,
        name: "Address Test Branch".to_string(),
        code: "ADDR".to_string(),
        address: Some("123 Initial St".to_string()),
        phone: Some("555-1111".to_string()),
        npwp: None,
        image: None,
    };

    // Create the branch
    repo.create(ctx, id, &branch)
        .await
        .expect("Failed to create branch");

    // Scenario 1: Update address with valid string value
    let update_with_value = BranchUpdate {
        address: Update::Set("456 Updated Ave".to_string()),
        ..Default::default()
    };
    repo.update(ctx, id, &update_with_value)
        .await
        .expect("Failed to update address with value");

    let fetched1 = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get branch")
        .expect("Branch not found");

    assert_eq!(
        fetched1.address,
        Some("456 Updated Ave".to_string()),
        "Address should be updated to new value"
    );
    assert_eq!(
        fetched1.phone,
        Some("555-1111".to_string()),
        "Phone should remain unchanged"
    );

    // Scenario 2: No update (Unchanged) -> keep the old value as it is
    let update_no_change = BranchUpdate {
        name: Some("Name Changed".to_string()), // Change name to prove update happened
        address: Update::Unchanged,             // Don't touch address
        ..Default::default()
    };
    repo.update(ctx, id, &update_no_change)
        .await
        .expect("Failed to update without address change");

    let fetched2 = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get branch")
        .expect("Branch not found");

    assert_eq!(fetched2.name, "Name Changed", "Name should be updated");
    assert_eq!(
        fetched2.address,
        Some("456 Updated Ave".to_string()),
        "Address should remain unchanged when update field is None"
    );

    // Scenario 3: Update address to nil/NULL value (Clear)
    let update_to_nil = BranchUpdate {
        address: Update::Clear, // Set address to NULL
        ..Default::default()
    };
    repo.update(ctx, id, &update_to_nil)
        .await
        .expect("Failed to update address to nil");

    let fetched3 = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get branch")
        .expect("Branch not found");

    assert_eq!(fetched3.address, None, "Address should be set to NULL/None");
    assert_eq!(
        fetched3.name, "Name Changed",
        "Name should remain unchanged from previous update"
    );
    assert_eq!(
        fetched3.phone,
        Some("555-1111".to_string()),
        "Phone should still remain unchanged"
    );
}

pub async fn branch_test_set_all_is_main_false<B: BranchRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &B,
) {
    // Create three branches, all with is_main = true
    let id1 = super::generate_test_id().await;
    let id2 = super::generate_test_id().await;
    let id3 = super::generate_test_id().await;

    let branch1 = BranchCreate {
        is_main: true,
        name: "Branch 1".to_string(),
        code: "BR1".to_string(),
        address: None,
        phone: None,
        npwp: None,
        image: None,
    };

    let branch2 = BranchCreate {
        is_main: true,
        name: "Branch 2".to_string(),
        code: "BR2".to_string(),
        address: None,
        phone: None,
        npwp: None,
        image: None,
    };

    let branch3 = BranchCreate {
        is_main: true,
        name: "Branch 3".to_string(),
        code: "BR3".to_string(),
        address: None,
        phone: None,
        npwp: None,
        image: None,
    };

    repo.create(ctx, id1, &branch1)
        .await
        .expect("Failed to create branch 1");
    repo.create(ctx, id2, &branch2)
        .await
        .expect("Failed to create branch 2");
    repo.create(ctx, id3, &branch3)
        .await
        .expect("Failed to create branch 3");

    // Verify all branches are created with is_main = true
    let b1 = repo
        .get_by_id(ctx, id1)
        .await
        .expect("Failed to get branch 1")
        .expect("Branch 1 not found");
    let b2 = repo
        .get_by_id(ctx, id2)
        .await
        .expect("Failed to get branch 2")
        .expect("Branch 2 not found");
    let b3 = repo
        .get_by_id(ctx, id3)
        .await
        .expect("Failed to get branch 3")
        .expect("Branch 3 not found");

    assert!(b1.is_main, "Branch 1 should initially be main");
    assert!(b2.is_main, "Branch 2 should initially be main");
    assert!(b3.is_main, "Branch 3 should initially be main");

    // Test 1: Set all branches to is_main = false except id2
    repo.set_all_is_main_false(ctx, Some(id2))
        .await
        .expect("Failed to set all is_main to false except id2");

    let b1_after = repo
        .get_by_id(ctx, id1)
        .await
        .expect("Failed to get branch 1")
        .expect("Branch 1 not found");
    let b2_after = repo
        .get_by_id(ctx, id2)
        .await
        .expect("Failed to get branch 2")
        .expect("Branch 2 not found");
    let b3_after = repo
        .get_by_id(ctx, id3)
        .await
        .expect("Failed to get branch 3")
        .expect("Branch 3 not found");

    assert!(
        !b1_after.is_main,
        "Branch 1 should be set to is_main = false"
    );
    assert!(
        b2_after.is_main,
        "Branch 2 should remain is_main = true (excluded)"
    );
    assert!(
        !b3_after.is_main,
        "Branch 3 should be set to is_main = false"
    );

    // Test 2: Set all branches to is_main = false with no exception (None)
    // First set branch 2 back to is_main = true for testing
    let update = BranchUpdate {
        is_main: Some(true),
        ..Default::default()
    };
    repo.update(ctx, id2, &update)
        .await
        .expect("Failed to update branch 2");

    repo.set_all_is_main_false(ctx, None)
        .await
        .expect("Failed to set all is_main to false");

    let b1_final = repo
        .get_by_id(ctx, id1)
        .await
        .expect("Failed to get branch 1")
        .expect("Branch 1 not found");
    let b2_final = repo
        .get_by_id(ctx, id2)
        .await
        .expect("Failed to get branch 2")
        .expect("Branch 2 not found");
    let b3_final = repo
        .get_by_id(ctx, id3)
        .await
        .expect("Failed to get branch 3")
        .expect("Branch 3 not found");

    assert!(
        !b1_final.is_main,
        "Branch 1 should be is_main = false (no exception)"
    );
    assert!(
        !b2_final.is_main,
        "Branch 2 should be is_main = false (no exception)"
    );
    assert!(
        !b3_final.is_main,
        "Branch 3 should be is_main = false (no exception)"
    );

    // Test 3: Verify that deleted branches are not affected
    let id4 = super::generate_test_id().await;
    let branch4 = BranchCreate {
        is_main: true,
        name: "Branch 4 (to be deleted)".to_string(),
        code: "BR4".to_string(),
        address: None,
        phone: None,
        npwp: None,
        image: None,
    };

    repo.create(ctx, id4, &branch4)
        .await
        .expect("Failed to create branch 4");

    // Soft delete branch 4
    repo.delete(ctx, id4)
        .await
        .expect("Failed to delete branch 4");

    // Call set_all_is_main_false - should not affect deleted branch
    repo.set_all_is_main_false(ctx, None)
        .await
        .expect("Failed to set all is_main to false");

    // Verify branch 4 cannot be retrieved (it's deleted)
    let b4_deleted = repo
        .get_by_id(ctx, id4)
        .await
        .expect("Failed to get deleted branch");
    assert!(
        b4_deleted.is_none(),
        "Deleted branch should not be retrievable"
    );

    // Clean up: delete test branches
    repo.delete(ctx, id1).await.ok();
    repo.delete(ctx, id2).await.ok();
    repo.delete(ctx, id3).await.ok();
}

// =============================================================================
// Cursor pagination
// =============================================================================

/// Test: cursor pagination works in ascending order (by name)
pub async fn branch_test_cursor_pagination_asc<B: BranchRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &B,
) {
    let names = [
        "AAA Branch",
        "BBB Branch",
        "CCC Branch",
        "DDD Branch",
        "EEE Branch",
    ];
    for (i, name) in names.iter().enumerate() {
        let id = super::generate_test_id().await;
        repo.create(
            ctx,
            id,
            &BranchCreate {
                is_main: i == 0,
                name: name.to_string(),
                code: format!("C{}", i),
                address: None,
                phone: None,
                npwp: None,
                image: None,
            },
        )
        .await
        .expect("Failed to create branch");
    }

    let mut query = BranchQuery {
        filter: BranchFilter::default(),
        sort_field: BranchSortField::Name,
        sort_direction: SortDirection::Asc,
        cursor: None,
        limit: 2,
    };

    // Page 1
    let page1 = repo.get_all(ctx, &query).await.expect("page 1 failed");
    assert_eq!(page1.items.len(), 2);
    assert_eq!(page1.items[0].name, "AAA Branch");
    assert_eq!(page1.items[1].name, "BBB Branch");
    assert!(page1.next_cursor.is_some(), "Should have next page");

    // Page 2
    query.cursor = page1.next_cursor;
    let page2 = repo.get_all(ctx, &query).await.expect("page 2 failed");
    assert_eq!(page2.items.len(), 2);
    assert_eq!(page2.items[0].name, "CCC Branch");
    assert_eq!(page2.items[1].name, "DDD Branch");
    assert!(page2.next_cursor.is_some(), "Should have next page");

    // Page 3
    query.cursor = page2.next_cursor;
    let page3 = repo.get_all(ctx, &query).await.expect("page 3 failed");
    assert_eq!(page3.items.len(), 1);
    assert_eq!(page3.items[0].name, "EEE Branch");
    assert!(page3.next_cursor.is_none(), "Should not have next page");

    // No overlap between pages
    let all_names: Vec<&str> = page1
        .items
        .iter()
        .chain(page2.items.iter())
        .chain(page3.items.iter())
        .map(|b| b.name.as_str())
        .collect();
    let unique: std::collections::HashSet<&&str> = all_names.iter().collect();
    assert_eq!(all_names.len(), unique.len(), "No duplicates across pages");
}

/// Test: cursor pagination works in descending order (by name)
pub async fn branch_test_cursor_pagination_desc<B: BranchRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &B,
) {
    let names = ["AAA Branch", "BBB Branch", "CCC Branch"];
    for (i, name) in names.iter().enumerate() {
        let id = super::generate_test_id().await;
        repo.create(
            ctx,
            id,
            &BranchCreate {
                is_main: i == 0,
                name: name.to_string(),
                code: format!("D{}", i),
                address: None,
                phone: None,
                npwp: None,
                image: None,
            },
        )
        .await
        .expect("Failed to create branch");
    }

    let mut query = BranchQuery {
        filter: BranchFilter::default(),
        sort_field: BranchSortField::Name,
        sort_direction: SortDirection::Desc,
        cursor: None,
        limit: 2,
    };

    // Page 1 — descending means CCC first
    let page1 = repo.get_all(ctx, &query).await.expect("page 1 failed");
    assert!(page1.items.len() >= 2);
    assert!(page1.next_cursor.is_some(), "Should have next page");

    // Page 2
    query.cursor = page1.next_cursor;
    let page2 = repo.get_all(ctx, &query).await.expect("page 2 failed");
    assert!(!page2.items.is_empty());

    // No overlap between pages
    let ids1: std::collections::HashSet<i64> = page1.items.iter().map(|b| b.id).collect();
    let ids2: std::collections::HashSet<i64> = page2.items.iter().map(|b| b.id).collect();
    assert!(ids1.is_disjoint(&ids2), "Pages should not overlap");
}

/// Test: when all results fit in one page, next_cursor is None
pub async fn branch_test_cursor_pagination_no_next_on_last_page<B: BranchRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &B,
) {
    let id1 = super::generate_test_id().await;
    let id2 = super::generate_test_id().await;

    for (i, id) in [id1, id2].iter().enumerate() {
        repo.create(
            ctx,
            *id,
            &BranchCreate {
                is_main: i == 0,
                name: format!("Solo Branch {}", i),
                code: format!("S{}", i),
                address: None,
                phone: None,
                npwp: None,
                image: None,
            },
        )
        .await
        .expect("Failed to create branch");
    }

    let query = BranchQuery {
        filter: BranchFilter::default(),
        sort_field: BranchSortField::CreatedAt,
        sort_direction: SortDirection::Asc,
        cursor: None,
        limit: 100,
    };

    let page = repo.get_all(ctx, &query).await.expect("get_all failed");
    assert!(page.items.len() >= 2);
    assert!(
        page.next_cursor.is_none(),
        "Should not have next cursor when all results fit in one page"
    );
}
