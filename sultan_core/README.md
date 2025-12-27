# Sultan Core

Core library for the Sultan system, providing domain models, storage interfaces, and cryptographic utilities.

## Features

- **Domain Models**: Defines the core entities of the system (User, Branch, etc.).
- **Storage**: Provides repository interfaces and implementations.
    - **SQLite**: Includes a SQLite implementation of the repositories using `sqlx`.
- **Cryptography**: Utilities for password hashing and verification (using Argon2).

## Architecture

The project is organized into the following modules:

- `src/domain`: Contains the domain models and error types.
- `src/storage`: Defines the repository traits and their implementations.
- `src/crypto`: Provides cryptographic helper functions.

## Dependencies

Key dependencies include:

- `tokio`: Asynchronous runtime.
- `sqlx`: Async SQL toolkit (SQLite).
- `argon2`: Password hashing.
- `serde`: Serialization and deserialization.
- `chrono`: Date and time handling.
- `uuid`: UUID generation.
- `tracing`: Instrumentation.

## License

This project is licensed under the [AGPL-3.0 License](LICENSE).
