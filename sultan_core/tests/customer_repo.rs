mod common;
use sultan_core::testing::storage::customer;

// =============================================================================
// Basic CRUD Tests
// =============================================================================

#[tokio::test]
async fn test_customer_repo_integration() {
    let repo = sultan_core::storage::sqlite::customer::SqliteCustomerRepository::new();
    let ctx = common::init_sqlite_repo_ctx().await;
    customer::customer_test_repo_integration(&ctx, &repo).await;
}

#[tokio::test]
async fn test_create_customer_with_all_fields() {
    let repo = sultan_core::storage::sqlite::customer::SqliteCustomerRepository::new();
    let ctx = common::init_sqlite_repo_ctx().await;
    customer::customer_test_create_with_all_fields(&ctx, &repo).await;
}

#[tokio::test]
async fn test_create_customer_minimal_fields() {
    let repo = sultan_core::storage::sqlite::customer::SqliteCustomerRepository::new();
    let ctx = common::init_sqlite_repo_ctx().await;
    customer::customer_test_create_minimal_fields(&ctx, &repo).await;
}

// =============================================================================
// Update Tests
// =============================================================================

#[tokio::test]
async fn test_partial_update_customer() {
    let repo = sultan_core::storage::sqlite::customer::SqliteCustomerRepository::new();
    let ctx = common::init_sqlite_repo_ctx().await;
    customer::customer_test_partial_update(&ctx, &repo).await;
}

#[tokio::test]
async fn test_update_address_scenarios() {
    let repo = sultan_core::storage::sqlite::customer::SqliteCustomerRepository::new();
    let ctx = common::init_sqlite_repo_ctx().await;
    customer::customer_test_update_address_scenarios(&ctx, &repo).await;
}

#[tokio::test]
async fn test_update_metadata() {
    let repo = sultan_core::storage::sqlite::customer::SqliteCustomerRepository::new();
    let ctx = common::init_sqlite_repo_ctx().await;
    customer::customer_test_update_metadata(&ctx, &repo).await;
}

#[tokio::test]
async fn test_update_email_scenarios() {
    let repo = sultan_core::storage::sqlite::customer::SqliteCustomerRepository::new();
    let ctx = common::init_sqlite_repo_ctx().await;
    customer::customer_test_update_email_scenarios(&ctx, &repo).await;
}

#[tokio::test]
async fn test_update_level() {
    let repo = sultan_core::storage::sqlite::customer::SqliteCustomerRepository::new();
    let ctx = common::init_sqlite_repo_ctx().await;
    customer::customer_test_update_level(&ctx, &repo).await;
}

#[tokio::test]
async fn test_update_non_existent_customer() {
    let repo = sultan_core::storage::sqlite::customer::SqliteCustomerRepository::new();
    let ctx = common::init_sqlite_repo_ctx().await;
    customer::customer_test_update_non_existent(&ctx, &repo).await;
}

// =============================================================================
// Delete Tests
// =============================================================================

#[tokio::test]
async fn test_delete_non_existent_customer() {
    let repo = sultan_core::storage::sqlite::customer::SqliteCustomerRepository::new();
    let ctx = common::init_sqlite_repo_ctx().await;
    customer::customer_test_delete_non_existent(&ctx, &repo).await;
}

#[tokio::test]
async fn test_get_deleted_customer() {
    let repo = sultan_core::storage::sqlite::customer::SqliteCustomerRepository::new();
    let ctx = common::init_sqlite_repo_ctx().await;
    customer::customer_test_get_deleted(&ctx, &repo).await;
}

#[tokio::test]
async fn test_deleted_customer_not_in_get_all() {
    let repo = sultan_core::storage::sqlite::customer::SqliteCustomerRepository::new();
    let ctx = common::init_sqlite_repo_ctx().await;
    customer::customer_test_deleted_not_in_get_all(&ctx, &repo).await;
}

// =============================================================================
// Get Tests
// =============================================================================

