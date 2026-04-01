use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};

use crate::{
    domain::model::{
        cashier_session::{
            CashierSessionClose, CashierSessionCreate, CashierSessionFilter, CashierSessionQuery,
            CashierSessionSortField, SessionStatus,
        },
        product::SortDirection,
    },
    storage::{CashierSessionRepository, RepoCtx},
};

/// Insert a minimal branch row to satisfy FK constraints.
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

/// Insert a minimal user row to satisfy FK constraints.
async fn create_test_user(ctx: &RepoCtx<DatabaseConnection>) -> i64 {
    let user_id = super::generate_test_id().await;
    ctx.db
        .execute_raw(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            "INSERT INTO users (id, username, password, name) VALUES (?, ?, 'hashed', 'Test User')",
            vec![user_id.into(), format!("user_{}", user_id).into()],
        ))
        .await
        .expect("Failed to insert test user");
    user_id
}

fn default_query(filter: CashierSessionFilter) -> CashierSessionQuery {
    CashierSessionQuery {
        filter,
        sort_field: CashierSessionSortField::OpenedAt,
        sort_direction: SortDirection::Asc,
        cursor: None,
        limit: 100,
    }
}

pub async fn cashier_session_test_all<C, F, Fut>(repo: &C, ctx_factory: F)
where
    C: CashierSessionRepository,
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = RepoCtx<DatabaseConnection>>,
{
    cashier_session_test_create_and_get_by_id(&ctx_factory().await, repo).await;
    cashier_session_test_open_and_close(&ctx_factory().await, repo).await;
    cashier_session_test_get_open_by_user(&ctx_factory().await, repo).await;
    cashier_session_test_get_open_by_user_none_after_close(&ctx_factory().await, repo).await;
    cashier_session_test_get_all_filter_by_branch(&ctx_factory().await, repo).await;
    cashier_session_test_get_all_filter_by_user(&ctx_factory().await, repo).await;
    cashier_session_test_get_all_filter_by_status(&ctx_factory().await, repo).await;
    cashier_session_test_cursor_pagination_asc(&ctx_factory().await, repo).await;
    cashier_session_test_cursor_pagination_no_next_on_last_page(&ctx_factory().await, repo).await;
    cashier_session_test_soft_delete(&ctx_factory().await, repo).await;
    cashier_session_test_delete_not_found(&ctx_factory().await, repo).await;
    cashier_session_test_close_not_found(&ctx_factory().await, repo).await;
    cashier_session_test_close_already_closed(&ctx_factory().await, repo).await;
}

// =============================================================================
// Basic CRUD
// =============================================================================

pub async fn cashier_session_test_create_and_get_by_id<C: CashierSessionRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let user_id = create_test_user(ctx).await;
    let id = super::generate_test_id().await;

    let create = CashierSessionCreate {
        branch_id,
        user_id,
        opening_cash: 500_000,
        notes: Some("Morning shift".to_string()),
    };

    repo.create(ctx, id, &create)
        .await
        .expect("Failed to create cashier session");

    let fetched = repo
        .get_by_id(ctx, id)
        .await
        .expect("get_by_id failed")
        .expect("Session not found");

    assert_eq!(fetched.id, id);
    assert_eq!(fetched.branch_id, branch_id);
    assert_eq!(fetched.user_id, user_id);
    assert_eq!(fetched.opening_cash, 500_000);
    assert_eq!(fetched.notes, Some("Morning shift".to_string()));
    assert_eq!(fetched.status, SessionStatus::Open);
    assert!(fetched.closed_at.is_none());
    assert!(fetched.closing_cash.is_none());
    assert!(!fetched.is_deleted);
}

pub async fn cashier_session_test_open_and_close<C: CashierSessionRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let user_id = create_test_user(ctx).await;
    let id = super::generate_test_id().await;

    let create = CashierSessionCreate {
        branch_id,
        user_id,
        opening_cash: 100_000,
        notes: None,
    };
    repo.create(ctx, id, &create)
        .await
        .expect("Failed to create session");

    let close_data = CashierSessionClose {
        closing_cash: 750_000,
        notes: Some("End of shift".to_string()),
    };
    repo.close(ctx, id, &close_data)
        .await
        .expect("Failed to close session");

    let updated = repo
        .get_by_id(ctx, id)
        .await
        .expect("get_by_id failed")
        .expect("Session not found after close");

    assert_eq!(updated.status, SessionStatus::Closed);
    assert!(updated.closed_at.is_some());
    assert_eq!(updated.closing_cash, Some(750_000));
    assert_eq!(updated.notes, Some("End of shift".to_string()));
}

// =============================================================================
// get_open_by_user
// =============================================================================

