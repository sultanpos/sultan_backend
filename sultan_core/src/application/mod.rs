pub mod auth_service;
pub mod branch_service;
pub mod category_service;
pub mod customer_service;
pub mod product_service;
pub mod supplier_service;
pub mod user_service;

pub use auth_service::{AuthService, AuthServiceTrait, AuthTokens};
pub use branch_service::{BranchService, BranchServiceTrait};
pub use category_service::{CategoryService, CategoryServiceTrait};
pub use customer_service::{CustomerService, CustomerServiceTrait};
pub use product_service::{ProductService, ProductServiceTrait};
pub use supplier_service::{SupplierService, SupplierServiceTrait};
pub use user_service::{UserService, UserServiceTrait};

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
