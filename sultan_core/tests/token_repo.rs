mod common;

use chrono::{Duration, Utc};
use sultan_core::domain::Context;
use sultan_core::domain::model::token::{self, Token};
use sultan_core::domain::model::user::{self, UserCreate};
use sultan_core::snowflake::SnowflakeGenerator;
use sultan_core::storage::sqlite::token::SqliteTokenRepository;
use sultan_core::storage::sqlite::user::SqliteUserRepository;
use sultan_core::storage::token_repo::TokenRepository;
use sultan_core::storage::user_repo::UserRepository;

use common::init_sqlite_pool;

fn generate_test_id() -> i64 {
    thread_local! {
        static GENERATOR: SnowflakeGenerator = SnowflakeGenerator::new(1).unwrap();
    }
    GENERATOR.with(|g| g.generate().unwrap())
}

/// Helper to create a test user (required due to foreign key constraint)
async fn create_test_user(user_repo: &SqliteUserRepository, ctx: &Context) -> i64 {
    let user_id = generate_test_id();
    let user = UserCreate {
        username: format!("token_test_user_{}", user_id),
        name: "Token Test User".to_string(),
        email: Some(format!("token_test_{}@example.com", user_id)),
        password: "hashed_password".to_string(),
        photo: None,
        pin: None,
        address: None,
        phone: None,
    };

    user_repo
        .create_user(ctx, user_id, &user)
        .await
        .expect("Failed to create test user");

    user_id
}

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
