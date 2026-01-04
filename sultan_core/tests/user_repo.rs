use sultan_core::testing::storage::user;

// =============================================================================
// Basic CRUD Tests
// =============================================================================

#[tokio::test]
async fn test_create_and_get_user_integration() {
    let (ctx, repo) = user::create_sqlite_user_repo().await;
    user::user_test_create_and_get_integration(&ctx, repo).await;
}

#[tokio::test]
async fn user_test_create_and_get_integration_tx() {
    let (ctx, repo, tx_manager) = user::create_sqlite_user_repo_tx().await;
    user::user_test_create_and_get_integration_tx(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn user_test_delete_tx() {
    let (ctx, repo, tx_manager) = user::create_sqlite_user_repo_tx().await;
    user::user_test_delete_tx(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_create_duplicate_user() {
    let (ctx, repo) = user::create_sqlite_user_repo().await;
    user::user_test_create_duplicate(&ctx, repo).await;
}

#[tokio::test]
async fn test_update_user() {
    let (ctx, repo) = user::create_sqlite_user_repo().await;
    user::user_test_update(&ctx, repo).await;
}

#[tokio::test]
async fn test_update_user_not_found() {
    let (ctx, repo) = user::create_sqlite_user_repo().await;
    user::user_test_update_not_found(&ctx, repo).await;
}

#[tokio::test]
async fn test_update_password() {
    let (ctx, repo) = user::create_sqlite_user_repo().await;
    user::user_test_update_password(&ctx, repo).await;
}

#[tokio::test]
async fn test_delete_user() {
    let (ctx, repo) = user::create_sqlite_user_repo().await;
    user::user_test_delete(&ctx, repo).await;
}

// =============================================================================
// Pagination Tests
// =============================================================================

#[tokio::test]
async fn test_get_all_users_pagination() {
    let (ctx, repo) = user::create_sqlite_user_repo().await;
    user::user_test_get_all_pagination(&ctx, repo).await;
}

// =============================================================================
// Filter Tests
// =============================================================================

#[tokio::test]
async fn test_get_all_users_filter_by_username() {
    let (ctx, repo) = user::create_sqlite_user_repo().await;
    user::user_test_filter_by_username(&ctx, repo).await;
}

#[tokio::test]
async fn test_get_all_users_filter_by_name() {
    let (ctx, repo) = user::create_sqlite_user_repo().await;
    user::user_test_filter_by_name(&ctx, repo).await;
}

#[tokio::test]
async fn test_get_all_users_filter_combined() {
    let (ctx, repo) = user::create_sqlite_user_repo().await;
    user::user_test_filter_combined(&ctx, repo).await;
}

#[tokio::test]
async fn test_get_all_users_filter_by_email() {
    let (ctx, repo) = user::create_sqlite_user_repo().await;
    user::user_test_filter_by_email(&ctx, repo).await;
}

// =============================================================================
// Get Tests
// =============================================================================

#[tokio::test]
async fn test_get_user_by_id() {
    let (ctx, repo) = user::create_sqlite_user_repo().await;
    user::user_test_get_by_id(&ctx, repo).await;
}

#[tokio::test]
async fn test_get_user_by_id_not_found() {
    let (ctx, repo) = user::create_sqlite_user_repo().await;
    user::user_test_get_by_id_not_found(&ctx, repo).await;
}

#[tokio::test]
async fn test_delete_user_not_found() {
    let (ctx, repo) = user::create_sqlite_user_repo().await;
    user::user_test_delete_not_found(&ctx, repo).await;
}

#[tokio::test]
async fn test_update_password_not_found() {
    let (ctx, repo) = user::create_sqlite_user_repo().await;
    user::user_test_update_password_not_found(&ctx, repo).await;
}

#[tokio::test]
async fn test_get_user_by_username_not_found() {
    let (ctx, repo) = user::create_sqlite_user_repo().await;
    user::user_test_get_by_username_not_found(&ctx, repo).await;
}

// =============================================================================
// Permission Tests
// =============================================================================

#[tokio::test]
async fn test_save_user_permission_with_branch() {
    let (ctx, repo) = user::create_sqlite_user_repo().await;
    user::user_test_save_permission_with_branch(&ctx, repo).await;
}

#[tokio::test]
async fn test_save_user_permission_without_branch() {
    let (ctx, repo) = user::create_sqlite_user_repo().await;
    user::user_test_save_permission_without_branch(&ctx, repo).await;
}

#[tokio::test]
async fn test_save_multiple_permissions() {
    let (ctx, repo) = user::create_sqlite_user_repo().await;
    user::user_test_save_multiple_permissions(&ctx, repo).await;
}

#[tokio::test]
async fn test_delete_user_permission_with_branch() {
    let (ctx, repo) = user::create_sqlite_user_repo().await;
    user::user_test_delete_permission_with_branch(&ctx, repo).await;
}

#[tokio::test]
async fn test_delete_user_permission_without_branch() {
    let (ctx, repo) = user::create_sqlite_user_repo().await;
    user::user_test_delete_permission_without_branch(&ctx, repo).await;
}

#[tokio::test]
async fn test_delete_specific_permission_keeps_others() {
    let (ctx, repo) = user::create_sqlite_user_repo().await;
    user::user_test_delete_specific_permission_keeps_others(&ctx, repo).await;
}

#[tokio::test]
async fn test_get_user_permission_not_found() {
    let (ctx, repo) = user::create_sqlite_user_repo().await;
    user::user_test_get_permission_not_found(&ctx, repo).await;
}

#[tokio::test]
async fn test_save_permission_null_branch_then_delete() {
    let (ctx, repo) = user::create_sqlite_user_repo().await;
    user::user_test_save_permission_null_branch_then_delete(&ctx, repo).await;
}

#[tokio::test]
async fn test_save_and_update_permission_null_branch() {
    let (ctx, repo) = user::create_sqlite_user_repo().await;
    user::user_test_save_and_update_permission_null_branch(&ctx, repo).await;
}

#[tokio::test]
async fn test_delete_permission_null_vs_non_null_branch() {
    let (ctx, repo) = user::create_sqlite_user_repo().await;
    user::user_test_delete_permission_null_vs_non_null_branch(&ctx, repo).await;
}