#[tokio::test]
async fn test_get_by_number_success() {
    let repo = sultan_core::storage::sqlite::customer::SqliteCustomerRepository::new();
    let ctx = common::init_sqlite_repo_ctx().await;
    customer::customer_test_get_by_number_success(&ctx, &repo).await;
}

#[tokio::test]
async fn test_get_by_number_not_found() {
    let repo = sultan_core::storage::sqlite::customer::SqliteCustomerRepository::new();
    let ctx = common::init_sqlite_repo_ctx().await;
    customer::customer_test_get_by_number_not_found(&ctx, &repo).await;
}

#[tokio::test]
async fn test_get_by_number_deleted_customer() {
    let repo = sultan_core::storage::sqlite::customer::SqliteCustomerRepository::new();
    let ctx = common::init_sqlite_repo_ctx().await;
    customer::customer_test_get_by_number_deleted(&ctx, &repo).await;
}

#[tokio::test]
async fn test_get_by_number_case_sensitive() {
    let repo = sultan_core::storage::sqlite::customer::SqliteCustomerRepository::new();
    let ctx = common::init_sqlite_repo_ctx().await;
    customer::customer_test_get_by_number_case_sensitive(&ctx, &repo).await;
}

#[tokio::test]
async fn test_get_by_id_not_found() {
    let repo = sultan_core::storage::sqlite::customer::SqliteCustomerRepository::new();
    let ctx = common::init_sqlite_repo_ctx().await;
    customer::customer_test_get_by_id_not_found(&ctx, &repo).await;
}

#[tokio::test]
async fn test_get_all_customers() {
    let repo = sultan_core::storage::sqlite::customer::SqliteCustomerRepository::new();
    let ctx = common::init_sqlite_repo_ctx().await;
    customer::customer_test_get_all(&ctx, &repo).await;
}

// =============================================================================
// Filter Tests
// =============================================================================

#[tokio::test]
async fn test_filter_by_name() {
    let repo = sultan_core::storage::sqlite::customer::SqliteCustomerRepository::new();
    let ctx = common::init_sqlite_repo_ctx().await;
    customer::customer_test_filter_by_name(&ctx, &repo).await;
}

#[tokio::test]
async fn test_filter_by_number() {
    let repo = sultan_core::storage::sqlite::customer::SqliteCustomerRepository::new();
    let ctx = common::init_sqlite_repo_ctx().await;
    customer::customer_test_filter_by_number(&ctx, &repo).await;
}

#[tokio::test]
async fn test_filter_by_email() {
    let repo = sultan_core::storage::sqlite::customer::SqliteCustomerRepository::new();
    let ctx = common::init_sqlite_repo_ctx().await;
    customer::customer_test_filter_by_email(&ctx, &repo).await;
}

#[tokio::test]
async fn test_filter_by_phone() {
    let repo = sultan_core::storage::sqlite::customer::SqliteCustomerRepository::new();
    let ctx = common::init_sqlite_repo_ctx().await;
    customer::customer_test_filter_by_phone(&ctx, &repo).await;
}

#[tokio::test]
async fn test_filter_by_level() {
    let repo = sultan_core::storage::sqlite::customer::SqliteCustomerRepository::new();
    let ctx = common::init_sqlite_repo_ctx().await;
    customer::customer_test_filter_by_level(&ctx, &repo).await;
}

#[tokio::test]
async fn test_filter_multiple_criteria() {
    let repo = sultan_core::storage::sqlite::customer::SqliteCustomerRepository::new();
    let ctx = common::init_sqlite_repo_ctx().await;
    customer::customer_test_filter_multiple_criteria(&ctx, &repo).await;
}

// =============================================================================
// Pagination Tests
// =============================================================================

#[tokio::test]
async fn test_pagination() {
    let repo = sultan_core::storage::sqlite::customer::SqliteCustomerRepository::new();
    let ctx = common::init_sqlite_repo_ctx().await;
    customer::customer_test_pagination(&ctx, &repo).await;
}
