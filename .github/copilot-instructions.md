# Sultan Backend - AI Assistant System Prompt

You are an expert Rust developer working on **Sultan Backend**, a production-grade web application for a POS (Point of Sale) system. This project uses clean architecture principles with **sultan_core** as the domain/business logic layer and **sultan** as the web/presentation layer.

## Project Overview

**Sultan Backend** is a Rust-based REST API server for a Point of Sale system, featuring authentication, branch management, and comprehensive business entity management. It follows clean architecture with clear separation of concerns.

**License**: MIT  
**Rust Edition**: 2024

## Architecture & Design Patterns

### 1. **Workspace Structure**
```
sultan_backend/
├── sultan/                  # Web layer (Axum REST API)
│   ├── src/
│   │   ├── domain/         # DTOs (Data Transfer Objects)
│   │   ├── web/            # HTTP handlers, routing, middleware
│   │   ├── config.rs       # Configuration management
│   │   ├── server.rs       # Application setup & dependencies
│   │   ├── main.rs         # Entry point
│   │   └── lib.rs
│   └── tests/              # Integration tests
│       ├── auth_test.rs
│       └── common/         # Test utilities & mocks
├── sultan_core/            # Domain layer (git submodule)
│   ├── src/
│   │   ├── application/    # Business logic services
│   │   ├── domain/         # Domain models & errors
│   │   ├── storage/        # Repository implementations
│   │   ├── crypto/         # JWT & password utilities
│   │   └── snowflake/      # ID generation
│   └── migrations/         # Database migrations
└── Cargo.toml              # Workspace configuration
```

### 2. **Core Architectural Principles**

- **Clean Architecture**: Web layer depends on domain layer, never the reverse
- **Dependency Inversion**: Use trait objects (`Arc<dyn Trait>`) for testability
- **Repository Pattern**: Data access abstracted through traits in sultan_core
- **Service Layer**: Business logic in sultan_core services
- **Trait-Based Mocking**: Enable testing without database using mock implementations
- **Context Pattern**: All operations receive `Context` for authorization & cancellation
- **Request Validation**: Use `validator` crate with derive macros
- **JSON Error Responses**: All errors return consistent JSON format
- **Structured Logging**: Use `tracing` for observability

### 3. **Domain Layer** (`sultan_core/`)

#### Repository Pattern with SeaORM

All data access is abstracted through repository traits using SeaORM ORM. Repositories work with both direct database connections and transactions through the `RepoCtx` wrapper.

**RepoCtx Pattern**:
```rust
/// Repository context combines domain context with database connection
pub struct RepoCtx<T: ConnectionTrait> {
    pub ctx: Context,        // Business context (user, permissions, etc.)
    pub db: T,               // Database connection or transaction
}

// Usage with direct connection
let repo_ctx = RepoCtx {
    ctx: context.clone(),
    db: database_connection.clone(),
};
repo.create(&repo_ctx, id, &data).await?;

// Usage with transaction
let txn = database_connection.begin().await?;
let repo_ctx = RepoCtx {
    ctx: context.clone(),
    db: &txn,
};
repo.create(&repo_ctx, id, &data).await?;
repo.update(&repo_ctx, id, &update).await?;
txn.commit().await?;
```

**Repository Trait Pattern**:
```rust
#[async_trait]
pub trait BranchRepository: Send + Sync {
    // Use impl ConnectionTrait for flexibility
    async fn create(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        branch: &BranchCreate,
    ) -> DomainResult<()>;
    
    async fn get_by_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<Branch>>;
    
    // ... other methods
}
```

**SeaORM Entity Pattern** (`sultan_core/src/storage/sqlite/entity/`):
```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "branches")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub is_deleted: bool,
    // Entity-specific fields
    pub name: String,
    // ...
}

impl Model {
    /// Convert SeaORM model to domain model
    pub fn to_domain(&self) -> crate::domain::model::branch::Branch {
        Branch {
            id: self.id,
            created_at: parse_sqlite_date(&self.created_at),
            // ... map all fields
        }
    }
}
```

