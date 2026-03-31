use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};

use crate::{
    domain::{
        error::Error,
        model::{
            Update,
            machine::{
                MachineCreate, MachineFilter, MachineQuery, MachineSortField, MachineUpdate,
            },
            product::SortDirection,
        },
    },
    storage::{MachineRepository, RepoCtx},
};

/// Insert a minimal branch row so FK constraints on machines are satisfied.
/// Returns the branch_id used.
async fn create_test_branch(ctx: &RepoCtx<DatabaseConnection>) -> i64 {
    let branch_id = super::generate_test_id().await;
    ctx.db
        .execute_raw(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            "INSERT INTO branches (id, name, code, is_main) VALUES (?, 'Test Branch', 'TB', 0)",
            vec![branch_id.into()],
        ))
        .await
        .expect("Failed to insert test branch");
    branch_id
}

fn default_query(filter: MachineFilter) -> MachineQuery {
    MachineQuery {
        filter,
        sort_field: MachineSortField::Name,
        sort_direction: SortDirection::Asc,
        cursor: None,
        limit: 100,
    }
}

pub async fn machine_test_all<C, F, Fut>(repo: &C, ctx_factory: F)
where
    C: MachineRepository,
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = RepoCtx<DatabaseConnection>>,
{
    machine_test_crud(&ctx_factory().await, repo).await;
    machine_test_create_with_all_fields(&ctx_factory().await, repo).await;
    machine_test_partial_update(&ctx_factory().await, repo).await;
    machine_test_update_clear_description(&ctx_factory().await, repo).await;
    machine_test_update_not_found(&ctx_factory().await, repo).await;
    machine_test_delete_not_found(&ctx_factory().await, repo).await;
    machine_test_soft_delete_excludes_from_get_by_id(&ctx_factory().await, repo).await;
    machine_test_soft_delete_excludes_from_get_all(&ctx_factory().await, repo).await;
    machine_test_unique_constraint_conflict(&ctx_factory().await, repo).await;
    machine_test_same_key_different_branch_ok(&ctx_factory().await, repo).await;
    machine_test_get_all_filter_by_branch_id(&ctx_factory().await, repo).await;
    machine_test_get_all_filter_by_name(&ctx_factory().await, repo).await;
    machine_test_cursor_pagination_asc(&ctx_factory().await, repo).await;
    machine_test_cursor_pagination_desc(&ctx_factory().await, repo).await;
    machine_test_cursor_pagination_no_next_on_last_page(&ctx_factory().await, repo).await;
}

// =============================================================================
// Basic CRUD
// =============================================================================

pub async fn machine_test_crud<C: MachineRepository>(ctx: &RepoCtx<DatabaseConnection>, repo: &C) {
    let branch_id = create_test_branch(ctx).await;
    let id = super::generate_test_id().await;

    let create = MachineCreate {
        branch_id,
        key: "POS-01".to_string(),
        name: "Counter 1".to_string(),
        description: Some("Main counter".to_string()),
        metadata: None,
    };

    repo.create(ctx, id, &create)
        .await
        .expect("Failed to create machine");

    // get_by_id — found
    let fetched = repo
        .get_by_id(ctx, id)
        .await
        .expect("get_by_id failed")
        .expect("Machine not found");

    assert_eq!(fetched.id, id);
    assert_eq!(fetched.branch_id, branch_id);
    assert_eq!(fetched.key, "POS-01");
    assert_eq!(fetched.name, "Counter 1");
    assert_eq!(fetched.description, Some("Main counter".to_string()));
    assert!(!fetched.is_deleted);

    // update
    let update = MachineUpdate {
        name: Some("Counter 1 Updated".to_string()),
        ..Default::default()
    };
    repo.update(ctx, id, &update)
        .await
        .expect("Failed to update machine");

    let updated = repo
        .get_by_id(ctx, id)
        .await
        .expect("get_by_id after update failed")
        .expect("Updated machine not found");
    assert_eq!(updated.name, "Counter 1 Updated");
    assert_eq!(updated.key, "POS-01"); // unchanged

    // delete
    repo.delete(ctx, id)
        .await
        .expect("Failed to delete machine");

    let after_delete = repo
        .get_by_id(ctx, id)
        .await
        .expect("get_by_id after delete failed");
    assert!(
        after_delete.is_none(),
        "Deleted machine should not be found"
    );
}

