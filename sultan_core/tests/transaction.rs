mod common;

use common::init_sqlite_pool;
use sultan_core::domain::Context;
use sultan_core::snowflake::SnowflakeGenerator;
use sultan_core::storage::UserRepository;
use sultan_core::storage::sqlite::transaction::SqliteTransactionManager;
use sultan_core::storage::sqlite::user::SqliteUserRepository;
use sultan_core::storage::transaction::TransactionManager;

fn generate_test_id() -> i64 {
    thread_local! {
        static GENERATOR: SnowflakeGenerator = SnowflakeGenerator::new(1).unwrap();
    }
    GENERATOR.with(|g| g.generate().unwrap())
}

// =============================================================================
// Transaction Basic Operations Tests
// =============================================================================

#[tokio::test]
async fn test_begin_transaction() {
    let pool = init_sqlite_pool().await;
    let tx_manager = SqliteTransactionManager::new(pool);

    let tx = tx_manager.begin().await;
    assert!(tx.is_ok());
}

#[tokio::test]
async fn test_commit_transaction() {
    let pool = init_sqlite_pool().await;
    let tx_manager = SqliteTransactionManager::new(pool.clone());
    let user_repo = SqliteUserRepository::new(pool);
    let ctx = Context::new();

    let user_id = generate_test_id();

    // Begin transaction
    let mut tx = tx_manager
        .begin()
        .await
        .expect("Failed to begin transaction");

    // Insert user within transaction
    sqlx::query(
        r#"
        INSERT INTO users (id, username, password, name, email, phone)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_id)
    .bind("testuser")
    .bind("hash123")
    .bind("Test User")
    .bind(Some("test@example.com"))
    .bind(Some("1234567890"))
    .execute(&mut *tx)
    .await
    .expect("Failed to insert user");

    // Commit transaction
    tx_manager
        .commit(tx)
        .await
        .expect("Failed to commit transaction");

    // Verify user was created
    let user = user_repo
        .get_by_id(&ctx, user_id)
        .await
        .expect("Failed to get user");
    assert!(user.is_some());
    assert_eq!(user.unwrap().username, "testuser");
}

#[tokio::test]
async fn test_rollback_transaction() {
    let pool = init_sqlite_pool().await;
    let tx_manager = SqliteTransactionManager::new(pool.clone());
    let user_repo = SqliteUserRepository::new(pool);
    let ctx = Context::new();

    let user_id = generate_test_id();

    // Begin transaction
    let mut tx = tx_manager
        .begin()
        .await
        .expect("Failed to begin transaction");

    // Insert user within transaction
    sqlx::query(
        r#"
        INSERT INTO users (id, username, password, name, email, phone)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_id)
    .bind("testuser_rollback")
    .bind("hash123")
    .bind("Test User Rollback")
    .bind(Some("rollback@example.com"))
    .bind(Some("9876543210"))
    .execute(&mut *tx)
    .await
    .expect("Failed to insert user");

    // Rollback transaction
    tx_manager
        .rollback(tx)
        .await
        .expect("Failed to rollback transaction");

    // Verify user was NOT created
    let user = user_repo
        .get_by_id(&ctx, user_id)
        .await
        .expect("Failed to get user");
    assert!(user.is_none());
}

