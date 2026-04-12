mod common;
use sultan_core::testing::storage::purchase_order;

#[tokio::test]
async fn test_purchase_order_repo_integration() {
    let repo = sultan_core::storage::sqlite::SqlitePurchaseOrderRepository::new();
    purchase_order::purchase_order_test_all(&repo, || async {
        common::init_sqlite_repo_ctx().await
    })
    .await;
}