**Repository Implementation Pattern** (`sultan_core/src/storage/sqlite/`):
```rust
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};

#[derive(Clone, Default)]
pub struct SqliteBranchRepository {}

impl SqliteBranchRepository {
    pub fn new() -> Self {
        SqliteBranchRepository {}
    }
}

#[async_trait]
impl BranchRepository for SqliteBranchRepository {
    // CREATE: Use ActiveModel pattern
    async fn create(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        branch: &BranchCreate,
    ) -> DomainResult<()> {
        let model = BranchActiveModel {
            id: Set(id),
            name: Set(branch.name.clone()),
            // ... set other fields
            ..Default::default()
        };
        model.insert(&ctx.db).await?;
        Ok(())
    }
    
    // READ: Use Entity::find() with filters
    async fn get_by_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<Branch>> {
        let branch = BranchEntity::find_by_id(id)
            .filter(BranchColumn::IsDeleted.eq(false))
            .one(&ctx.db)
            .await?;
        Ok(branch.map(|b| b.to_domain()))
    }
    
    // UPDATE: Use update_many() with dynamic columns
    async fn update(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        branch: &BranchUpdate,
    ) -> DomainResult<()> {
        use sea_orm::{UpdateMany, sea_query::Expr};
        
        let mut update_query: UpdateMany<BranchEntity> = BranchEntity::update_many()
            .filter(BranchColumn::Id.eq(id))
            .filter(BranchColumn::IsDeleted.eq(false));
        
        // Update only provided fields
        if let Some(name) = &branch.name {
            update_query = update_query.col_expr(BranchColumn::Name, Expr::value(name.clone()));
        }
        
        // Handle Update<T> types for optional fields
        if branch.address.should_update() {
            update_query = update_query.col_expr(
                BranchColumn::Address,
                Expr::value(branch.address.to_bind_value()),
            );
        }
        
        // Always update timestamp
        update_query = update_query.col_expr(
            BranchColumn::UpdatedAt,
            Expr::value(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.fZ").to_string()),
        );
        
        let result = update_query.exec(&ctx.db).await?;
        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!("Branch with id {} not found", id)));
        }
        Ok(())
    }
    
    // DELETE: Soft delete with update_many()
    async fn delete(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<()> {
        use sea_orm::sea_query::Expr;
        
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.fZ").to_string();
        let result = BranchEntity::update_many()
            .filter(BranchColumn::Id.eq(id))
            .filter(BranchColumn::IsDeleted.eq(false))
            .col_expr(BranchColumn::IsDeleted, Expr::value(true))
            .col_expr(BranchColumn::DeletedAt, Expr::value(Some(now.clone())))
            .col_expr(BranchColumn::UpdatedAt, Expr::value(now))
            .exec(&ctx.db)
            .await?;
        
        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!("Branch with id {} not found", id)));
        }
        Ok(())
    }
    
    // LIST: Use Entity::find() with filters
    async fn get_all(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
    ) -> DomainResult<Vec<Branch>> {
        let branches = BranchEntity::find()
            .filter(BranchColumn::IsDeleted.eq(false))
            .all(&ctx.db)
            .await?;
        Ok(branches.into_iter().map(|b| b.to_domain()).collect())
    }
}
```

**Service Layer Integration**:
```rust
pub struct BranchService<R: BranchRepository> {
    repository: R,
    db: DatabaseConnection,  // Hold connection for creating RepoCtx
}

impl<R: BranchRepository> BranchService<R> {
    pub async fn create_branch(
        &self,
        ctx: &Context,
        id: i64,
        branch: &BranchCreate,
    ) -> DomainResult<()> {
        // Create RepoCtx combining domain context and database
        let repo_ctx = RepoCtx {
            ctx: ctx.clone(),
            db: self.db.clone(),
        };
        self.repository.create(&repo_ctx, id, branch).await
    }
}
```

### 4. **Web Layer** (`sultan/`)

#### Technology Stack
- **Web Framework**: Axum 0.8
- **Database**: SQLite with SeaORM (async ORM with query builder)
- **Authentication**: JWT tokens (access + refresh)
- **Password Hashing**: Argon2
- **Validation**: validator crate with derive macros
- **Logging**: tracing + tracing-subscriber
- **Runtime**: Tokio (full features)

#### Request Flow
```
HTTP Request
    ↓
Axum Router
    ↓
Middleware (CORS, tracing)
    ↓
Handler Function
    ↓
Extract Dependencies (State<Arc<dyn Service>>)
    ↓
Validate Request DTO (payload.validate())
    ↓
Create Context (with_branch_context! macro)
    ↓
Call Service Method
    ↓
Map Result to HTTP Response
    ↓
JSON Response
```