pub async fn cashier_session_test_get_open_by_user<C: CashierSessionRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let user_id = create_test_user(ctx).await;

    // None before any session
    let none = repo
        .get_open_by_user(ctx, branch_id, user_id)
        .await
        .expect("get_open_by_user failed");
    assert!(none.is_none());

    let id = super::generate_test_id().await;
    repo.create(
        ctx,
        id,
        &CashierSessionCreate {
            branch_id,
            user_id,
            opening_cash: 0,
            notes: None,
        },
    )
    .await
    .expect("Failed to create session");

    let found = repo
        .get_open_by_user(ctx, branch_id, user_id)
        .await
        .expect("get_open_by_user failed")
        .expect("Expected open session");

    assert_eq!(found.id, id);
    assert_eq!(found.status, SessionStatus::Open);
}

pub async fn cashier_session_test_get_open_by_user_none_after_close<C: CashierSessionRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let user_id = create_test_user(ctx).await;
    let id = super::generate_test_id().await;

    repo.create(
        ctx,
        id,
        &CashierSessionCreate {
            branch_id,
            user_id,
            opening_cash: 0,
            notes: None,
        },
    )
    .await
    .expect("Failed to create session");

    repo.close(
        ctx,
        id,
        &CashierSessionClose {
            closing_cash: 0,
            notes: None,
        },
    )
    .await
    .expect("Failed to close session");

    let none = repo
        .get_open_by_user(ctx, branch_id, user_id)
        .await
        .expect("get_open_by_user failed");
    assert!(none.is_none(), "Closed session should not appear as open");
}

// =============================================================================
// get_all filters
// =============================================================================

pub async fn cashier_session_test_get_all_filter_by_branch<C: CashierSessionRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_a = create_test_branch(ctx).await;
    let branch_b = create_test_branch(ctx).await;
    let user_id = create_test_user(ctx).await;

    let id_a = super::generate_test_id().await;
    let id_b = super::generate_test_id().await;

    repo.create(
        ctx,
        id_a,
        &CashierSessionCreate {
            branch_id: branch_a,
            user_id,
            opening_cash: 0,
            notes: None,
        },
    )
    .await
    .expect("create a");
    repo.create(
        ctx,
        id_b,
        &CashierSessionCreate {
            branch_id: branch_b,
            user_id,
            opening_cash: 0,
            notes: None,
        },
    )
    .await
    .expect("create b");

    let results = repo
        .get_all(
            ctx,
            &default_query(CashierSessionFilter {
                branch_id: Some(branch_a),
                ..Default::default()
            }),
        )
        .await
        .expect("get_all failed");

    assert_eq!(results.items.len(), 1);
    assert_eq!(results.items[0].id, id_a);
}

pub async fn cashier_session_test_get_all_filter_by_user<C: CashierSessionRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let user_a = create_test_user(ctx).await;
    let user_b = create_test_user(ctx).await;

    let id_a = super::generate_test_id().await;
    let id_b = super::generate_test_id().await;

    repo.create(
        ctx,
        id_a,
        &CashierSessionCreate {
            branch_id,
            user_id: user_a,
            opening_cash: 0,
            notes: None,
        },
    )
    .await
    .expect("create a");
    repo.create(
        ctx,
        id_b,
        &CashierSessionCreate {
            branch_id,
            user_id: user_b,
            opening_cash: 0,
            notes: None,
        },
    )
    .await
    .expect("create b");

    let results = repo
        .get_all(
            ctx,
            &default_query(CashierSessionFilter {
                user_id: Some(user_a),
                ..Default::default()
            }),
        )
        .await
        .expect("get_all failed");

    assert_eq!(results.items.len(), 1);
    assert_eq!(results.items[0].id, id_a);
}

pub async fn cashier_session_test_get_all_filter_by_status<C: CashierSessionRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let user_id = create_test_user(ctx).await;

    let id_open = super::generate_test_id().await;
    let id_closed = super::generate_test_id().await;

    repo.create(
        ctx,
        id_open,
        &CashierSessionCreate {
            branch_id,
            user_id,
            opening_cash: 0,
            notes: None,
        },
    )
    .await
    .expect("create open");

    // Need a second user to open another session (business rule: one open per user/branch)
    let user2 = create_test_user(ctx).await;
    repo.create(
        ctx,
        id_closed,
        &CashierSessionCreate {
            branch_id,
            user_id: user2,
            opening_cash: 0,
            notes: None,
        },
    )
    .await
    .expect("create to close");
    repo.close(
        ctx,
        id_closed,
        &CashierSessionClose {
            closing_cash: 0,
            notes: None,
        },
    )
    .await
    .expect("close session");

    let open_results = repo
        .get_all(
            ctx,
            &default_query(CashierSessionFilter {
                status: Some(SessionStatus::Open),
                ..Default::default()
            }),
        )
        .await
        .expect("get_all open failed");

    assert!(
        open_results
            .items
            .iter()
            .all(|s| s.status == SessionStatus::Open)
    );
    assert!(open_results.items.iter().any(|s| s.id == id_open));
    assert!(!open_results.items.iter().any(|s| s.id == id_closed));
}

// =============================================================================
// Soft delete
// =============================================================================