#[tokio::test]
async fn test_implicit_rollback_on_drop() {
    let pool = init_sqlite_pool().await;
    let tx_manager = SqliteTransactionManager::new(pool.clone());
    let user_repo = SqliteUserRepository::new(pool);
    let ctx = Context::new();

    let user_id = generate_test_id();

    {
        // Begin transaction
        let mut tx = tx_manager
            .begin()
            .await
            .expect("Failed to begin transaction");

        // Insert user within transaction
        sqlx::query(
            r#"
            INSERT INTO users (id, username, password, name, email, phone)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(user_id)
        .bind("testuser_drop")
        .bind("hash123")
        .bind("Test User Drop")
        .bind(Some("drop@example.com"))
        .bind(Some("1111111111"))
        .execute(&mut *tx)
        .await
        .expect("Failed to insert user");

        // Transaction dropped here without commit - should rollback
    }

    // Verify user was NOT created (implicit rollback)
    let user = user_repo
        .get_by_id(&ctx, user_id)
        .await
        .expect("Failed to get user");
    assert!(user.is_none());
}

// =============================================================================
// Transaction with Multiple Operations
// =============================================================================

#[tokio::test]
async fn test_multiple_operations_in_transaction() {
    let pool = init_sqlite_pool().await;
    let tx_manager = SqliteTransactionManager::new(pool.clone());
    let user_repo = SqliteUserRepository::new(pool);
    let ctx = Context::new();

    let user_id1 = generate_test_id();
    let user_id2 = generate_test_id();

    // Begin transaction
    let mut tx = tx_manager
        .begin()
        .await
        .expect("Failed to begin transaction");

    // Insert first user
    sqlx::query(
        r#"
        INSERT INTO users (id, username, password, name, email, phone)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_id1)
    .bind("user1")
    .bind("hash1")
    .bind("User One")
    .bind(Some("user1@example.com"))
    .bind(Some("1111111111"))
    .execute(&mut *tx)
    .await
    .expect("Failed to insert user1");

    // Insert second user
    sqlx::query(
        r#"
        INSERT INTO users (id, username, password, name, email, phone)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_id2)
    .bind("user2")
    .bind("hash2")
    .bind("User Two")
    .bind(Some("user2@example.com"))
    .bind(Some("2222222222"))
    .execute(&mut *tx)
    .await
    .expect("Failed to insert user2");

    // Commit transaction
    tx_manager
        .commit(tx)
        .await
        .expect("Failed to commit transaction");

    // Verify both users were created
    let user1 = user_repo
        .get_by_id(&ctx, user_id1)
        .await
        .expect("Failed to get user1");
    assert!(user1.is_some());
    assert_eq!(user1.unwrap().username, "user1");

    let user2 = user_repo
        .get_by_id(&ctx, user_id2)
        .await
        .expect("Failed to get user2");
    assert!(user2.is_some());
    assert_eq!(user2.unwrap().username, "user2");
}

#[tokio::test]
async fn test_rollback_multiple_operations() {
    let pool = init_sqlite_pool().await;
    let tx_manager = SqliteTransactionManager::new(pool.clone());
    let user_repo = SqliteUserRepository::new(pool);
    let ctx = Context::new();

    let user_id1 = generate_test_id();
    let user_id2 = generate_test_id();

    // Begin transaction
    let mut tx = tx_manager
        .begin()
        .await
        .expect("Failed to begin transaction");

    // Insert first user
    sqlx::query(
        r#"
        INSERT INTO users (id, username, password, name, email, phone)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_id1)
    .bind("user1_rollback")
    .bind("hash1")
    .bind("User One Rollback")
    .bind(Some("user1rb@example.com"))
    .bind(Some("3333333333"))
    .execute(&mut *tx)
    .await
    .expect("Failed to insert user1");

    // Insert second user
    sqlx::query(
        r#"
        INSERT INTO users (id, username, password, name, email, phone)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_id2)
    .bind("user2_rollback")
    .bind("hash2")
    .bind("User Two Rollback")
    .bind(Some("user2rb@example.com"))
    .bind(Some("4444444444"))
    .execute(&mut *tx)
    .await
    .expect("Failed to insert user2");

    // Rollback transaction
    tx_manager
        .rollback(tx)
        .await
        .expect("Failed to rollback transaction");

    // Verify neither user was created
    let user1 = user_repo
        .get_by_id(&ctx, user_id1)
        .await
        .expect("Failed to get user1");
    assert!(user1.is_none());

    let user2 = user_repo
        .get_by_id(&ctx, user_id2)
        .await
        .expect("Failed to get user2");
    assert!(user2.is_none());
}

// =============================================================================
// Transaction Error Handling
// =============================================================================

#[tokio::test]
async fn test_transaction_with_error_requires_rollback() {
    let pool = init_sqlite_pool().await;
    let tx_manager = SqliteTransactionManager::new(pool.clone());
    let user_repo = SqliteUserRepository::new(pool);
    let ctx = Context::new();

    let user_id1 = generate_test_id();
    let user_id2 = generate_test_id();

    // Begin transaction
    let mut tx = tx_manager
        .begin()
        .await
        .expect("Failed to begin transaction");

    // Insert first user
    sqlx::query(
        r#"
        INSERT INTO users (id, username, password, name, email, phone)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_id1)
    .bind("user_before_error")
    .bind("hash1")
    .bind("User Before Error")
    .bind(Some("before@example.com"))
    .bind(Some("5555555555"))
    .execute(&mut *tx)
    .await
    .expect("Failed to insert user1");

    // Try to insert duplicate username (should fail)
    let result = sqlx::query(
        r#"
        INSERT INTO users (id, username, password, name, email, phone)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_id2)
    .bind("user_before_error") // Duplicate username
    .bind("hash2")
    .bind("User Duplicate")
    .bind(Some("duplicate@example.com"))
    .bind(Some("6666666666"))
    .execute(&mut *tx)
    .await;

    // Should fail due to unique constraint
    assert!(result.is_err());

    // Rollback transaction
    tx_manager
        .rollback(tx)
        .await
        .expect("Failed to rollback transaction");

    // Verify first user was also rolled back (atomicity)
    let user1 = user_repo
        .get_by_id(&ctx, user_id1)
        .await
        .expect("Failed to get user1");
    assert!(user1.is_none());
}

// =============================================================================
// Transaction Isolation Tests
// =============================================================================

#[tokio::test]
async fn test_transaction_commit_makes_data_visible() {
    let pool = init_sqlite_pool().await;
    let tx_manager = SqliteTransactionManager::new(pool.clone());
    let user_repo = SqliteUserRepository::new(pool);
    let ctx = Context::new();

    let user_id = generate_test_id();

    // Begin transaction
    let mut tx = tx_manager
        .begin()
        .await
        .expect("Failed to begin transaction");

    // Insert user within transaction
    sqlx::query(
        r#"
        INSERT INTO users (id, username, password, name, email, phone)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_id)
    .bind("isolated_user")
    .bind("hash123")
    .bind("Isolated User")
    .bind(Some("isolated@example.com"))
    .bind(Some("7777777777"))
    .execute(&mut *tx)
    .await
    .expect("Failed to insert user");

    // Commit transaction
    tx_manager
        .commit(tx)
        .await
        .expect("Failed to commit transaction");

    // After commit, user should be visible
    let user = user_repo
        .get_by_id(&ctx, user_id)
        .await
        .expect("Failed to get user");
    assert!(user.is_some());
    assert_eq!(user.unwrap().username, "isolated_user");
}

// =============================================================================
// Pool Access Tests
// =============================================================================

#[tokio::test]
async fn test_transaction_manager_pool_access() {
    let pool = init_sqlite_pool().await;
    let tx_manager = SqliteTransactionManager::new(pool.clone());

    // Verify we can access the pool
    let pool_ref = tx_manager.pool();
    assert_eq!(pool_ref.size() as u32, pool.size() as u32);
}
