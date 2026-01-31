# Sultan Backend

[![Pull Request CI](https://github.com/sultanpos/sultan_backend/actions/workflows/pr.yml/badge.svg)](https://github.com/sultanpos/sultan_backend/actions/workflows/pr.yml)

A modern, production-ready Point of Sale (POS) backend system built with Rust, featuring clean architecture principles and comprehensive testing.

## 🏗️ Architecture

Sultan Backend is built using Clean Architecture principles with clear separation of concerns:

- **sultan_core**: Domain layer (git submodule)
  - Domain models, entities, and context
  - Application services (auth, branch, category, customer, product, supplier, user)
  - Storage abstractions (repositories with SeaORM)
  - Cryptography utilities (JWT, password hashing)
  - Database migrations
  
- **sultan_web**: Presentation layer
  - HTTP handlers and routing
  - Request/response DTOs
  - Middleware (JWT verification, context)
  - Mock services for testing
  
- **sultan**: Main application
  - Configuration management
  - Application bootstrapping
  - Server setup and initialization

## 🚀 Features

- **Authentication System**
  - JWT-based authentication with refresh tokens
  - Secure password hashing with Argon2
  - Token management and rotation
  
- **Business Entities**
  - Branch management
  - User management with permissions
  - Category management
  - Supplier management
  - Customer management
  - Product management

- **Technical Features**
  - Async/await with Tokio runtime
  - SQLite database with SeaORM ORM
  - Type-safe query builder
  - Database migrations
  - Request validation
  - Comprehensive error handling
  - Structured logging with tracing
  - CORS support
  - Cancellation token support for graceful shutdown

## 📋 Requirements

- Rust 1.75 or higher
- SQLite 3.x

## 🛠️ Setup

### 1. Clone the repository

```bash
git clone https://github.com/sultanpos/sultan_backend.git
cd sultan_backend
```

### 2. Initialize submodules

```bash
git submodule update --init --recursive
```

### 3. Configure environment

Create a `.env` file in the project root:

```env
JWT_SECRET=your-secret-key-here
DATABASE_URL=sqlite://sultan.db
REFRESH_TOKEN_TTL_DAYS=365
ACCESS_TOKEN_TTL_SECS=900
WRITE_LOG_TO_FILE=0
```

### 4. Run migrations

Migrations are stored in the root `migrations/` directory and are applied using SeaORM migration tools. The system will:
- Create the database if it doesn't exist
- Apply all pending migrations
- Each table follows the standard Sultan schema with soft delete support

### 5. Build and run

```bash
# Development mode
cargo run

# Production build
cargo build --release
./target/release/sultan
```

The server will start on `http://0.0.0.0:8721`

### 6. Access API Documentation

Once the server is running, you can access the interactive Swagger UI at:

**http://localhost:8721/swagger-ui/**

The Swagger UI provides:
- Interactive API documentation
- Ability to test endpoints directly from the browser
- Request/response examples
- Schema definitions

## 🧪 Testing

Sultan Backend has comprehensive test coverage across all layers:

### Test Suites

**Web Layer Tests** (`sultan_web/tests/`):
- `auth_test.rs` - Authentication endpoint tests (10 tests)
- `category_test.rs` - Category CRUD operations (25 tests)
- `customer_test.rs` - Customer management (22 tests)
- `middleware_test.rs` - JWT verification & context middleware (8 tests)

**Configuration Tests** (`sultan/tests/`):
- `config_test.rs` - Environment configuration validation (12 tests)

**Domain Layer Tests** (`sultan_core/tests/`):
- Repository tests for all entities (branch, category, customer, product, supplier, user)
- Transaction handling tests
- Business logic validation

**Total**: 77+ integration and unit tests

### Running Tests

```bash
# Run all tests
cargo test

# Run specific package tests
cargo test --package sultan
cargo test --package sultan_web
cargo test --package sultan_core

# Run specific test file
cargo test --test auth_test
cargo test --test config_test
cargo test --test middleware_test

# Run with coverage report
cargo install cargo-llvm-cov
cargo llvm-cov --html --open
```

### Test Features

- **Mock Services**: Trait-based mocking for isolated testing
- **Serial Tests**: Config tests use `serial_test` to avoid environment variable conflicts
- **Integration Tests**: Full HTTP request/response testing with Axum test utilities
- **Coverage Tracking**: SonarCloud integration for quality gates

### Linting & Quality Checks

```bash
# Format code
cargo fmt --package sultan
cargo fmt --package sultan_web

# Run clippy (zero warnings enforced)
cargo clippy --package sultan --all-targets -- -D warnings
cargo clippy --package sultan_web --all-targets -- -D warnings

# Check formatting
cargo fmt --all -- --check
```

## 📁 Project Structure

```
sultan_backend/
├── .github/
│   └── workflows/
│       └── pr.yml              # CI/CD pipeline
├── migrations/                 # Database migrations (raw SQL)
│   ├── 20251123020602_branch.sql
│   ├── 20251123021242_user.sql
│   └── ...
├── sultan/                     # Web layer
│   ├── src/
│   │   ├── config.rs          # Configuration management
│   │   ├── server.rs          # Application setup
│   │   ├── main.rs            # Entry point
│   │   └── lib.rs
│   └── tests/                 # Integration tests
│       └── config_test.rs     # Configuration tests (12 tests)
├── sultan_web/                 # Web handlers layer
│   ├── src/
│   │   ├── dto/               # Data Transfer Objects
│   │   ├── handler/           # HTTP handlers & routing
│   │   │   ├── auth_router.rs
│   │   │   ├── category_router.rs
│   │   │   ├── customer_router.rs
│   │   │   └── middleware.rs  # JWT & context middleware
│   │   ├── app_state.rs       # Application state
│   │   └── lib.rs
│   └── tests/                 # Integration tests
│       ├── auth_test.rs       # Auth endpoint tests (10 tests)
│       ├── category_test.rs   # Category tests (25 tests)
│       ├── customer_test.rs   # Customer tests (22 tests)
│       ├── middleware_test.rs # Middleware tests (8 tests)
│       └── common/            # Test utilities & mocks
├── sultan_core/               # Domain layer (submodule)
│   ├── src/
│   │   ├── application/       # Business logic services
│   │   ├── domain/            # Domain models
│   │   ├── storage/           # Repository traits & implementations
│   │   │   ├── sqlite/        # SeaORM repository implementations
│   │   │   │   ├── entity/    # SeaORM entities
│   │   │   │   ├── branch.rs
│   │   │   │   ├── category.rs
│   │   │   │   └── ...
│   │   │   ├── branch_repo.rs # Repository trait
│   │   │   └── ...
│   │   ├── crypto/            # JWT & password utilities
│   │   └── snowflake/         # ID generation
│   └── tests/                 # Unit & repository tests
├── Cargo.toml                 # Workspace configuration
└── README.md
```

## 🔌 API Endpoints

All API endpoints are documented using OpenAPI 3.0 specification and available through Swagger UI at `/swagger-ui/`.

### Authentication

**Base URL**: `/api/auth`

- `POST /api/auth` - Login with username and password
- `POST /api/auth/refresh` - Refresh access token using refresh token
- `DELETE /api/auth` - Logout (invalidate refresh token)

For detailed request/response schemas and to test the endpoints interactively, visit the Swagger UI documentation.

## 🔧 Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `JWT_SECRET` | Secret key for JWT signing | Required |
| `DATABASE_URL` | SQLite database path | Required |
| `REFRESH_TOKEN_TTL_DAYS` | Refresh token expiry in days | 30 |
| `ACCESS_TOKEN_TTL_SECS` | Access token expiry in seconds | 900 (15 min) |
| `WRITE_LOG_TO_FILE` | Enable file logging (0/1) | 0 |
| `DATABASE_MAX_CONNECTIONS` | Max database connections | 5 |

## 🏗️ Development

### Architecture Principles

- **Clean Architecture**: Clear separation between domain, application, and infrastructure layers
- **Dependency Inversion**: Core domain doesn't depend on external frameworks
- **Repository Pattern**: Data access abstracted through traits with SeaORM
- **RepoCtx Pattern**: Combines domain context with database connection/transaction
- **Trait-based Design**: Easy to mock and test with dependency injection
- **Type Safety**: Leverage Rust's type system for compile-time guarantees
- **Async First**: Built for high concurrency with Tokio

### Testing Strategy

- **Unit Tests**: In `sultan_core/tests/` for business logic and repositories
- **Integration Tests**: In `sultan_web/tests/` for API endpoints and middleware
- **Configuration Tests**: In `sultan/tests/` for environment handling
- **Manual Mock Pattern**: For repositories with `impl Trait` parameters (mockall doesn't support this)
- **In-Memory Database**: Use `Database::connect("sqlite::memory:")` for isolated repository tests
- **RepoCtx Testing**: Tests use `RepoCtx` with both direct connections and transactions
- **Serial Tests**: Environment-dependent tests use `serial_test` crate to prevent race conditions
- **Coverage**: Tracked with cargo-llvm-cov and SonarCloud for quality gates
- **CI/CD**: Automated testing on every pull request with GitHub Actions

### Code Quality

- **Formatting**: `cargo fmt` with default settings
- **Linting**: `cargo clippy` with warnings as errors in CI
- **Type Checking**: Full compile-time verification with Rust's type system

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and linting:
   ```bash
   # Format code
   cargo fmt --package sultan
   cargo fmt --package sultan_web
   
   # Run linting
   cargo clippy --package sultan --all-targets -- -D warnings
   cargo clippy --package sultan_web --all-targets -- -D warnings
   
   # Run all tests
   cargo test
   ```
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

### Commit Convention

Follow conventional commits:
- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation changes
- `test:` - Test additions or changes
- `refactor:` - Code refactoring
- `chore:` - Maintenance tasks

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Axum](https://github.com/tokio-rs/axum) web framework
- Database operations with [SeaORM](https://github.com/SeaQL/sea-orm) ORM
- JWT handling with [jsonwebtoken](https://github.com/Keats/jsonwebtoken)
- Password hashing with [Argon2](https://github.com/RustCrypto/password-hashes)

## 📧 Contact

Sultan POS - [@sultanpos](https://github.com/sultanpos)

Project Link: [https://github.com/sultanpos/sultan_backend](https://github.com/sultanpos/sultan_backend)
