mod common;

// =============================================================================
// Basic CRUD Tests
// =============================================================================

#[tokio::test]
async fn test_create_category() {
    let (ctx, repo) = common::category_share::create_sqlite_category_repo().await;
    common::category_share::category_test_create(&ctx, repo).await;
}

#[tokio::test]
async fn test_create_category_without_description() {
    let (ctx, repo) = common::category_share::create_sqlite_category_repo().await;
    common::category_share::category_test_create_without_description(&ctx, repo).await;
}

#[tokio::test]
async fn test_update_category_name() {
    let (ctx, repo) = common::category_share::create_sqlite_category_repo().await;
    common::category_share::category_test_update_name(&ctx, repo).await;
}

#[tokio::test]
async fn test_update_category_description() {
    let (ctx, repo) = common::category_share::create_sqlite_category_repo().await;
    common::category_share::category_test_update_description(&ctx, repo).await;
}

#[tokio::test]
async fn test_update_non_existent_category() {
    let (ctx, repo) = common::category_share::create_sqlite_category_repo().await;
    common::category_share::category_test_update_non_existent(&ctx, repo).await;
}

#[tokio::test]
async fn test_delete_category() {
    let (ctx, repo) = common::category_share::create_sqlite_category_repo().await;
    common::category_share::category_test_delete(&ctx, repo).await;
}

#[tokio::test]
async fn test_delete_non_existent_category() {
    let (ctx, repo) = common::category_share::create_sqlite_category_repo().await;
    common::category_share::category_test_delete_non_existent(&ctx, repo).await;
}

#[tokio::test]
async fn test_get_by_id_not_found() {
    let (ctx, repo) = common::category_share::create_sqlite_category_repo().await;
    common::category_share::category_test_get_by_id_not_found(&ctx, repo).await;
}

#[tokio::test]
async fn test_get_all_empty() {
    let (ctx, repo) = common::category_share::create_sqlite_category_repo().await;
    common::category_share::category_test_get_all_empty(&ctx, repo).await;
}

#[tokio::test]
async fn test_get_all_multiple_categories() {
    let (ctx, repo) = common::category_share::create_sqlite_category_repo().await;
    common::category_share::category_test_get_all_multiple(&ctx, repo).await;
}

// =============================================================================
// Parent-Child Relationship Tests
// =============================================================================

#[tokio::test]
async fn test_create_category_with_parent() {
    let (ctx, repo) = common::category_share::create_sqlite_category_repo().await;
    common::category_share::category_test_create_with_parent(&ctx, repo).await;
}

#[tokio::test]
async fn test_create_nested_categories() {
    let (ctx, repo) = common::category_share::create_sqlite_category_repo().await;
    common::category_share::category_test_create_nested(&ctx, repo).await;
}

#[tokio::test]
async fn test_get_all_returns_tree_structure() {
    let (ctx, repo) = common::category_share::create_sqlite_category_repo().await;
    common::category_share::category_test_get_all_tree_structure(&ctx, repo).await;
}

#[tokio::test]
async fn test_update_category_parent() {
    let (ctx, repo) = common::category_share::create_sqlite_category_repo().await;
    common::category_share::category_test_update_parent(&ctx, repo).await;
}

#[tokio::test]
async fn test_multiple_children() {
    let (ctx, repo) = common::category_share::create_sqlite_category_repo().await;
    common::category_share::category_test_multiple_children(&ctx, repo).await;
}

// =============================================================================
// Depth Limit Tests (MAX_DEPTH = 5)
// =============================================================================

#[tokio::test]
async fn test_create_at_max_depth() {
    let (ctx, repo) = common::category_share::create_sqlite_category_repo().await;
    common::category_share::category_test_create_at_max_depth(&ctx, repo).await;
}

#[tokio::test]
async fn test_create_exceeds_max_depth() {
    let (ctx, repo) = common::category_share::create_sqlite_category_repo().await;
    common::category_share::category_test_create_exceeds_max_depth(&ctx, repo).await;
}

#[tokio::test]
async fn test_move_category_exceeds_max_depth() {
    let (ctx, repo) = common::category_share::create_sqlite_category_repo().await;
    common::category_share::category_test_move_exceeds_max_depth(&ctx, repo).await;
}

#[tokio::test]
async fn test_move_category_within_depth_limit() {
    let (ctx, repo) = common::category_share::create_sqlite_category_repo().await;
    common::category_share::category_test_move_within_depth_limit(&ctx, repo).await;
}

// =============================================================================
// Soft Delete Tests
// =============================================================================

#[tokio::test]
async fn test_deleted_category_not_in_get_all() {
    let (ctx, repo) = common::category_share::create_sqlite_category_repo().await;
    common::category_share::category_test_deleted_not_in_get_all(&ctx, repo).await;
}

#[tokio::test]
async fn test_deleted_child_not_returned() {
    let (ctx, repo) = common::category_share::create_sqlite_category_repo().await;
    common::category_share::category_test_deleted_child_not_returned(&ctx, repo).await;
}

#[tokio::test]
async fn test_cannot_delete_already_deleted() {
    let (ctx, repo) = common::category_share::create_sqlite_category_repo().await;
    common::category_share::category_test_cannot_delete_already_deleted(&ctx, repo).await;
}

#[tokio::test]
async fn test_cannot_update_deleted_category() {
    let (ctx, repo) = common::category_share::create_sqlite_category_repo().await;
    common::category_share::category_test_cannot_update_deleted(&ctx, repo).await;
}

// =============================================================================
// Edge Cases
// =============================================================================

#[tokio::test]
async fn test_get_child_category_by_id() {
    let (ctx, repo) = common::category_share::create_sqlite_category_repo().await;
    common::category_share::category_test_get_child_by_id(&ctx, repo).await;
}

#[tokio::test]
async fn test_category_without_children() {
    let (ctx, repo) = common::category_share::create_sqlite_category_repo().await;
    common::category_share::category_test_without_children(&ctx, repo).await;
}

#[tokio::test]
async fn test_deep_nested_tree_retrieval() {
    let (ctx, repo) = common::category_share::create_sqlite_category_repo().await;
    common::category_share::category_test_deep_nested_tree_retrieval(&ctx, repo).await;
}
