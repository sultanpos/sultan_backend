use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};

use crate::{
    domain::{
        error::Error,
        model::{
            Update,
            payment_channel::{
                PaymentChannelCreate, PaymentChannelFilter, PaymentChannelPriorityUpdate,
                PaymentChannelUpdate,
            },
        },
    },
    storage::{PaymentChannelRepository, RepoCtx},
};

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

pub async fn payment_channel_test_all<C, F, Fut>(repo: &C, ctx_factory: F)
where
    C: PaymentChannelRepository,
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = RepoCtx<DatabaseConnection>>,
{
    payment_channel_test_crud(&ctx_factory().await, repo).await;
    payment_channel_test_create_with_metadata(&ctx_factory().await, repo).await;
    payment_channel_test_partial_update(&ctx_factory().await, repo).await;
    payment_channel_test_update_clear_metadata(&ctx_factory().await, repo).await;
    payment_channel_test_update_not_found(&ctx_factory().await, repo).await;
    payment_channel_test_delete_not_found(&ctx_factory().await, repo).await;
    payment_channel_test_soft_delete_excludes_from_get_by_id(&ctx_factory().await, repo).await;
    payment_channel_test_soft_delete_excludes_from_get_all(&ctx_factory().await, repo).await;
    payment_channel_test_update_after_delete_fails(&ctx_factory().await, repo).await;
    payment_channel_test_get_all_ordered_by_priority(&ctx_factory().await, repo).await;
    payment_channel_test_get_all_filter_by_branch_id(&ctx_factory().await, repo).await;
    payment_channel_test_get_all_filter_by_name(&ctx_factory().await, repo).await;
    payment_channel_test_update_priorities(&ctx_factory().await, repo).await;
    payment_channel_test_update_priorities_skips_deleted(&ctx_factory().await, repo).await;
}

// =============================================================================
// Basic CRUD
// =============================================================================

pub async fn payment_channel_test_crud<C: PaymentChannelRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id = super::generate_test_id().await;

    let create = PaymentChannelCreate {
        branch_id: None,
        name: "Cash".to_string(),
        priority: 1,
        metadata: None,
    };

    repo.create(ctx, id, &create)
        .await
        .expect("Failed to create payment channel");

    let fetched = repo
        .get_by_id(ctx, id)
        .await
        .expect("get_by_id failed")
        .expect("Payment channel not found");

    assert_eq!(fetched.id, id);
    assert_eq!(fetched.name, "Cash");
    assert_eq!(fetched.priority, 1);
    assert!(fetched.branch_id.is_none());
    assert!(fetched.metadata.is_none());
    assert!(!fetched.is_deleted);

    // update
    let update = PaymentChannelUpdate {
        name: Some("Cash Updated".to_string()),
        ..Default::default()
    };
    repo.update(ctx, id, &update)
        .await
        .expect("Failed to update");

    let updated = repo
        .get_by_id(ctx, id)
        .await
        .expect("get_by_id after update failed")
        .expect("Updated channel not found");
    assert_eq!(updated.name, "Cash Updated");
    assert_eq!(updated.priority, 1); // unchanged

    // delete
    repo.delete(ctx, id).await.expect("Failed to delete");

    let after_delete = repo
        .get_by_id(ctx, id)
        .await
        .expect("get_by_id after delete failed");
    assert!(
        after_delete.is_none(),
        "Deleted channel should not be found"
    );
}

pub async fn payment_channel_test_create_with_metadata<C: PaymentChannelRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let id = super::generate_test_id().await;

    let create = PaymentChannelCreate {
        branch_id: Some(branch_id),
        name: "Card".to_string(),
        priority: 2,
        metadata: Some(serde_json::json!({"provider": "Visa", "requires_pin": true})),
    };

    repo.create(ctx, id, &create)
        .await
        .expect("Failed to create with metadata");

    let fetched = repo
        .get_by_id(ctx, id)
        .await
        .expect("get_by_id failed")
        .expect("Channel not found");

    assert_eq!(fetched.branch_id, Some(branch_id));
    assert_eq!(fetched.name, "Card");
    assert!(fetched.metadata.is_some());
    let meta = fetched.metadata.unwrap();
    assert_eq!(meta["provider"], "Visa");
    assert_eq!(meta["requires_pin"], true);
}

