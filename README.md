# Sultan Backend

[![Pull Request CI](https://github.com/sultanpos/sultan_backend/actions/workflows/pr.yml/badge.svg)](https://github.com/sultanpos/sultan_backend/actions/workflows/pr.yml)

A modern, production-ready Point of Sale (POS) backend system built with Rust, featuring clean architecture principles and comprehensive testing.

## 🏗️ Architecture

Sultan Backend is built using Clean Architecture principles with clear separation of concerns:

- **sultan_core**: Domain layer containing business logic, entities, and use cases
  - Domain models and context
  - Application services (auth, branch, category, customer, product, supplier, user)
  - Storage abstractions (repositories)
  - Cryptography utilities (JWT, password hashing)
  
- **sultan**: Web layer providing HTTP API
  - Axum web framework
  - REST API endpoints
  - Request/response DTOs
  - Middleware and error handling

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
  - SQLite database with SQLx
  - Compile-time query checking
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

Migrations run automatically on application startup. The system will:
- Create the database if it doesn't exist
- Apply all pending migrations from `sultan_core/migrations`

### 5. Build and run

```bash
# Development mode
cargo run

# Production build
cargo build --release
./target/release/sultan
```

The server will start on `http://0.0.0.0:8721`

## 🧪 Testing

### Run all tests

```bash
cargo test
```

### Run specific test suites

```bash
# Authentication tests
cargo test --package sultan --test auth_test

# Repository tests
cargo test --package sultan_core
```

### Run with coverage

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --html
```

### Linting

```bash
# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy -- -D warnings
```

## 📁 Project Structure

```
sultan_backend/
├── .github/
│   └── workflows/
│       └── pr.yml              # CI/CD pipeline
├── sultan/                     # Web layer
│   ├── src/
│   │   ├── domain/            # DTOs
│   │   ├── web/               # HTTP handlers & routing
│   │   ├── config.rs          # Configuration
│   │   ├── server.rs          # Application setup
│   │   ├── main.rs            # Entry point
│   │   └── lib.rs
│   └── tests/                 # Integration tests
│       ├── auth_test.rs
│       └── common/            # Test utilities
├── sultan_core/               # Domain layer (submodule)
│   ├── src/
│   │   ├── application/       # Business logic services
│   │   ├── domain/            # Domain models
│   │   ├── storage/           # Repository implementations
│   │   ├── crypto/            # JWT & password utilities
│   │   └── snowflake/         # ID generation
│   ├── migrations/            # Database migrations
│   └── tests/                 # Unit tests
├── Cargo.toml                 # Workspace configuration
└── README.md
```

## 🔌 API Endpoints

### Authentication

- `POST /api/auth` - Login with username and password
  ```json
  {
    "username": "admin",
    "password": "password123"
  }
  ```
  
  Response:
  ```json
  {
    "access_token": "eyJ...",
    "refresh_token": "eyJ..."
  }
  ```

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
- **Trait-based Design**: Easy to mock and test with dependency injection
- **Type Safety**: Leverage Rust's type system for compile-time guarantees
- **Async First**: Built for high concurrency with Tokio

### Testing Strategy

- **Unit Tests**: In `sultan_core/tests/` for business logic
- **Integration Tests**: In `sultan/tests/` for API endpoints
- **Mock Services**: Trait-based mocking for isolated testing
- **Coverage**: Tracked with cargo-llvm-cov

### Code Quality

- **Formatting**: `cargo fmt` with default settings
- **Linting**: `cargo clippy` with warnings as errors in CI
- **Type Checking**: Full compile-time verification with Rust's type system

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and linting (`cargo test && cargo clippy`)
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
- Database operations with [SQLx](https://github.com/launchbadge/sqlx)
- JWT handling with [jsonwebtoken](https://github.com/Keats/jsonwebtoken)
- Password hashing with [Argon2](https://github.com/RustCrypto/password-hashes)

## 📧 Contact

Sultan POS - [@sultanpos](https://github.com/sultanpos)

Project Link: [https://github.com/sultanpos/sultan_backend](https://github.com/sultanpos/sultan_backend)