pub async fn machine_test_create_with_all_fields<C: MachineRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let id = super::generate_test_id().await;

    let create = MachineCreate {
        branch_id,
        key: "POS-ALL".to_string(),
        name: "Full Machine".to_string(),
        description: Some("Description text".to_string()),
        metadata: Some(serde_json::json!({"screen": "15inch", "printer": true})),
    };

    repo.create(ctx, id, &create)
        .await
        .expect("Failed to create machine with all fields");

    let fetched = repo
        .get_by_id(ctx, id)
        .await
        .expect("get_by_id failed")
        .expect("Machine not found");

    assert_eq!(fetched.description, Some("Description text".to_string()));
    assert!(fetched.metadata.is_some());
    let meta = fetched.metadata.unwrap();
    assert_eq!(meta["screen"], "15inch");
    assert_eq!(meta["printer"], true);
}

// =============================================================================
// Update scenarios
// =============================================================================

pub async fn machine_test_partial_update<C: MachineRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let id = super::generate_test_id().await;

    repo.create(
        ctx,
        id,
        &MachineCreate {
            branch_id,
            key: "POS-P".to_string(),
            name: "Original".to_string(),
            description: Some("Keep me".to_string()),
            metadata: None,
        },
    )
    .await
    .expect("create failed");

    // Update only the name
    repo.update(
        ctx,
        id,
        &MachineUpdate {
            name: Some("Updated Name".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("update name failed");

    let fetched = repo.get_by_id(ctx, id).await.unwrap().unwrap();
    assert_eq!(fetched.name, "Updated Name");
    assert_eq!(fetched.key, "POS-P"); // key is immutable — must not change
    assert_eq!(fetched.description, Some("Keep me".to_string())); // unchanged
}

pub async fn machine_test_update_clear_description<C: MachineRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let id = super::generate_test_id().await;

    repo.create(
        ctx,
        id,
        &MachineCreate {
            branch_id,
            key: "POS-CLR".to_string(),
            name: "Has Desc".to_string(),
            description: Some("Remove this".to_string()),
            metadata: None,
        },
    )
    .await
    .expect("create failed");

    // Clear description using Update::Clear
    repo.update(
        ctx,
        id,
        &MachineUpdate {
            description: Update::Clear,
            ..Default::default()
        },
    )
    .await
    .expect("update clear failed");

    let fetched = repo.get_by_id(ctx, id).await.unwrap().unwrap();
    assert_eq!(fetched.description, None);
}

pub async fn machine_test_update_not_found<C: MachineRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let result = repo
        .update(
            ctx,
            999_999_999,
            &MachineUpdate {
                name: Some("Ghost".to_string()),
                ..Default::default()
            },
        )
        .await;

    assert!(matches!(result, Err(Error::NotFound(_))));
}

pub async fn machine_test_delete_not_found<C: MachineRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let result = repo.delete(ctx, 999_999_998).await;
    assert!(matches!(result, Err(Error::NotFound(_))));
}

// =============================================================================
// Soft-delete exclusion
// =============================================================================

pub async fn machine_test_soft_delete_excludes_from_get_by_id<C: MachineRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let id = super::generate_test_id().await;

    repo.create(
        ctx,
        id,
        &MachineCreate {
            branch_id,
            key: "POS-DEL".to_string(),
            name: "To Delete".to_string(),
            description: None,
            metadata: None,
        },
    )
    .await
    .expect("create failed");

    repo.delete(ctx, id).await.expect("delete failed");

    let result = repo.get_by_id(ctx, id).await.unwrap();
    assert!(
        result.is_none(),
        "Soft-deleted machine must not be returned by get_by_id"
    );
}

pub async fn machine_test_soft_delete_excludes_from_get_all<C: MachineRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let id = super::generate_test_id().await;

    repo.create(
        ctx,
        id,
        &MachineCreate {
            branch_id,
            key: "POS-DELALL".to_string(),
            name: "Delete From List".to_string(),
            description: None,
            metadata: None,
        },
    )
    .await
    .expect("create failed");

    repo.delete(ctx, id).await.expect("delete failed");

    let query = default_query(MachineFilter {
        branch_id: Some(branch_id),
        name: None,
    });
    let page = repo.get_all(ctx, &query).await.expect("get_all failed");
    assert!(
        page.items.iter().all(|m| m.id != id),
        "Soft-deleted machine must not appear in get_all"
    );
}

// =============================================================================
// Unique constraint / conflict
// =============================================================================