// =============================================================================
// Update scenarios
// =============================================================================

pub async fn payment_channel_test_partial_update<C: PaymentChannelRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id = super::generate_test_id().await;

    repo.create(
        ctx,
        id,
        &PaymentChannelCreate {
            branch_id: None,
            name: "Original".to_string(),
            priority: 5,
            metadata: Some(serde_json::json!({"key": "value"})),
        },
    )
    .await
    .expect("create failed");

    // Update only the priority — name and metadata should be unchanged
    repo.update(
        ctx,
        id,
        &PaymentChannelUpdate {
            priority: Some(10),
            ..Default::default()
        },
    )
    .await
    .expect("update priority failed");

    let fetched = repo.get_by_id(ctx, id).await.unwrap().unwrap();
    assert_eq!(fetched.priority, 10);
    assert_eq!(fetched.name, "Original"); // unchanged
    assert!(fetched.metadata.is_some()); // unchanged
}

pub async fn payment_channel_test_update_clear_metadata<C: PaymentChannelRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id = super::generate_test_id().await;

    repo.create(
        ctx,
        id,
        &PaymentChannelCreate {
            branch_id: None,
            name: "Transfer".to_string(),
            priority: 3,
            metadata: Some(serde_json::json!({"bank": "BNI"})),
        },
    )
    .await
    .expect("create failed");

    repo.update(
        ctx,
        id,
        &PaymentChannelUpdate {
            metadata: Update::Clear,
            ..Default::default()
        },
    )
    .await
    .expect("update clear metadata failed");

    let fetched = repo.get_by_id(ctx, id).await.unwrap().unwrap();
    assert!(fetched.metadata.is_none(), "Metadata should be cleared");
    assert_eq!(fetched.name, "Transfer"); // unchanged
}

pub async fn payment_channel_test_update_not_found<C: PaymentChannelRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let err = repo
        .update(
            ctx,
            9999999,
            &PaymentChannelUpdate {
                name: Some("X".to_string()),
                ..Default::default()
            },
        )
        .await
        .expect_err("Expected NotFound error");

    assert!(
        matches!(err, Error::NotFound(_)),
        "Expected NotFound, got {:?}",
        err
    );
}

pub async fn payment_channel_test_delete_not_found<C: PaymentChannelRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let err = repo
        .delete(ctx, 9999999)
        .await
        .expect_err("Expected NotFound error");

    assert!(
        matches!(err, Error::NotFound(_)),
        "Expected NotFound, got {:?}",
        err
    );
}

// =============================================================================
// Soft delete exclusion
// =============================================================================

pub async fn payment_channel_test_soft_delete_excludes_from_get_by_id<
    C: PaymentChannelRepository,
>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id = super::generate_test_id().await;

    repo.create(
        ctx,
        id,
        &PaymentChannelCreate {
            branch_id: None,
            name: "QRIS".to_string(),
            priority: 1,
            metadata: None,
        },
    )
    .await
    .expect("create failed");

    repo.delete(ctx, id).await.expect("delete failed");

    let result = repo.get_by_id(ctx, id).await.expect("get_by_id failed");
    assert!(result.is_none(), "Soft-deleted channel should not be found");
}

pub async fn payment_channel_test_soft_delete_excludes_from_get_all<C: PaymentChannelRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id_active = super::generate_test_id().await;
    let id_deleted = super::generate_test_id().await;

    repo.create(
        ctx,
        id_active,
        &PaymentChannelCreate {
            branch_id: None,
            name: "Active".to_string(),
            priority: 1,
            metadata: None,
        },
    )
    .await
    .expect("create active failed");

    repo.create(
        ctx,
        id_deleted,
        &PaymentChannelCreate {
            branch_id: None,
            name: "Deleted".to_string(),
            priority: 2,
            metadata: None,
        },
    )
    .await
    .expect("create to-delete failed");

    repo.delete(ctx, id_deleted).await.expect("delete failed");

    let all = repo
        .get_all(ctx, &PaymentChannelFilter::default())
        .await
        .expect("get_all failed");

    let ids: Vec<i64> = all.iter().map(|c| c.id).collect();
    assert!(ids.contains(&id_active), "Active channel should appear");
    assert!(
        !ids.contains(&id_deleted),
        "Deleted channel should not appear"
    );
}

