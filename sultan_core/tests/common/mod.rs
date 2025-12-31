#![allow(dead_code)]
pub mod branch_share;
pub mod category_share;

use once_cell::sync::Lazy;
use sqlx::SqlitePool;
use sultan_core::snowflake::SnowflakeGenerator;
use tokio::sync::Mutex;

pub static ID_GENERATOR: Lazy<Mutex<SnowflakeGenerator>> =
    Lazy::new(|| Mutex::new(SnowflakeGenerator::new(1).unwrap()));

pub async fn generate_test_id() -> i64 {
    let generator = ID_GENERATOR.lock().await;
    generator.generate().unwrap()
}

pub async fn init_sqlite_pool() -> SqlitePool {
    // Create an isolated in-memory database for each test to avoid schema conflicts
    let connection_string = "sqlite::memory:".to_string();

    let new_pool = sqlx::sqlite::SqlitePoolOptions::new()
        .min_connections(1)
        .connect(&connection_string)
        .await
        .expect("Failed to create in-memory SQLite database");

    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let migrations = std::path::Path::new(&crate_dir).join("../migrations");
    print!(
        "migration folder {}",
        migrations.as_path().to_string_lossy()
    );

    sqlx::migrate::Migrator::new(migrations)
        .await
        .expect("Failed to load migrations")
        .run(&new_pool)
        .await
        .expect("Failed to run SQLite migrations");

    new_pool
}

/*
pub async fn init_postgres_pool() -> PgPool {
    let mut pool = POSTGRES_POOL.lock().await;

    if let Some(existing_pool) = pool.as_ref() {
        return existing_pool.clone();
    }

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL environment variable must be set for PostgreSQL tests");

    let new_pool = sqlx::PgPool::connect(&database_url)
        .await
        .unwrap_or_else(|e| {
            panic!(
                "Failed to connect to PostgreSQL database.\n\
                 DATABASE_URL: {}\n\
                 Error: {}\n\
                 Make sure PostgreSQL is running and the database exists.",
                database_url, e
            )
        });

    sqlx::migrate!("./migrations-postgres")
        .run(&new_pool)
        .await
        .expect("Failed to run PostgreSQL migrations");

    *pool = Some(new_pool.clone());
    new_pool
}
*/