pub async fn machine_test_unique_constraint_conflict<C: MachineRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let id1 = super::generate_test_id().await;
    let id2 = super::generate_test_id().await;

    repo.create(
        ctx,
        id1,
        &MachineCreate {
            branch_id,
            key: "DUPE-KEY".to_string(),
            name: "First".to_string(),
            description: None,
            metadata: None,
        },
    )
    .await
    .expect("first create failed");

    let result = repo
        .create(
            ctx,
            id2,
            &MachineCreate {
                branch_id,
                key: "DUPE-KEY".to_string(), // same key, same branch
                name: "Second".to_string(),
                description: None,
                metadata: None,
            },
        )
        .await;

    assert!(
        matches!(result, Err(Error::Conflict(_))),
        "Duplicate (branch_id, key) must return Error::Conflict, got: {:?}",
        result
    );
}

pub async fn machine_test_same_key_different_branch_ok<C: MachineRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id_a = create_test_branch(ctx).await;
    let branch_id_b = create_test_branch(ctx).await;
    let id1 = super::generate_test_id().await;
    let id2 = super::generate_test_id().await;

    repo.create(
        ctx,
        id1,
        &MachineCreate {
            branch_id: branch_id_a,
            key: "SHARED-KEY".to_string(),
            name: "Branch A Machine".to_string(),
            description: None,
            metadata: None,
        },
    )
    .await
    .expect("create in branch A failed");

    repo.create(
        ctx,
        id2,
        &MachineCreate {
            branch_id: branch_id_b,
            key: "SHARED-KEY".to_string(), // same key, different branch — allowed
            name: "Branch B Machine".to_string(),
            description: None,
            metadata: None,
        },
    )
    .await
    .expect("create in branch B with same key should succeed");
}

// =============================================================================
// Filter tests
// =============================================================================

pub async fn machine_test_get_all_filter_by_branch_id<C: MachineRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_a = create_test_branch(ctx).await;
    let branch_b = create_test_branch(ctx).await;

    let id_a = super::generate_test_id().await;
    let id_b = super::generate_test_id().await;

    repo.create(
        ctx,
        id_a,
        &MachineCreate {
            branch_id: branch_a,
            key: "F-A".to_string(),
            name: "A Machine".to_string(),
            description: None,
            metadata: None,
        },
    )
    .await
    .expect("create A failed");
    repo.create(
        ctx,
        id_b,
        &MachineCreate {
            branch_id: branch_b,
            key: "F-B".to_string(),
            name: "B Machine".to_string(),
            description: None,
            metadata: None,
        },
    )
    .await
    .expect("create B failed");

    let page = repo
        .get_all(
            ctx,
            &default_query(MachineFilter {
                branch_id: Some(branch_a),
                name: None,
            }),
        )
        .await
        .expect("get_all failed");

    assert!(
        page.items.iter().all(|m| m.branch_id == branch_a),
        "Only branch_a machines expected"
    );
    assert!(page.items.iter().any(|m| m.id == id_a));
    assert!(page.items.iter().all(|m| m.id != id_b));
}

pub async fn machine_test_get_all_filter_by_name<C: MachineRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let id1 = super::generate_test_id().await;
    let id2 = super::generate_test_id().await;

    repo.create(
        ctx,
        id1,
        &MachineCreate {
            branch_id,
            key: "NF-1".to_string(),
            name: "Alpha Counter".to_string(),
            description: None,
            metadata: None,
        },
    )
    .await
    .expect("create 1 failed");
    repo.create(
        ctx,
        id2,
        &MachineCreate {
            branch_id,
            key: "NF-2".to_string(),
            name: "Beta Counter".to_string(),
            description: None,
            metadata: None,
        },
    )
    .await
    .expect("create 2 failed");

    let page = repo
        .get_all(
            ctx,
            &default_query(MachineFilter {
                branch_id: None,
                name: Some("Alpha".to_string()),
            }),
        )
        .await
        .expect("get_all failed");

    assert!(
        page.items.iter().any(|m| m.id == id1),
        "Alpha Counter must be in results"
    );
    assert!(
        page.items.iter().all(|m| m.id != id2),
        "Beta Counter must not be in results"
    );
}

// =============================================================================
// Cursor pagination
// =============================================================================

