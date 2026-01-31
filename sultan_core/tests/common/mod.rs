#![allow(dead_code)]
use once_cell::sync::Lazy;
use sea_orm::{Database, DatabaseConnection};
use sqlx::SqlitePool;
use sultan_core::{
    domain::model::pagination::PaginationOptions, snowflake::SnowflakeGenerator, storage::RepoCtx,
};
use tokio::sync::Mutex;
use uuid::Uuid;

pub static ID_GENERATOR: Lazy<Mutex<SnowflakeGenerator>> =
    Lazy::new(|| Mutex::new(SnowflakeGenerator::new(1).unwrap()));

pub async fn generate_test_id() -> i64 {
    let generator = ID_GENERATOR.lock().await;
    generator.generate().unwrap()
}

pub fn default_pagination() -> PaginationOptions {
    PaginationOptions::new(1, 100, None)
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

pub async fn init_sqlite_repo_ctx() -> RepoCtx<DatabaseConnection> {
    // Create an isolated in-memory database for each test to avoid schema conflicts
    let temp_file = format!("/tmp/test_{}.db", Uuid::new_v4());
    let connection_string = format!("sqlite://{}?mode=rwc", temp_file);

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

    let db_connection = Database::connect(connection_string)
        .await
        .expect("unable to connect sqlite");

    RepoCtx {
        ctx: sultan_core::domain::Context::new(),
        db: db_connection,
    }
}

pub async fn init_sqlite_db() -> DatabaseConnection {
    // Create an isolated in-memory database for each test to avoid schema conflicts
    let temp_file = format!("/tmp/test_{}.db", Uuid::new_v4());
    let connection_string = format!("sqlite://{}?mode=rwc", temp_file);

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

    // Configure connection pool with multiple connections for transaction testing
    let mut opt = sea_orm::ConnectOptions::new(connection_string);
    opt.max_connections(5)
        .min_connections(1)
        .sqlx_logging(false);

    let db_connection = Database::connect(opt)
        .await
        .expect("unable to connect sqlite");

    db_connection
}

pub async fn setup_test_db() -> SqlitePool {
    init_sqlite_pool().await
}

/// Initialize a SQLite database and return both the pool (for SQLx-based repos)
/// and the RepoCtx (for SeaORM-based repos) from the same database.
pub async fn init_sqlite_repo_ctx_with_pool() -> (RepoCtx<DatabaseConnection>, SqlitePool) {
    // Create an isolated in-memory database for each test to avoid schema conflicts
    let temp_file = format!("/tmp/test_{}.db", Uuid::new_v4());
    let connection_string = format!("sqlite://{}?mode=rwc", temp_file);

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

    let db_connection = Database::connect(connection_string)
        .await
        .expect("unable to connect sqlite");

    let repo_ctx = RepoCtx {
        ctx: sultan_core::domain::Context::new(),
        db: db_connection,
    };

    (repo_ctx, new_pool)
}

pub async fn create_test_branch(pool: &SqlitePool, id: i64, code: &str) -> i64 {
    sqlx::query(
        r#"
        INSERT INTO branches (id, name, code, is_main)
        VALUES (?, ?, ?, 0)
        "#,
    )
    .bind(id)
    .bind(format!("Test Branch {}", code))
    .bind(code)
    .execute(pool)
    .await
    .expect("Failed to create test branch");

    id
}

/// Initialize a SQLite database and return both the SqlitePool (for SQLx transaction tests)
/// and a SeaORM DatabaseConnection pointing to the same physical database file.
/// This allows transaction tests to insert data via SQLx and verify via SeaORM repositories.
pub async fn init_sqlite_pool_with_seaorm() -> (SqlitePool, DatabaseConnection) {
    // Create an isolated file-based database shared between SQLx and SeaORM
    let temp_file = format!("/tmp/test_{}.db", Uuid::new_v4());
    let connection_string = format!("sqlite://{}?mode=rwc", temp_file);

    let new_pool = sqlx::sqlite::SqlitePoolOptions::new()
        .min_connections(1)
        .connect(&connection_string)
        .await
        .expect("Failed to create SQLite database");

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

    // Create SeaORM connection to the same database
    let db_connection = Database::connect(&connection_string)
        .await
        .expect("unable to connect sqlite");

    (new_pool, db_connection)
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
