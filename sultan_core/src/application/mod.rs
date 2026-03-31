pub mod auth_service;
pub mod branch_service;
pub mod cache;
pub mod category_service;
pub mod customer_service;
pub mod machine_service;
pub mod number_service;
pub mod product_service;
pub mod supplier_service;
pub mod user_service;

pub use auth_service::{AuthService, AuthServiceTrait, AuthTokens};
pub use branch_service::{BranchService, BranchServiceTrait};
pub use cache::{CacheService, InMemoryCache};
pub use category_service::{CategoryService, CategoryServiceTrait};
pub use customer_service::{CustomerService, CustomerServiceTrait};
pub use machine_service::{MachineService, MachineServiceTrait};
pub use number_service::{NumberService, NumberServiceTrait};
pub use product_service::{ProductService, ProductServiceTrait};
pub use supplier_service::{SupplierService, SupplierServiceTrait};
pub use user_service::{UserService, UserServiceTrait};

use sea_orm::{DatabaseConnection, TransactionTrait};

use crate::domain::{Context, DomainResult};
use crate::storage::RepoCtx;

/// Shared helper trait for services that hold a `DatabaseConnection`.
///
/// Implement [`database`] and get [`repo_ctx`] / [`txn_repo_ctx`] for free.
#[allow(async_fn_in_trait)]
pub trait ServiceDbHelper {
    /// Returns a reference to the underlying database connection.
    fn database(&self) -> &DatabaseConnection;

    /// Creates a [`RepoCtx`] backed by a plain (non-transactional) connection.
    fn repo_ctx(&self, ctx: &Context) -> RepoCtx<DatabaseConnection> {
        RepoCtx {
            ctx: ctx.clone(),
            db: self.database().clone(),
        }
    }

    /// Creates a [`RepoCtx`] backed by a new database transaction.
    async fn txn_repo_ctx(
        &self,
        ctx: &Context,
    ) -> DomainResult<RepoCtx<sea_orm::DatabaseTransaction>> {
        Ok(RepoCtx {
            ctx: ctx.clone(),
            db: self.database().begin().await?,
        })
    }
}

#[cfg(test)]
mockall::mock! {
    pub IdGen {}
    impl crate::snowflake::IdGenerator for IdGen {
        fn generate(&self) -> Result<i64, crate::snowflake::SnowflakeError>;
    }
}

#[cfg(test)]
pub fn create_mock_id_gen(id: i64) -> MockIdGen {
    let mut mock_id_gen = MockIdGen::new();
    mock_id_gen.expect_generate().returning(move || Ok(id));
    mock_id_gen
}