pub async fn payment_channel_test_update_after_delete_fails<C: PaymentChannelRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id = super::generate_test_id().await;

    repo.create(
        ctx,
        id,
        &PaymentChannelCreate {
            branch_id: None,
            name: "Temp".to_string(),
            priority: 1,
            metadata: None,
        },
    )
    .await
    .expect("create failed");

    repo.delete(ctx, id).await.expect("delete failed");

    let err = repo
        .update(
            ctx,
            id,
            &PaymentChannelUpdate {
                name: Some("Should Fail".to_string()),
                ..Default::default()
            },
        )
        .await
        .expect_err("Expected NotFound after delete");

    assert!(
        matches!(err, Error::NotFound(_)),
        "Expected NotFound, got {:?}",
        err
    );
}

// =============================================================================
// Ordering and filtering
// =============================================================================

pub async fn payment_channel_test_get_all_ordered_by_priority<C: PaymentChannelRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id_high = super::generate_test_id().await;
    let id_low = super::generate_test_id().await;
    let id_mid = super::generate_test_id().await;

    repo.create(
        ctx,
        id_high,
        &PaymentChannelCreate {
            branch_id: None,
            name: "High Priority".to_string(),
            priority: 1,
            metadata: None,
        },
    )
    .await
    .expect("create high failed");

    repo.create(
        ctx,
        id_low,
        &PaymentChannelCreate {
            branch_id: None,
            name: "Low Priority".to_string(),
            priority: 100,
            metadata: None,
        },
    )
    .await
    .expect("create low failed");

    repo.create(
        ctx,
        id_mid,
        &PaymentChannelCreate {
            branch_id: None,
            name: "Mid Priority".to_string(),
            priority: 50,
            metadata: None,
        },
    )
    .await
    .expect("create mid failed");

    let all = repo
        .get_all(ctx, &PaymentChannelFilter::default())
        .await
        .expect("get_all failed");

    // Filter to only our created channels, then check ordering
    let ours: Vec<_> = all
        .iter()
        .filter(|c| [id_high, id_low, id_mid].contains(&c.id))
        .collect();

    assert_eq!(ours.len(), 3);
    assert!(
        ours[0].priority <= ours[1].priority && ours[1].priority <= ours[2].priority,
        "Channels should be ordered by priority ascending"
    );
    assert_eq!(ours[0].id, id_high);
    assert_eq!(ours[2].id, id_low);
}

pub async fn payment_channel_test_get_all_filter_by_branch_id<C: PaymentChannelRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch1_id = create_test_branch(ctx).await;
    let branch2_id = create_test_branch(ctx).await;
    let id_branch1 = super::generate_test_id().await;
    let id_branch2 = super::generate_test_id().await;
    let id_no_branch = super::generate_test_id().await;

    repo.create(
        ctx,
        id_branch1,
        &PaymentChannelCreate {
            branch_id: Some(branch1_id),
            name: "Branch1 Channel".to_string(),
            priority: 1,
            metadata: None,
        },
    )
    .await
    .expect("create branch1 failed");

    repo.create(
        ctx,
        id_branch2,
        &PaymentChannelCreate {
            branch_id: Some(branch2_id),
            name: "Branch2 Channel".to_string(),
            priority: 1,
            metadata: None,
        },
    )
    .await
    .expect("create branch2 failed");

    repo.create(
        ctx,
        id_no_branch,
        &PaymentChannelCreate {
            branch_id: None,
            name: "Global Channel".to_string(),
            priority: 2,
            metadata: None,
        },
    )
    .await
    .expect("create global failed");

    let branch1_only = repo
        .get_all(
            ctx,
            &PaymentChannelFilter {
                branch_id: Some(branch1_id),
                name: None,
            },
        )
        .await
        .expect("get_all branch1 failed");

    let ids: Vec<i64> = branch1_only.iter().map(|c| c.id).collect();
    assert!(ids.contains(&id_branch1));
    assert!(!ids.contains(&id_branch2));
    assert!(!ids.contains(&id_no_branch));
}