pub async fn machine_test_cursor_pagination_asc<C: MachineRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;

    // Insert 5 machines with predictable names (z→a so DB insert order ≠ sort order)
    for (i, name) in ["Echo", "Delta", "Charlie", "Beta", "Alpha"]
        .iter()
        .enumerate()
    {
        let id = super::generate_test_id().await;
        repo.create(
            ctx,
            id,
            &MachineCreate {
                branch_id,
                key: format!("CP-{}", i),
                name: (*name).to_string(),
                description: None,
                metadata: None,
            },
        )
        .await
        .expect("create failed");
    }

    // Page 1: limit=2, sort by name ASC
    let query1 = MachineQuery {
        filter: MachineFilter {
            branch_id: Some(branch_id),
            name: None,
        },
        sort_field: MachineSortField::Name,
        sort_direction: SortDirection::Asc,
        cursor: None,
        limit: 2,
    };
    let page1 = repo.get_all(ctx, &query1).await.expect("page1 failed");
    assert_eq!(page1.items.len(), 2);
    assert_eq!(page1.items[0].name, "Alpha");
    assert_eq!(page1.items[1].name, "Beta");
    assert!(page1.next_cursor.is_some(), "Should have next cursor");

    // Page 2: use cursor from page1
    let query2 = MachineQuery {
        filter: MachineFilter {
            branch_id: Some(branch_id),
            name: None,
        },
        sort_field: MachineSortField::Name,
        sort_direction: SortDirection::Asc,
        cursor: page1.next_cursor.clone(),
        limit: 2,
    };
    let page2 = repo.get_all(ctx, &query2).await.expect("page2 failed");
    assert_eq!(page2.items.len(), 2);
    assert_eq!(page2.items[0].name, "Charlie");
    assert_eq!(page2.items[1].name, "Delta");
    assert!(page2.next_cursor.is_some(), "Should have next cursor");

    // Page 3: last page
    let query3 = MachineQuery {
        filter: MachineFilter {
            branch_id: Some(branch_id),
            name: None,
        },
        sort_field: MachineSortField::Name,
        sort_direction: SortDirection::Asc,
        cursor: page2.next_cursor.clone(),
        limit: 2,
    };
    let page3 = repo.get_all(ctx, &query3).await.expect("page3 failed");
    assert_eq!(page3.items.len(), 1);
    assert_eq!(page3.items[0].name, "Echo");
    assert!(
        page3.next_cursor.is_none(),
        "Last page must have no next cursor"
    );
}

pub async fn machine_test_cursor_pagination_desc<C: MachineRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;

    for (i, name) in ["Alpha", "Beta", "Charlie"].iter().enumerate() {
        let id = super::generate_test_id().await;
        repo.create(
            ctx,
            id,
            &MachineCreate {
                branch_id,
                key: format!("DESC-{}", i),
                name: (*name).to_string(),
                description: None,
                metadata: None,
            },
        )
        .await
        .expect("create failed");
    }

    // Page 1: limit=2, sort by name DESC → Charlie, Beta
    let query1 = MachineQuery {
        filter: MachineFilter {
            branch_id: Some(branch_id),
            name: None,
        },
        sort_field: MachineSortField::Name,
        sort_direction: SortDirection::Desc,
        cursor: None,
        limit: 2,
    };
    let page1 = repo.get_all(ctx, &query1).await.expect("page1 failed");
    assert_eq!(page1.items.len(), 2);
    assert_eq!(page1.items[0].name, "Charlie");
    assert_eq!(page1.items[1].name, "Beta");
    assert!(page1.next_cursor.is_some());

    // Page 2: → Alpha, no more
    let query2 = MachineQuery {
        filter: MachineFilter {
            branch_id: Some(branch_id),
            name: None,
        },
        sort_field: MachineSortField::Name,
        sort_direction: SortDirection::Desc,
        cursor: page1.next_cursor.clone(),
        limit: 2,
    };
    let page2 = repo.get_all(ctx, &query2).await.expect("page2 failed");
    assert_eq!(page2.items.len(), 1);
    assert_eq!(page2.items[0].name, "Alpha");
    assert!(page2.next_cursor.is_none());
}

pub async fn machine_test_cursor_pagination_no_next_on_last_page<C: MachineRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let id = super::generate_test_id().await;

    repo.create(
        ctx,
        id,
        &MachineCreate {
            branch_id,
            key: "SINGLE".to_string(),
            name: "Only One".to_string(),
            description: None,
            metadata: None,
        },
    )
    .await
    .expect("create failed");

    let query = MachineQuery {
        filter: MachineFilter {
            branch_id: Some(branch_id),
            name: None,
        },
        sort_field: MachineSortField::Name,
        sort_direction: SortDirection::Asc,
        cursor: None,
        limit: 10,
    };
    let page = repo.get_all(ctx, &query).await.expect("get_all failed");
    assert!(page.items.iter().any(|m| m.id == id));
    assert!(
        page.next_cursor.is_none(),
        "Single item within limit must have no next cursor"
    );
}
