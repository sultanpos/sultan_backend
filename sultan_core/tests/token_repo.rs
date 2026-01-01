mod common;

#[tokio::test]
async fn test_save_and_get_token() {
    let (ctx, token_repo, user_repo) =
        common::token_share::create_sqlite_user_and_token_repo().await;
    common::token_share::token_test_save_and_get_token(&ctx, token_repo, user_repo).await;
}

#[tokio::test]
async fn test_get_token_not_found() {
    let (ctx, token_repo, _) = common::token_share::create_sqlite_user_and_token_repo().await;
    common::token_share::token_test_get_token_not_found(&ctx, token_repo).await;
}

#[tokio::test]
async fn test_delete_token() {
    let (ctx, token_repo, user_repo) =
        common::token_share::create_sqlite_user_and_token_repo().await;
    common::token_share::token_test_delete_token(&ctx, token_repo, user_repo).await;
}

#[tokio::test]
async fn test_delete_token_not_found() {
    let (ctx, token_repo, _) = common::token_share::create_sqlite_user_and_token_repo().await;
    common::token_share::token_test_delete_token_not_found(&ctx, token_repo).await;
}

#[tokio::test]
async fn test_multiple_tokens_same_user() {
    let (ctx, token_repo, user_repo) =
        common::token_share::create_sqlite_user_and_token_repo().await;
    common::token_share::token_test_multiple_tokens_same_user(&ctx, token_repo, user_repo).await;
}

#[tokio::test]
async fn test_token_with_expired_time() {
    let (ctx, token_repo, user_repo) =
        common::token_share::create_sqlite_user_and_token_repo().await;
    common::token_share::token_test_token_with_expired_time(&ctx, token_repo, user_repo).await;
}