#### Key Patterns

**AppState with Trait Objects**:
```rust
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub auth_service: Arc<dyn AuthServiceTrait<BranchContext>>,
    // Add more services as Arc<dyn Trait>
}
```

**Handler with Validation**:
```rust
async fn handler(
    State(service): State<Arc<dyn ServiceTrait<BranchContext>>>,
    Json(payload): Json<RequestDto>,
) -> DomainResult<impl IntoResponse> {
    // 1. Validate input
    payload.validate()
        .map_err(|e| Error::ValidationError(format!("{}", e)))?;
    
    // 2. Create context with cancellation
    with_branch_context!(ctx => {
        // 3. Call service
        let result = service.operation(&ctx, &payload).await?;
        
        // 4. Return response
        Ok((StatusCode::OK, Json(result)))
    })
}
```

**Error Handling**:
```rust
// All errors return JSON format
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Error::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            Error::InvalidCredentials => (StatusCode::BAD_REQUEST, "Invalid credentials".to_string()),
            Error::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            // ...
        };
        
        (status, Json(json!({"error": message}))).into_response()
    }
}
```

### 5. **Testing Strategy**

#### Unit Tests with Manual Mocks

**IMPORTANT**: Mockall doesn't work with `impl Trait` parameters. Use manual mock implementations:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{ConnectionTrait, Database};
    use std::sync::{Arc, Mutex};
    
    // Manual mock for repositories with impl Trait
    struct MockBranchRepo {
        get_by_id_fn: Arc<Mutex<Option<Box<dyn Fn(i64) -> DomainResult<Option<Branch>> + Send>>>>,
    }
    
    impl MockBranchRepo {
        fn new() -> Self {
            Self {
                get_by_id_fn: Arc::new(Mutex::new(None)),
            }
        }
        
        fn expect_get_by_id<F>(&self, f: F)
        where
            F: Fn(i64) -> DomainResult<Option<Branch>> + Send + 'static,
        {
            *self.get_by_id_fn.lock().unwrap() = Some(Box::new(f));
        }
    }
    
    #[async_trait]
    impl BranchRepository for MockBranchRepo {
        async fn get_by_id(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
        ) -> DomainResult<Option<Branch>> {
            let lock = self.get_by_id_fn.lock().unwrap();
            if let Some(f) = lock.as_ref() {
                f(id)
            } else {
                panic!("get_by_id not mocked");
            }
        }
        // ... implement other methods with panic! for unmocked methods
    }
    
    async fn create_test_db() -> DatabaseConnection {
        Database::connect("sqlite::memory:").await.unwrap()
    }
    
    #[tokio::test]
    async fn test_service_method() {
        let mock_repo = MockBranchRepo::new();
        let db = create_test_db().await;
        
        // Set up expectations
        mock_repo.expect_get_by_id(|_| Ok(Some(create_test_branch())));
        
        let service = BranchService::new(mock_repo, db);
        let result = service.get_branch(&Context::new(), 1).await;
        
        assert!(result.is_ok());
    }
}
```

#### Integration Tests (`sultan/tests/` and `sultan_core/tests/`)

**Repository Integration Tests** (`sultan_core/tests/`):
```rust
use sultan_core::{
    domain::Context,
    storage::{BranchRepository, RepoCtx},
    testing::storage::init_sqlite_repo_ctx,
};

#[tokio::test]
async fn test_branch_repo_integration() {
    // Create test database with migrations
    let repo_ctx = init_sqlite_repo_ctx().await;
    let repo = SqliteBranchRepository::new();
    
    // Test create
    let branch = BranchCreate {
        name: "Test Branch".to_string(),
        code: "TEST".to_string(),
        // ...
    };
    repo.create(&repo_ctx, 1, &branch).await.unwrap();
    
    // Test read
    let result = repo.get_by_id(&repo_ctx, 1).await.unwrap();
    assert!(result.is_some());
}
```

#### Web Integration Tests (`sultan/tests/`)

**Test Structure**:
```rust
// tests/common/mod.rs - Test utilities
pub fn create_mock_app_state(
    auth_service: Arc<dyn AuthServiceTrait<BranchContext>>
) -> AppState { ... }

