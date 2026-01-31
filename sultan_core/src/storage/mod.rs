pub mod branch_repo;
pub mod category_repo;
pub mod customer_repo;
pub mod number_repo;
pub mod product_repo;
pub mod sell_price_repo;
pub mod sqlite;
pub mod supplier_repo;
pub mod token_repo;
pub mod transaction;
pub mod unit_repo;
pub mod user_repo;

pub use branch_repo::BranchRepository;
pub use category_repo::CategoryRepository;
pub use customer_repo::CustomerRepository;
pub use number_repo::NumberRepository;
pub use product_repo::ProductRepository;
use sea_orm::ConnectionTrait;
pub use sqlite::SqliteUserRepository;
pub use supplier_repo::SupplierRepository;
pub use token_repo::TokenRepository;
pub use unit_repo::UnitOfMeasureRepository;
pub use user_repo::UserRepository;

use crate::domain::Context;

/// Repository context that wraps both the domain context and database connection.
///
/// The `RepoCtx` provides a unified way to pass both business context (authorization, cancellation)
/// and database access (connection or transaction) to repository methods.
///
/// # Generic Parameter
///
/// * `T: ConnectionTrait` - Any SeaORM connection type (DatabaseConnection, DatabaseTransaction, etc.)
///
/// # Usage with Direct Connection
///
/// ```rust,ignore
/// let ctx = RepoCtx {
///     ctx: Context::new(),
///     db: &database_connection,
/// };
/// repo.create(&ctx, id, &data).await?;
/// ```
///
/// # Usage with Transaction
///
/// ```rust,ignore
/// let txn = database_connection.begin().await?;
/// let ctx = RepoCtx {
///     ctx: Context::new(),
///     db: &txn,
/// };
/// repo.create(&ctx, id, &data).await?;
/// repo.update(&ctx, id, &update_data).await?;
/// txn.commit().await?;
/// ```
pub struct RepoCtx<T: ConnectionTrait> {
    pub ctx: Context,
    pub db: T,
}
