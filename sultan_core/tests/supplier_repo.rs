mod common;

// =============================================================================
// Basic CRUD Tests
// =============================================================================

#[tokio::test]
async fn test_supplier_repo_integration() {
    let (ctx, repo) = common::supplier_share::create_sqlite_supplier_repo().await;
    common::supplier_share::supplier_test_repo_integration(&ctx, repo).await;
}

#[tokio::test]
async fn test_create_supplier_with_all_fields() {
    let (ctx, repo) = common::supplier_share::create_sqlite_supplier_repo().await;
    common::supplier_share::supplier_test_create_with_all_fields(&ctx, repo).await;
}

#[tokio::test]
async fn test_create_supplier_minimal_fields() {
    let (ctx, repo) = common::supplier_share::create_sqlite_supplier_repo().await;
    common::supplier_share::supplier_test_create_minimal_fields(&ctx, repo).await;
}

// =============================================================================
// Update Tests
// =============================================================================

#[tokio::test]
async fn test_partial_update_supplier() {
    let (ctx, repo) = common::supplier_share::create_sqlite_supplier_repo().await;
    common::supplier_share::supplier_test_partial_update(&ctx, repo).await;
}

#[tokio::test]
async fn test_update_address_scenarios() {
    let (ctx, repo) = common::supplier_share::create_sqlite_supplier_repo().await;
    common::supplier_share::supplier_test_update_address_scenarios(&ctx, repo).await;
}

#[tokio::test]
async fn test_update_metadata() {
    let (ctx, repo) = common::supplier_share::create_sqlite_supplier_repo().await;
    common::supplier_share::supplier_test_update_metadata(&ctx, repo).await;
}

#[tokio::test]
async fn test_update_email_scenarios() {
    let (ctx, repo) = common::supplier_share::create_sqlite_supplier_repo().await;
    common::supplier_share::supplier_test_update_email_scenarios(&ctx, repo).await;
}

#[tokio::test]
async fn test_update_non_existent_supplier() {
    let (ctx, repo) = common::supplier_share::create_sqlite_supplier_repo().await;
    common::supplier_share::supplier_test_update_non_existent(&ctx, repo).await;
}

// =============================================================================
// Delete Tests
// =============================================================================

#[tokio::test]
async fn test_delete_non_existent_supplier() {
    let (ctx, repo) = common::supplier_share::create_sqlite_supplier_repo().await;
    common::supplier_share::supplier_test_delete_non_existent(&ctx, repo).await;
}

#[tokio::test]
async fn test_get_deleted_supplier() {
    let (ctx, repo) = common::supplier_share::create_sqlite_supplier_repo().await;
    common::supplier_share::supplier_test_get_deleted(&ctx, repo).await;
}

#[tokio::test]
async fn test_deleted_supplier_not_in_get_all() {
    let (ctx, repo) = common::supplier_share::create_sqlite_supplier_repo().await;
    common::supplier_share::supplier_test_deleted_not_in_get_all(&ctx, repo).await;
}

// =============================================================================
// Get Tests
// =============================================================================

#[tokio::test]
async fn test_get_by_id_not_found() {
    let (ctx, repo) = common::supplier_share::create_sqlite_supplier_repo().await;
    common::supplier_share::supplier_test_get_by_id_not_found(&ctx, repo).await;
}

#[tokio::test]
async fn test_get_all_suppliers() {
    let (ctx, repo) = common::supplier_share::create_sqlite_supplier_repo().await;
    common::supplier_share::supplier_test_get_all(&ctx, repo).await;
}

// =============================================================================
// Filter Tests
// =============================================================================

#[tokio::test]
async fn test_filter_by_name() {
    let (ctx, repo) = common::supplier_share::create_sqlite_supplier_repo().await;
    common::supplier_share::supplier_test_filter_by_name(&ctx, repo).await;
}

#[tokio::test]
async fn test_filter_by_code() {
    let (ctx, repo) = common::supplier_share::create_sqlite_supplier_repo().await;
    common::supplier_share::supplier_test_filter_by_code(&ctx, repo).await;
}

#[tokio::test]
async fn test_filter_by_email() {
    let (ctx, repo) = common::supplier_share::create_sqlite_supplier_repo().await;
    common::supplier_share::supplier_test_filter_by_email(&ctx, repo).await;
}

#[tokio::test]
async fn test_filter_by_phone() {
    let (ctx, repo) = common::supplier_share::create_sqlite_supplier_repo().await;
    common::supplier_share::supplier_test_filter_by_phone(&ctx, repo).await;
}

#[tokio::test]
async fn test_filter_by_npwp() {
    let (ctx, repo) = common::supplier_share::create_sqlite_supplier_repo().await;
    common::supplier_share::supplier_test_filter_by_npwp(&ctx, repo).await;
}

#[tokio::test]
async fn test_filter_multiple_criteria() {
    let (ctx, repo) = common::supplier_share::create_sqlite_supplier_repo().await;
    common::supplier_share::supplier_test_filter_multiple_criteria(&ctx, repo).await;
}

// =============================================================================
// Pagination Tests
// =============================================================================

#[tokio::test]
async fn test_pagination() {
    let (ctx, repo) = common::supplier_share::create_sqlite_supplier_repo().await;
    common::supplier_share::supplier_test_pagination(&ctx, repo).await;
}