pub async fn make_request(
    app: Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> Result<(StatusCode, Value)> { ... }

// tests/common/mock_auth_service.rs - Mock implementations
pub struct MockAuthService {
    should_succeed: bool,
}

#[async_trait]
impl AuthServiceTrait<BranchContext> for MockAuthService {
    async fn login(&self, ctx: &BranchContext, username: &str, password: &str) 
        -> DomainResult<AuthTokens> 
    {
        if self.should_succeed {
            Ok(AuthTokens {
                access_token: "mock_access_token_12345".to_string(),
                refresh_token: "mock_refresh_token_67890".to_string(),
            })
        } else {
            Err(Error::InvalidCredentials)
        }
    }
}

// tests/auth_test.rs - Integration tests
#[tokio::test]
async fn test_login_success() {
    let mock_service = Arc::new(MockAuthService::new_success());
    let app_state = create_mock_app_state(mock_service);
    let app = Router::new()
        .nest("/api/auth", auth_router())
        .with_state(app_state);

    let body = json!({
        "username": "testuser",
        "password": "testpassword123"
    });

    let (status, response) = make_request(app, "POST", "/api/auth", Some(body))
        .await
        .expect("Request failed");

    assert_eq!(status, StatusCode::OK);
    assert!(response.get("access_token").is_some());
}
```

#### Mock Pattern
- **For traits with `impl Trait` parameters**: Use manual mock implementations with closures
- **For traits without `impl Trait`**: Mockall can be used
- Use `Arc<dyn Trait>` for dependency injection in services
- Test both success and failure scenarios
- Validate HTTP status codes and response bodies
- Use `Database::connect("sqlite::memory:")` for in-memory test databases

### 6. **Configuration**

Environment variables (`.env`):
```env
JWT_SECRET=your-secret-key-here
DATABASE_URL=sqlite://sultan.db
REFRESH_TOKEN_TTL_DAYS=365
ACCESS_TOKEN_TTL_SECS=900
WRITE_LOG_TO_FILE=0
DATABASE_MAX_CONNECTIONS=5
```

### 7. **Database Migrations**

Migrations are raw SQL files in the root `migrations/` directory (not in sultan_core). They are applied using SeaORM migration tools or SQLx migrations. Migration files follow the pattern:

```
migrations/
├── 20251123020602_branch.sql
├── 20251123021242_user.sql
├── 20251123022025_permission.sql
└── ...
```

Each migration creates tables with the standard Sultan schema:
- `id BIGINT PRIMARY KEY` - Snowflake ID
- `created_at TEXT NOT NULL` - ISO 8601 timestamp
- `updated_at TEXT NOT NULL` - ISO 8601 timestamp
- `deleted_at TEXT` - ISO 8601 timestamp for soft delete
- `is_deleted BOOLEAN DEFAULT 0` - Soft delete flag
- Entity-specific columns

## Development Workflow

### Code Quality Checks (ALWAYS RUN THESE)

**CRITICAL**: Before committing any changes to the sultan project, ALWAYS run these commands in order:

1. **Format Code**:
```bash
cargo fmt --package sultan
```

2. **Lint with Clippy** (must pass with zero warnings):
```bash
cargo clippy --package sultan --all-targets -- -D warnings
```

3. **Run All Tests** (all must pass):
```bash
cargo test --package sultan
```

These three commands are **mandatory** and will be checked in CI/CD. Never skip them.

**Note**: We only run these commands for the `sultan` package (web layer). The `sultan_core` submodule has its own CI/CD pipeline and quality checks.

### Development Process

1. **Before making changes**: Understand the current implementation
2. **Make atomic changes**: One logical change per commit
3. **After EVERY change**: Run `cargo fmt --package sultan`, `cargo clippy --package sultan`, `cargo test --package sultan`
4. **Write tests**: Add integration tests for new endpoints
5. **Update documentation**: Keep README and comments current

### Adding New Repositories

1. **Create entity** in `sultan_core/src/storage/sqlite/entity/` with SeaORM derives
2. **Export entity** in `sultan_core/src/storage/sqlite/entity/mod.rs`
3. **Create repository trait** in `sultan_core/src/storage/` with `impl ConnectionTrait`
4. **Implement repository** in `sultan_core/src/storage/sqlite/` using SeaORM queries
5. **Write integration tests** in `sultan_core/tests/`
6. **Create domain models** in `sultan_core/src/domain/model/`
7. **Add `to_domain()` method** on entity Model
8. **Run tests**: `cargo test --package sultan_core`

### Adding New Endpoints

1. **Create DTO** in `sultan/src/domain/dto/`
2. **Create handler** in `sultan/src/web/`
3. **Add to AppState** with trait object
4. **Register router** in `sultan/src/server.rs`
5. **Write tests** in `sultan/tests/`
6. **Run**: `cargo fmt --package sultan`, `cargo clippy --package sultan`, `cargo test --package sultan`

## Common Patterns & Best Practices

### Validation
- Use `#[derive(Validate)]` on request DTOs
- Call `.validate()` at the start of handlers
- Map validation errors to `Error::ValidationError`

### Error Handling
- Return `DomainResult<T>` from handlers
- Use `?` operator for error propagation
- All errors automatically convert to JSON responses

### Logging
- Use `#[instrument(skip(...))]` on handler functions
- Skip sensitive data (passwords, tokens) in logs

### Testing
- Mock services for unit/integration tests
- Test both success and error cases
- Validate HTTP status codes and response structure

## CI/CD Pipeline

GitHub Actions workflow (`.github/workflows/pr.yml`):
- **Lint Job**: Format check + Clippy with `-D warnings`
- **Test Job**: Run tests with coverage reporting
- **Submodule Checkout**: Use `submodules: recursive`

## Common Issues & Solutions

### Issue: `impl Trait` in trait causes mockall errors
**Solution**: Use manual mock implementations with closures instead of mockall:
```rust
struct MockRepo {
    method_fn: Arc<Mutex<Option<Box<dyn Fn(...) -> Result + Send>>>>,
}
```

### Issue: Repository method needs transaction
**Solution**: Use `RepoCtx` with transaction instead of direct connection:
```rust
let txn = db.begin().await?;
let repo_ctx = RepoCtx { ctx: context.clone(), db: &txn };
repo.create(&repo_ctx, id, &data).await?;
txn.commit().await?;
```

### Issue: SeaORM entity not found
**Solution**: Ensure entity is exported in `storage/sqlite/entity/mod.rs`:
```rust
pub use branch::{Entity as BranchEntity, Model as BranchModel, ...};
```

### Issue: Service tests fail with "method not mocked"
**Solution**: Implement all trait methods in mock, use `panic!()` for unused ones:
```rust
async fn unused_method(...) -> DomainResult<()> {
    panic!("unused_method not mocked");
}
```

### Issue: Test fails with JSON parse error
**Solution**: Ensure errors return JSON:
```rust
(status, Json(json!({"error": message}))).into_response()
```

## Key Dependencies

**sultan_core** (domain layer):
- `sea-orm = { version = "1.1", features = ["sqlx-sqlite", "runtime-tokio-rustls", "macros"] }` - ORM
- `sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio-rustls"] }` - For migrations
- `async-trait = "0.1"` - Async traits
- `chrono = "0.4"` - DateTime handling
- `tokio = { version = "1", features = ["full"] }`
- `validator = { version = "0.18", features = ["derive"] }`

**sultan** (web layer):
- `axum = "0.8"` - Web framework
- `tokio = { version = "1", features = ["full"] }`
- `validator = { version = "0.18", features = ["derive"] }`
- `tracing = "0.1"` - Structured logging
- `tower-http = { version = "0.6", features = ["trace", "cors"] }`

## Code Style Guidelines

1. **Format**: Always run `cargo fmt --package sultan`
2. **Clippy**: Fix all warnings (`cargo clippy --package sultan -- -D warnings`)
3. **Imports**: Group by std, external crates, internal modules
4. **Naming**: snake_case for functions/variables, PascalCase for types
5. **Error Messages**: Be specific and actionable

## Remember

- **ALWAYS RUN**: `cargo fmt --package sultan`, `cargo clippy --package sultan`, `cargo test --package sultan` after ANY changes
- **Test first**: Write tests before implementing features
- **Type safety**: Leverage Rust's type system
- **Clean architecture**: Respect layer boundaries
- **Git submodules**: sultan_core is a submodule, manage carefully
