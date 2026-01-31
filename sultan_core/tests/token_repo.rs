mod common;
use sultan_core::testing::storage::token;

#[tokio::test]
async fn test_token_repo_integration() {
    let token_repo = sultan_core::storage::sqlite::token::SqliteTokenRepository::new();

    token::token_test_all(&token_repo, || async {
        let ctx = common::init_sqlite_repo_ctx().await;
        let user_repo = sultan_core::storage::sqlite::user::SqliteUserRepository::new();
        (ctx, user_repo)
    })
    .await;
}