pub async fn payment_channel_test_get_all_filter_by_name<C: PaymentChannelRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id_cash = super::generate_test_id().await;
    let id_card = super::generate_test_id().await;

    repo.create(
        ctx,
        id_cash,
        &PaymentChannelCreate {
            branch_id: None,
            name: "Cash Payment".to_string(),
            priority: 1,
            metadata: None,
        },
    )
    .await
    .expect("create cash failed");

    repo.create(
        ctx,
        id_card,
        &PaymentChannelCreate {
            branch_id: None,
            name: "Card Payment".to_string(),
            priority: 2,
            metadata: None,
        },
    )
    .await
    .expect("create card failed");

    let cash_results = repo
        .get_all(
            ctx,
            &PaymentChannelFilter {
                branch_id: None,
                name: Some("Cash".to_string()),
            },
        )
        .await
        .expect("get_all by name failed");

    let ids: Vec<i64> = cash_results.iter().map(|c| c.id).collect();
    assert!(ids.contains(&id_cash), "Cash channel should appear");
    assert!(!ids.contains(&id_card), "Card channel should not appear");
}

// =============================================================================
// update_priorities
// =============================================================================

pub async fn payment_channel_test_update_priorities<C: PaymentChannelRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id1 = super::generate_test_id().await;
    let id2 = super::generate_test_id().await;
    let id3 = super::generate_test_id().await;

    for (i, id) in [id1, id2, id3].iter().enumerate() {
        repo.create(
            ctx,
            *id,
            &PaymentChannelCreate {
                branch_id: None,
                name: format!("Channel {}", i + 1),
                priority: (i + 1) as i64,
                metadata: None,
            },
        )
        .await
        .expect("create failed");
    }

    // Reverse the order
    let updates = vec![
        PaymentChannelPriorityUpdate {
            id: id1,
            priority: 30,
        },
        PaymentChannelPriorityUpdate {
            id: id2,
            priority: 20,
        },
        PaymentChannelPriorityUpdate {
            id: id3,
            priority: 10,
        },
    ];

    repo.update_priorities(ctx, &updates)
        .await
        .expect("update_priorities failed");

    let ch1 = repo.get_by_id(ctx, id1).await.unwrap().unwrap();
    let ch2 = repo.get_by_id(ctx, id2).await.unwrap().unwrap();
    let ch3 = repo.get_by_id(ctx, id3).await.unwrap().unwrap();

    assert_eq!(ch1.priority, 30);
    assert_eq!(ch2.priority, 20);
    assert_eq!(ch3.priority, 10);

    // Verify ordering via get_all
    let all = repo
        .get_all(ctx, &PaymentChannelFilter::default())
        .await
        .expect("get_all failed");
    let ours: Vec<_> = all
        .iter()
        .filter(|c| [id1, id2, id3].contains(&c.id))
        .collect();
    assert_eq!(ours[0].id, id3); // priority 10 comes first
    assert_eq!(ours[1].id, id2); // priority 20
    assert_eq!(ours[2].id, id1); // priority 30
}

pub async fn payment_channel_test_update_priorities_skips_deleted<C: PaymentChannelRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id_live = super::generate_test_id().await;
    let id_dead = super::generate_test_id().await;

    repo.create(
        ctx,
        id_live,
        &PaymentChannelCreate {
            branch_id: None,
            name: "Live".to_string(),
            priority: 1,
            metadata: None,
        },
    )
    .await
    .expect("create live failed");

    repo.create(
        ctx,
        id_dead,
        &PaymentChannelCreate {
            branch_id: None,
            name: "Dead".to_string(),
            priority: 2,
            metadata: None,
        },
    )
    .await
    .expect("create dead failed");

    repo.delete(ctx, id_dead).await.expect("delete failed");

    // update_priorities on both — deleted one should be silently skipped (no error)
    let updates = vec![
        PaymentChannelPriorityUpdate {
            id: id_live,
            priority: 50,
        },
        PaymentChannelPriorityUpdate {
            id: id_dead,
            priority: 99,
        },
    ];

    repo.update_priorities(ctx, &updates)
        .await
        .expect("update_priorities should not error on deleted ids");

    let live = repo.get_by_id(ctx, id_live).await.unwrap().unwrap();
    assert_eq!(live.priority, 50, "Live channel priority should be updated");
}
