use axum::{
    Json, Router,
    http::{self, StatusCode},
    middleware::from_fn,
    response::IntoResponse,
};
use http::header::{AUTHORIZATION, CONTENT_TYPE};
use sea_orm::{Database, DatabaseConnection};
use sqlx::{Sqlite, migrate::MigrateDatabase, sqlite::SqlitePoolOptions};
use std::{fs::File, sync::Arc};
use sultan_core::{
    application::{
        AuthService, AuthServiceTrait, BranchService, CategoryService, CustomerService,
        InMemoryCache, NumberService, ProductService, SupplierService, UserService,
        UserServiceTrait,
    },
    crypto::{Argon2PasswordHasher, DefaultJwtManager, JwtConfig, JwtManager},
    domain::{
        Context,
        model::{
            branch::BranchCreate,
            permission::{PermissionCreate, resource},
            user::UserCreate,
        },
    },
    snowflake::SnowflakeGenerator,
    storage::{
        BranchRepository, RepoCtx, SqliteStockRepository, SqliteUserRepository,
        sqlite::{
            SqliteBranchRepository, SqliteCategoryRepository, SqliteCustomerRepository,
            SqliteNumberRepository, SqliteProductRepository, SqliteSupplierRepository,
            SqliteTokenRepository,
        },
    },
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

use crate::config::AppConfig;
use sultan_web::{
    AppState,
    supplier_routes::supplier_router,
    user_routes::{UserApiDoc, user_router},
};
use sultan_web::{
    handler::{
        auth_router::{AuthApiDoc, auth_router},
        branch_router::{BranchApiDoc, branch_router},
        category_router::{CategoryApiDoc, category_router},
        customer_router::{CustomerApiDoc, customer_router},
        middleware::{context_middleware, verify_jwt},
        product_router::{ProductApiDoc, product_router},
    },
    supplier_routes::SupplierApiDoc,
};

async fn init_sqlite_db(config: &AppConfig) -> anyhow::Result<DatabaseConnection> {
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
    sqlx::migrate!("../migrations").run(&pool).await?;

    pool.close().await;

    let mut opt = sea_orm::ConnectOptions::new(database_url);
    opt.max_connections(config.database_max_connections)
        .min_connections(1)
        .sqlx_logging(false);
    let database = Database::connect(opt).await?;
    tracing::info!("Connected to SQLite database");

    Ok(database)
}

async fn init_app_state(config: &AppConfig) -> anyhow::Result<AppState> {
    let db_connection = init_sqlite_db(config).await?;

    let branch_repository = SqliteBranchRepository::new();
    let user_repository = SqliteUserRepository::new();
    let token_repository = SqliteTokenRepository::new();
    let category_repository = SqliteCategoryRepository::new();
    let supplier_repository = SqliteSupplierRepository::new();
    let customer_repository = SqliteCustomerRepository::new();
    let number_repository = SqliteNumberRepository::new();
    let product_repository = SqliteProductRepository::new();
    let stock_repository = SqliteStockRepository::new();

    let password_hasher = Argon2PasswordHasher::default();
    let jwt_manager = DefaultJwtManager::new(JwtConfig::new(
        config.jwt_secret.clone(),
        config.access_token_ttl.whole_minutes(),
    ));
    let permission_cache = InMemoryCache::<i64>::new();
    let auth_service = AuthService::new(
        user_repository.clone(),
        token_repository,
        password_hasher,
        jwt_manager.clone(),
        db_connection.clone(),
    );

    let number_service = Arc::new(NumberService::new(
        number_repository,
        branch_repository.clone(),
        db_connection.clone(),
    ));
    let category_service = CategoryService::new(
        category_repository,
        SnowflakeGenerator::new(1)?,
        db_connection.clone(),
    );
    let customer_service = CustomerService::new(
        customer_repository,
        SnowflakeGenerator::new(1)?,
        number_service,
        db_connection.clone(),
    );
    let supplier_service = SupplierService::new(
        supplier_repository,
        SnowflakeGenerator::new(1)?,
        db_connection.clone(),
    );
    let branch_service = BranchService::new(
        branch_repository.clone(),
        SnowflakeGenerator::new(1)?,
        db_connection.clone(),
    );
    let user_service = UserService::new(
        user_repository.clone(),
        Arc::new(Argon2PasswordHasher::default()),
        SnowflakeGenerator::new(1)?,
        Arc::new(permission_cache),
        db_connection.clone(),
    );
    let product_service = ProductService::new(
        product_repository.clone(),
        stock_repository.clone(),
        SnowflakeGenerator::new(1)?,
        db_connection.clone(),
    );

    // init data when not available
    let ctx = Context::new_internal();
    let repo_ctx = RepoCtx {
        ctx: ctx.clone(),
        db: db_connection.clone(),
    };
    let branches = branch_repository.get_all(&repo_ctx).await?;
    if branches.is_empty() {
        let id_generator = SnowflakeGenerator::new(1)?;
        let id = id_generator.generate()?;
        let default_branch = BranchCreate {
            is_main: true,
            code: "SULTAN".to_string(),
            name: "Sultan".to_string(),
            address: None,
            phone: None,
            npwp: None,
            image: None,
        };
        branch_repository
            .create(&repo_ctx, id, &default_branch)
            .await?;
        tracing::info!("Created default branch");

        user_service
            .create(
                &ctx,
                &UserCreate {
                    username: "sultan".to_string(),
                    password: "sultan".to_string(),
                    name: "sultan".to_string(),
                    email: None,
                    photo: None,
                    pin: None,
                    address: None,
                    phone: None,
                },
                &[PermissionCreate {
                    branch_id: None,
                    resource: resource::ADMIN,
                    action: 0,
                }],
            )
            .await?;
    }

    Ok(AppState {
        auth_service: Arc::new(auth_service) as Arc<dyn AuthServiceTrait>,
        jwt_manager: Arc::new(jwt_manager) as Arc<dyn JwtManager>,
        branch_service: Arc::new(branch_service),
        category_service: Arc::new(category_service),
        customer_service: Arc::new(customer_service),
        supplier_service: Arc::new(supplier_service),
        user_service: Arc::new(user_service),
        product_service: Arc::new(product_service),
        extensions: Arc::new(std::collections::HashMap::new()),
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
        .allow_methods([
            http::Method::POST,
            http::Method::GET,
            http::Method::PUT,
            http::Method::DELETE,
            http::Method::PATCH,
        ])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION])
        .allow_credentials(true);

    let protected_router = Router::new()
        .nest("/branch", branch_router())
        .nest("/category", category_router())
        .nest("/customer", customer_router())
        .nest("/supplier", supplier_router())
        .nest("/product", product_router())
        .nest("/user", user_router())
        .route_layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            verify_jwt,
        ));

    // Merge OpenAPI specs
    let mut openapi = AuthApiDoc::openapi();
    openapi.merge(BranchApiDoc::openapi());
    openapi.merge(CategoryApiDoc::openapi());
    openapi.merge(CustomerApiDoc::openapi());
    openapi.merge(SupplierApiDoc::openapi());
    openapi.merge(UserApiDoc::openapi());
    openapi.merge(ProductApiDoc::openapi());
    // Add Bearer token security scheme
    if let Some(components) = openapi.components.as_mut() {
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }

    let router = Router::new()
        .nest("/api/auth", auth_router())
        .nest("/api/", protected_router)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi))
        .fallback(handle_404)
        .layer(from_fn(context_middleware))
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
