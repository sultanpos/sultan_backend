use axum::{Json, Router, http, http::StatusCode, response::IntoResponse};
use http::header::{AUTHORIZATION, CONTENT_TYPE};
use sqlx::{Sqlite, SqlitePool, migrate::MigrateDatabase, sqlite::SqlitePoolOptions};
use std::{fs::File, sync::Arc};
use sultan_core::{
    application::AuthService,
    crypto::{Argon2PasswordHasher, DefaultJwtManager, JwtConfig},
    storage::{SqliteUserRepository, sqlite::SqliteTokenRepository},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

use crate::{
    config::AppConfig,
    web::{AppState, auth_router::auth_router},
};

async fn init_sqlite_db(config: &AppConfig) -> anyhow::Result<SqlitePool> {
    let database_url = &config.database_url;

    // Create database if it doesn't exist
    if !Sqlite::database_exists(database_url).await? {
        tracing::info!("Creating SQLite database at: {}", database_url);
        Sqlite::create_database(database_url).await?;
    }

    tracing::info!("Connecting to SQLite database");
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    tracing::info!("Running SQLite migrations");
    sqlx::migrate!("../sultan_core/migrations")
        .run(&pool)
        .await?;

    tracing::info!("Connected to SQLite database");
    Ok(pool)
}

async fn init_app_state(config: &AppConfig) -> anyhow::Result<AppState> {
    let pool = init_sqlite_db(config).await?;

    let user_repository = SqliteUserRepository::new(pool.clone());
    let token_repository = SqliteTokenRepository::new(pool);

    let password_hasher = Argon2PasswordHasher::default();
    let jwt_manager = DefaultJwtManager::new(JwtConfig::new(
        config.jwt_secret.clone(),
        config.access_token_ttl.whole_minutes(),
    ));
    let auth_service = AuthService::new(
        user_repository,
        token_repository,
        password_hasher,
        jwt_manager.clone(),
    );

    Ok(AppState {
        config: Arc::new(config.clone()),
        auth_service: Arc::new(auth_service)
            as Arc<
                dyn sultan_core::application::AuthServiceTrait<
                        sultan_core::domain::context::BranchContext,
                    >,
            >,
    })
}

async fn handle_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "no route found" })),
    )
}

fn init_tracing(write_log_to_file: bool) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "clean_architecture=debug,tower_http=debug".into());

    // Console (pretty logs)
    let console_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_level(true)
        .pretty();

    let registry = tracing_subscriber::registry()
        .with(filter)
        .with(console_layer);

    if write_log_to_file {
        // File (structured JSON logs)
        let file = File::create("app.log").expect("Cannot create log file");
        let json_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_writer(file)
            .with_current_span(true)
            .with_span_list(true);

        registry.with(json_layer).try_init().ok();
    } else {
        registry.try_init().ok();
    }
}

pub async fn create_app() -> anyhow::Result<Router> {
    let config = AppConfig::from_env();
    init_tracing(config.write_log_to_file);

    let app_state = init_app_state(&config).await?;

    let cors = CorsLayer::new()
        .allow_origin(
            "http://localhost:5173"
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .allow_methods([http::Method::POST, http::Method::GET])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION])
        .allow_credentials(true);

    let router = Router::new()
        .nest("/api/auth", auth_router())
        .fallback(handle_404)
        .with_state(app_state)
        .layer(cors)
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &http::Request<_>| {
                let request_id = Uuid::new_v4();
                tracing::info_span!(
                    "http-request",
                    method = %request.method(),
                    uri = %request.uri(),
                    version = ?request.version(),
                    request_id = %request_id
                )
            }),
        );

    Ok(router)
}
