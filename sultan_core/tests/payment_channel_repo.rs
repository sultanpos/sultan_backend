mod common;
use sultan_core::storage::sqlite::SqlitePaymentChannelRepository;
use sultan_core::testing::storage::payment_channel;

#[tokio::test]
async fn test_payment_channel_repo_integration() {
    let repo = SqlitePaymentChannelRepository::new();
    payment_channel::payment_channel_test_all(&repo, || async {
        common::init_sqlite_repo_ctx().await
    })
    .await;
}