pub async fn cashier_session_test_soft_delete<C: CashierSessionRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let user_id = create_test_user(ctx).await;
    let id = super::generate_test_id().await;

    repo.create(
        ctx,
        id,
        &CashierSessionCreate {
            branch_id,
            user_id,
            opening_cash: 0,
            notes: None,
        },
    )
    .await
    .expect("create failed");

    repo.delete(ctx, id).await.expect("delete failed");

    let result = repo.get_by_id(ctx, id).await.expect("get_by_id failed");
    assert!(result.is_none(), "Deleted session should not be returned");
}

pub async fn cashier_session_test_delete_not_found<C: CashierSessionRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let result = repo.delete(ctx, 999_999_999).await;
    assert!(
        matches!(result, Err(crate::domain::error::Error::NotFound(_))),
        "Expected NotFound for delete on non-existent session"
    );
}

// =============================================================================
// Error cases
// =============================================================================

pub async fn cashier_session_test_close_not_found<C: CashierSessionRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let result = repo
        .close(
            ctx,
            999_999_999,
            &CashierSessionClose {
                closing_cash: 0,
                notes: None,
            },
        )
        .await;
    assert!(
        matches!(result, Err(crate::domain::error::Error::NotFound(_))),
        "Expected NotFound when closing non-existent session"
    );
}

pub async fn cashier_session_test_close_already_closed<C: CashierSessionRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let user_id = create_test_user(ctx).await;
    let id = super::generate_test_id().await;

    repo.create(
        ctx,
        id,
        &CashierSessionCreate {
            branch_id,
            user_id,
            opening_cash: 0,
            notes: None,
        },
    )
    .await
    .expect("create failed");

    repo.close(
        ctx,
        id,
        &CashierSessionClose {
            closing_cash: 0,
            notes: None,
        },
    )
    .await
    .expect("first close failed");

    let result = repo
        .close(
            ctx,
            id,
            &CashierSessionClose {
                closing_cash: 0,
                notes: None,
            },
        )
        .await;
    assert!(
        matches!(result, Err(crate::domain::error::Error::NotFound(_))),
        "Expected NotFound when closing an already-closed session"
    );
}

// =============================================================================
// Cursor pagination
// =============================================================================

pub async fn cashier_session_test_cursor_pagination_asc<C: CashierSessionRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;

    // Create 3 sessions with different users (one open per user/branch)
    for _ in 0..3 {
        let user_id = create_test_user(ctx).await;
        let id = super::generate_test_id().await;
        repo.create(
            ctx,
            id,
            &CashierSessionCreate {
                branch_id,
                user_id,
                opening_cash: 0,
                notes: None,
            },
        )
        .await
        .expect("create failed");
    }

    // Page 1: limit 2
    let page1 = repo
        .get_all(
            ctx,
            &CashierSessionQuery {
                filter: CashierSessionFilter {
                    branch_id: Some(branch_id),
                    ..Default::default()
                },
                sort_field: CashierSessionSortField::OpenedAt,
                sort_direction: SortDirection::Asc,
                cursor: None,
                limit: 2,
            },
        )
        .await
        .expect("page 1 failed");

    assert_eq!(page1.items.len(), 2);
    assert!(page1.next_cursor.is_some(), "Expected a next cursor");

    // Page 2: use cursor from page 1
    let page2 = repo
        .get_all(
            ctx,
            &CashierSessionQuery {
                filter: CashierSessionFilter {
                    branch_id: Some(branch_id),
                    ..Default::default()
                },
                sort_field: CashierSessionSortField::OpenedAt,
                sort_direction: SortDirection::Asc,
                cursor: page1.next_cursor,
                limit: 2,
            },
        )
        .await
        .expect("page 2 failed");

    assert_eq!(page2.items.len(), 1);
    assert!(page2.next_cursor.is_none(), "No more pages expected");

    // IDs across pages must not overlap
    let ids_p1: Vec<i64> = page1.items.iter().map(|s| s.id).collect();
    let ids_p2: Vec<i64> = page2.items.iter().map(|s| s.id).collect();
    assert!(
        ids_p1.iter().all(|id| !ids_p2.contains(id)),
        "Pages must not contain duplicate IDs"
    );
}

pub async fn cashier_session_test_cursor_pagination_no_next_on_last_page<
    C: CashierSessionRepository,
>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let user_id = create_test_user(ctx).await;
    let id = super::generate_test_id().await;

    repo.create(
        ctx,
        id,
        &CashierSessionCreate {
            branch_id,
            user_id,
            opening_cash: 0,
            notes: None,
        },
    )
    .await
    .expect("create failed");

    let page = repo
        .get_all(
            ctx,
            &CashierSessionQuery {
                filter: CashierSessionFilter {
                    branch_id: Some(branch_id),
                    ..Default::default()
                },
                sort_field: CashierSessionSortField::OpenedAt,
                sort_direction: SortDirection::Asc,
                cursor: None,
                limit: 10,
            },
        )
        .await
        .expect("get_all failed");

    assert_eq!(page.items.len(), 1);
    assert!(page.next_cursor.is_none());
}
