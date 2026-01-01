mod common;

use crate::common::product_share::create_sqlite_product_repo;

// =============================================================================
// Product CRUD Tests
// =============================================================================

#[tokio::test]
async fn test_create_product_success() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::product_test_create_success(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_create_product_without_optional_fields() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_create_product_without_optional_fields(&ctx, &tx_manager, &repo)
        .await;
}

#[tokio::test]
async fn test_create_product_with_categories() {
    let (ctx, tx_manager, repo, pool) = create_sqlite_product_repo().await;
    common::product_share::test_create_product_with_categories(&ctx, &tx_manager, &repo, &pool)
        .await;
}

#[tokio::test]
async fn test_update_product_name() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_create_product_without_optional_fields(&ctx, &tx_manager, &repo)
        .await;
    common::product_share::test_update_product_name(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_product_clear_description() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_create_product_without_optional_fields(&ctx, &tx_manager, &repo)
        .await;
    common::product_share::test_update_product_clear_description(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_product_all_fields() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_create_product_without_optional_fields(&ctx, &tx_manager, &repo)
        .await;
    common::product_share::test_update_product_all_fields(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_product_categories() {
    let (ctx, tx_manager, repo, pool) = create_sqlite_product_repo().await;
    common::product_share::test_create_product_without_optional_fields(&ctx, &tx_manager, &repo)
        .await;
    common::product_share::test_update_product_categories(&ctx, &tx_manager, &repo, &pool).await;
}

#[tokio::test]
async fn test_update_product_not_found() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_update_product_not_found(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_delete_product_success() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_delete_product_success(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_delete_product_not_found() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_delete_product_not_found(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_get_product_by_id_not_found() {
    let (ctx, _, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_get_product_by_id_not_found(&ctx, &repo).await;
}

// =============================================================================
// Product Variant CRUD Tests
// =============================================================================

#[tokio::test]
async fn test_create_variant_success() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_create_variant_success(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_create_variant_without_optional_fields() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_create_variant_without_optional_fields(&ctx, &tx_manager, &repo)
        .await;
}

#[tokio::test]
async fn test_update_variant_barcode() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_update_variant_barcode(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_variant_clear_name() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_update_variant_clear_name(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_variant_all_fields() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_update_variant_all_fields(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_variant_not_found() {
    let (ctx, _, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_update_variant_not_found(&ctx, &repo).await;
}

#[tokio::test]
async fn test_delete_variant_success() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_delete_variant_success(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_delete_variant_not_found() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_delete_variant_not_found(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_delete_variants_by_product_id() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_delete_variants_by_product_id(&ctx, &tx_manager, &repo).await;
}

// =============================================================================
// Variant Query Tests
// =============================================================================

#[tokio::test]
async fn test_get_variant_by_barcode_success() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_get_variant_by_barcode_success(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_get_variant_by_barcode_not_found() {
    let (ctx, _, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_get_variant_by_barcode_not_found(&ctx, &repo).await;
}

#[tokio::test]
async fn test_get_variant_by_id_not_found() {
    let (ctx, _, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_get_variant_by_id_not_found(&ctx, &repo).await;
}

#[tokio::test]
async fn test_get_variant_by_product_id_success() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_get_variant_by_product_id_success(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_get_variant_by_product_id_empty() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_get_variant_by_product_id_empty(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_get_variant_by_product_id_product_not_found() {
    let (ctx, _, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_get_variant_by_product_id_product_not_found(&ctx, &repo).await;
}

// =============================================================================
// Transaction Tests
// =============================================================================

#[tokio::test]
async fn test_transaction_rollback_product_creation() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_transaction_rollback_product_creation(&ctx, &tx_manager, &repo)
        .await;
}

#[tokio::test]
async fn test_transaction_rollback_variant_creation() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_transaction_rollback_variant_creation(&ctx, &tx_manager, &repo)
        .await;
}

#[tokio::test]
async fn test_transaction_product_and_variant_atomic() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_transaction_product_and_variant_atomic(&ctx, &tx_manager, &repo)
        .await;
}

// =============================================================================
// Soft Delete Verification Tests
// =============================================================================

#[tokio::test]
async fn test_soft_delete_product_preserves_data() {
    let (ctx, tx_manager, repo, pool) = create_sqlite_product_repo().await;
    common::product_share::test_soft_delete_product_preserves_data(&ctx, &tx_manager, &repo, &pool)
        .await;
}

#[tokio::test]
async fn test_soft_delete_variant_preserves_data() {
    let (ctx, tx_manager, repo, pool) = create_sqlite_product_repo().await;
    common::product_share::test_soft_delete_variant_preserves_data(&ctx, &tx_manager, &repo, &pool)
        .await;
}

// =============================================================================
// Edge Cases
// =============================================================================

#[tokio::test]
async fn test_product_with_metadata_json() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_product_with_metadata_json(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_deleted_product_fails() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_update_deleted_product_fails(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_deleted_variant_fails() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_update_deleted_variant_fails(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_delete_already_deleted_product_fails() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_delete_already_deleted_product_fails(&ctx, &tx_manager, &repo)
        .await;
}

// =============================================================================
// Additional Coverage Tests
// =============================================================================

#[tokio::test]
async fn test_get_variant_by_id_success() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_get_variant_by_id_success(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_get_variant_by_id_when_product_deleted() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_get_variant_by_id_when_product_deleted(&ctx, &tx_manager, &repo)
        .await;
}

#[tokio::test]
async fn test_get_variant_by_barcode_when_product_deleted() {
    let (ctx, tx_manager, repo, _) = create_sqlite_product_repo().await;
    common::product_share::test_get_variant_by_barcode_when_product_deleted(
        &ctx,
        &tx_manager,
        &repo,
    )
    .await;
}

#[tokio::test]
async fn test_update_product_with_empty_category_update() {
    let (ctx, tx_manager, repo, pool) = common::product_share::create_sqlite_product_repo().await;
    common::product_share::test_update_product_with_empty_category_update(
        &ctx,
        &tx_manager,
        &repo,
        &pool,
    )
    .await;
}

#[tokio::test]
async fn test_update_product_replace_categories() {
    let (ctx, tx_manager, repo, pool) = common::product_share::create_sqlite_product_repo().await;
    common::product_share::test_update_product_replace_categories(&ctx, &tx_manager, &repo, &pool)
        .await;
}

#[tokio::test]
async fn test_update_product_only_metadata() {
    let (ctx, tx_manager, repo, _pool) = common::product_share::create_sqlite_product_repo().await;
    common::product_share::test_update_product_only_metadata(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_product_clear_metadata() {
    let (ctx, tx_manager, repo, _pool) = common::product_share::create_sqlite_product_repo().await;
    common::product_share::test_update_product_clear_metadata(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_variant_clear_metadata() {
    let (ctx, tx_manager, repo, _pool) = common::product_share::create_sqlite_product_repo().await;
    common::product_share::test_update_variant_clear_metadata(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_variant_clear_barcode() {
    let (ctx, tx_manager, repo, _pool) = create_sqlite_product_repo().await;
    common::product_share::test_update_variant_clear_barcode(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_multiple_variants_for_single_product() {
    let (ctx, tx_manager, repo, _pool) = create_sqlite_product_repo().await;
    common::product_share::test_multiple_variants_for_single_product(&ctx, &tx_manager, &repo)
        .await;
}

#[tokio::test]
async fn test_delete_variants_by_product_id_preserves_other_products() {
    let (ctx, tx_manager, repo, _pool) = create_sqlite_product_repo().await;
    common::product_share::test_delete_variants_by_product_id_preserves_other_products(
        &ctx,
        &tx_manager,
        &repo,
    )
    .await;
}

#[tokio::test]
async fn test_update_product_boolean_flags() {
    let (ctx, tx_manager, repo, _pool) = create_sqlite_product_repo().await;
    common::product_share::test_update_product_boolean_flags(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_product_main_image() {
    let (ctx, tx_manager, repo, _pool) = create_sqlite_product_repo().await;
    common::product_share::test_update_product_main_image(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_product_clear_main_image() {
    let (ctx, tx_manager, repo, _pool) = create_sqlite_product_repo().await;
    common::product_share::test_update_product_clear_main_image(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_product_type() {
    let (ctx, tx_manager, repo, _pool) = create_sqlite_product_repo().await;
    common::product_share::test_update_product_type(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_variant_without_barcode() {
    let (ctx, tx_manager, repo, _pool) = create_sqlite_product_repo().await;
    common::product_share::test_variant_without_barcode(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_get_deleted_variant_returns_none() {
    let (ctx, tx_manager, repo, _pool) = create_sqlite_product_repo().await;
    common::product_share::test_get_deleted_variant_returns_none(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_get_deleted_product_returns_none() {
    let (ctx, tx_manager, repo, _pool) = create_sqlite_product_repo().await;
    common::product_share::test_get_deleted_product_returns_none(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_variant_only_name() {
    let (ctx, tx_manager, repo, _pool) = create_sqlite_product_repo().await;
    common::product_share::test_update_variant_only_name(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_variant_only_barcode() {
    let (ctx, tx_manager, repo, _pool) = create_sqlite_product_repo().await;
    common::product_share::test_update_variant_only_barcode(&ctx, &tx_manager, &repo).await;
}

#[tokio::test]
async fn test_update_variant_set_metadata() {
    let (ctx, tx_manager, repo, _pool) = create_sqlite_product_repo().await;
    common::product_share::test_update_variant_set_metadata(&ctx, &tx_manager, &repo).await;
}
