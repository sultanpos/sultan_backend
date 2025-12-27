use crate::{
    domain::{
        Context, DomainResult,
        model::{
            customer::{Customer, CustomerCreate, CustomerFilter, CustomerUpdate},
            pagination::PaginationOptions,
            permission::{action, resource},
        },
    },
    snowflake::IdGenerator,
    storage::CustomerRepository,
};
use async_trait::async_trait;

#[async_trait]
pub trait CustomerServiceTrait: Send + Sync {
    async fn create(&self, ctx: &Context, customer: &CustomerCreate) -> DomainResult<i64>;
    async fn update(&self, ctx: &Context, id: i64, customer: &CustomerUpdate) -> DomainResult<()>;
    async fn delete(&self, ctx: &Context, id: i64) -> DomainResult<()>;
    async fn get_by_number(&self, ctx: &Context, number: &str) -> DomainResult<Option<Customer>>;
    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<Customer>>;
    async fn get_all(
        &self,
        ctx: &Context,
        filter: &CustomerFilter,
        pagination: &PaginationOptions,
    ) -> DomainResult<Vec<Customer>>;
}

pub struct CustomerService<R, I> {
    repository: R,
    id_generator: I,
}

impl<R, I> CustomerService<R, I>
where
    R: CustomerRepository,
    I: IdGenerator,
{
    pub fn new(repository: R, id_generator: I) -> Self {
        Self {
            repository,
            id_generator,
        }
    }
}

#[async_trait]
impl<R, I> CustomerServiceTrait for CustomerService<R, I>
where
    R: CustomerRepository,
    I: IdGenerator,
{
    async fn create(&self, ctx: &Context, customer: &CustomerCreate) -> DomainResult<i64> {
        ctx.require_access(None, resource::CUSTOMER, action::CREATE)?;
        let id = self.id_generator.generate()?;
        self.repository.create(ctx, id, customer).await?;
        Ok(id)
    }

    async fn update(&self, ctx: &Context, id: i64, customer: &CustomerUpdate) -> DomainResult<()> {
        ctx.require_access(None, resource::CUSTOMER, action::UPDATE)?;
        self.repository.update(ctx, id, customer).await
    }

    async fn delete(&self, ctx: &Context, id: i64) -> DomainResult<()> {
        ctx.require_access(None, resource::CUSTOMER, action::DELETE)?;
        self.repository.delete(ctx, id).await
    }

    async fn get_by_number(&self, ctx: &Context, number: &str) -> DomainResult<Option<Customer>> {
        ctx.require_access(None, resource::CUSTOMER, action::READ)?;
        self.repository.get_by_number(ctx, number).await
    }

    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<Customer>> {
        ctx.require_access(None, resource::CUSTOMER, action::READ)?;
        self.repository.get_by_id(ctx, id).await
    }

    async fn get_all(
        &self,
        ctx: &Context,
        filter: &CustomerFilter,
        pagination: &PaginationOptions,
    ) -> DomainResult<Vec<Customer>> {
        ctx.require_access(None, resource::CUSTOMER, action::READ)?;
        self.repository.get_all(ctx, filter, pagination).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::create_mock_id_gen;
    use crate::domain::Error;
    use crate::domain::model::Update;
    use async_trait::async_trait;
    use chrono::Utc;
    use mockall::mock;
    use std::collections::HashMap;

    mock! {
        pub CustomerRepo {}
        #[async_trait]
        impl CustomerRepository for CustomerRepo {
            async fn create(&self, ctx: &Context, id: i64, customer: &CustomerCreate) -> DomainResult<()>;
            async fn update(&self, ctx: &Context, id: i64, customer: &CustomerUpdate) -> DomainResult<()>;
            async fn delete(&self, ctx: &Context, id: i64) -> DomainResult<()>;
            async fn get_all(&self, ctx: &Context, filter: &CustomerFilter, pagination: &PaginationOptions) -> DomainResult<Vec<Customer>>;
            async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<Customer>>;
            async fn get_by_number(&self, ctx: &Context, number: &str) -> DomainResult<Option<Customer>>;
        }
    }

    /// Creates a test context with full permissions for CUSTOMER resource
    fn create_test_context() -> Context {
        let mut permissions = HashMap::new();
        permissions.insert((resource::CUSTOMER, None), 0b1111);
        let ctx = Context::new().with_permission(permissions);
        ctx
    }

    /// Creates a test context with no permissions
    fn create_no_permission_context() -> Context {
        Context::new()
    }

    fn create_test_customer_create() -> CustomerCreate {
        CustomerCreate {
            number: "CUST001".to_string(),
            name: "Test Customer".to_string(),
            address: Some("123 Test St".to_string()),
            email: Some("test@customer.com".to_string()),
            phone: Some("555-1234".to_string()),
            level: 1,
            metadata: None,
        }
    }

    fn create_full_customer() -> Customer {
        Customer {
            id: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            is_deleted: false,
            number: "CUST001".to_string(),
            name: "Test Customer".to_string(),
            address: Some("123 Test St".to_string()),
            email: Some("test@customer.com".to_string()),
            phone: Some("555-1234".to_string()),
            level: 1,
            metadata: None,
        }
    }

    fn create_customer_update() -> CustomerUpdate {
        CustomerUpdate {
            number: Some("CUST002".to_string()),
            name: Some("Updated Customer".to_string()),
            address: Update::Unchanged,
            email: Update::Unchanged,
            phone: Update::Unchanged,
            level: Some(2),
            metadata: Update::Unchanged,
        }
    }

    fn create_default_filter() -> CustomerFilter {
        CustomerFilter {
            number: None,
            name: None,
            email: None,
            phone: None,
            level: None,
        }
    }

    fn create_default_pagination() -> PaginationOptions {
        PaginationOptions::new(1, 10, None)
    }

    // =============================================================================
    // Create Tests
    // =============================================================================

    #[tokio::test]
    async fn test_create_customer_success() {
        let mut mock_repo = MockCustomerRepo::new();
        let ctx = create_test_context();

        mock_repo
            .expect_create()
            .withf(|_, _, customer| customer.name == "Test Customer")
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = CustomerService::new(mock_repo, create_mock_id_gen(1));
        let customer = create_test_customer_create();
        let result = service.create(&ctx, &customer).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_customer_no_permission() {
        let ctx = create_no_permission_context();
        let service = CustomerService::new(MockCustomerRepo::new(), create_mock_id_gen(1));
        let customer = create_test_customer_create();

        let result = service.create(&ctx, &customer).await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_create_customer_repo_error() {
        let mut mock_repo = MockCustomerRepo::new();
        let ctx = create_test_context();

        mock_repo
            .expect_create()
            .times(1)
            .returning(|_, _, _| Err(Error::Database("DB Error".to_string())));

        let service = CustomerService::new(mock_repo, create_mock_id_gen(1));
        let customer = create_test_customer_create();
        let result = service.create(&ctx, &customer).await;

        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }

    // =============================================================================
    // Update Tests
    // =============================================================================

    #[tokio::test]
    async fn test_update_customer_success() {
        let mut mock_repo = MockCustomerRepo::new();
        let ctx = create_test_context();

        mock_repo
            .expect_update()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(1),
                mockall::predicate::always(),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = CustomerService::new(mock_repo, create_mock_id_gen(1));
        let update = create_customer_update();
        let result = service.update(&ctx, 1, &update).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_customer_no_permission() {
        let ctx = create_no_permission_context();
        let service = CustomerService::new(MockCustomerRepo::new(), create_mock_id_gen(1));
        let update = create_customer_update();

        let result = service.update(&ctx, 1, &update).await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_update_customer_repo_error() {
        let mut mock_repo = MockCustomerRepo::new();
        let ctx = create_test_context();

        mock_repo
            .expect_update()
            .times(1)
            .returning(|_, _, _| Err(Error::Database("DB Error".to_string())));

        let service = CustomerService::new(mock_repo, create_mock_id_gen(1));
        let update = create_customer_update();
        let result = service.update(&ctx, 1, &update).await;

        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }

    #[tokio::test]
    async fn test_update_customer_not_found() {
        let mut mock_repo = MockCustomerRepo::new();
        let ctx = create_test_context();

        mock_repo
            .expect_update()
            .times(1)
            .returning(|_, _, _| Err(Error::NotFound("Customer not found".to_string())));

        let service = CustomerService::new(mock_repo, create_mock_id_gen(1));
        let update = create_customer_update();
        let result = service.update(&ctx, 999, &update).await;

        assert!(matches!(result, Err(Error::NotFound(_))));
    }

    // =============================================================================
    // Delete Tests
    // =============================================================================

    #[tokio::test]
    async fn test_delete_customer_success() {
        let mut mock_repo = MockCustomerRepo::new();
        let ctx = create_test_context();

        mock_repo.expect_delete().times(1).returning(|_, _| Ok(()));

        let service = CustomerService::new(mock_repo, create_mock_id_gen(1));
        let result = service.delete(&ctx, 1).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_customer_no_permission() {
        let ctx = create_no_permission_context();
        let service = CustomerService::new(MockCustomerRepo::new(), create_mock_id_gen(1));

        let result = service.delete(&ctx, 1).await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_delete_customer_repo_error() {
        let mut mock_repo = MockCustomerRepo::new();
        let ctx = create_test_context();

        mock_repo
            .expect_delete()
            .times(1)
            .returning(|_, _| Err(Error::Database("DB Error".to_string())));

        let service = CustomerService::new(mock_repo, create_mock_id_gen(1));
        let result = service.delete(&ctx, 1).await;
        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }

    #[tokio::test]
    async fn test_delete_customer_not_found() {
        let mut mock_repo = MockCustomerRepo::new();
        let ctx = create_test_context();

        mock_repo
            .expect_delete()
            .times(1)
            .returning(|_, _| Err(Error::NotFound("Customer not found".to_string())));

        let service = CustomerService::new(mock_repo, create_mock_id_gen(1));
        let result = service.delete(&ctx, 999).await;
        assert!(matches!(result, Err(Error::NotFound(_))));
    }

    // =============================================================================
    // Get By Number Tests
    // =============================================================================

    #[tokio::test]
    async fn test_get_by_number_success() {
        let mut mock_repo = MockCustomerRepo::new();
        let ctx = create_test_context();

        let expected_customer = create_full_customer();
        let customer_clone = expected_customer.clone();

        mock_repo
            .expect_get_by_number()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq("CUST001"),
            )
            .times(1)
            .returning(move |_, _| Ok(Some(customer_clone.clone())));

        let service = CustomerService::new(mock_repo, create_mock_id_gen(1));
        let result = service.get_by_number(&ctx, "CUST001").await;

        assert!(result.is_ok());
        let customer = result.unwrap();
        assert!(customer.is_some());
        let customer = customer.unwrap();
        assert_eq!(customer.name, expected_customer.name);
        assert_eq!(customer.number, "CUST001");
    }

    #[tokio::test]
    async fn test_get_by_number_not_found() {
        let mut mock_repo = MockCustomerRepo::new();
        let ctx = create_test_context();

        mock_repo
            .expect_get_by_number()
            .times(1)
            .returning(|_, _| Ok(None));

        let service = CustomerService::new(mock_repo, create_mock_id_gen(1));
        let result = service.get_by_number(&ctx, "NONEXISTENT").await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_by_number_no_permission() {
        let ctx = create_no_permission_context();
        let service = CustomerService::new(MockCustomerRepo::new(), create_mock_id_gen(1));

        let result = service.get_by_number(&ctx, "CUST001").await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_get_by_number_repo_error() {
        let mut mock_repo = MockCustomerRepo::new();
        let ctx = create_test_context();

        mock_repo
            .expect_get_by_number()
            .times(1)
            .returning(|_, _| Err(Error::Database("DB Error".to_string())));

        let service = CustomerService::new(mock_repo, create_mock_id_gen(1));
        let result = service.get_by_number(&ctx, "CUST001").await;
        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }

    // =============================================================================
    // Get By ID Tests
    // =============================================================================

    #[tokio::test]
    async fn test_get_by_id_success() {
        let mut mock_repo = MockCustomerRepo::new();
        let ctx = create_test_context();

        let expected_customer = create_full_customer();
        let customer_clone = expected_customer.clone();

        mock_repo
            .expect_get_by_id()
            .with(mockall::predicate::always(), mockall::predicate::eq(1))
            .times(1)
            .returning(move |_, _| Ok(Some(customer_clone.clone())));

        let service = CustomerService::new(mock_repo, create_mock_id_gen(1));
        let result = service.get_by_id(&ctx, 1).await;

        assert!(result.is_ok());
        let customer = result.unwrap();
        assert!(customer.is_some());
        let customer = customer.unwrap();
        assert_eq!(customer.name, expected_customer.name);
        assert_eq!(customer.number, expected_customer.number);
    }

    #[tokio::test]
    async fn test_get_by_id_not_found() {
        let mut mock_repo = MockCustomerRepo::new();
        let ctx = create_test_context();

        mock_repo
            .expect_get_by_id()
            .times(1)
            .returning(|_, _| Ok(None));

        let service = CustomerService::new(mock_repo, create_mock_id_gen(1));
        let result = service.get_by_id(&ctx, 999).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_by_id_no_permission() {
        let ctx = create_no_permission_context();
        let service = CustomerService::new(MockCustomerRepo::new(), create_mock_id_gen(1));

        let result = service.get_by_id(&ctx, 1).await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_get_by_id_repo_error() {
        let mut mock_repo = MockCustomerRepo::new();
        let ctx = create_test_context();

        mock_repo
            .expect_get_by_id()
            .times(1)
            .returning(|_, _| Err(Error::Database("DB Error".to_string())));

        let service = CustomerService::new(mock_repo, create_mock_id_gen(1));
        let result = service.get_by_id(&ctx, 1).await;
        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }

    // =============================================================================
    // Get All Tests
    // =============================================================================

    #[tokio::test]
    async fn test_get_all_success() {
        let mut mock_repo = MockCustomerRepo::new();
        let ctx = create_test_context();

        let customers = vec![create_full_customer()];
        let customers_clone = customers.clone();

        mock_repo
            .expect_get_all()
            .times(1)
            .returning(move |_, _, _| Ok(customers_clone.clone()));

        let service = CustomerService::new(mock_repo, create_mock_id_gen(1));
        let filter = create_default_filter();
        let pagination = create_default_pagination();
        let result = service.get_all(&ctx, &filter, &pagination).await;

        assert!(result.is_ok());
        let result_customers = result.unwrap();
        assert_eq!(result_customers.len(), 1);
        assert_eq!(result_customers[0].name, "Test Customer");
    }

    #[tokio::test]
    async fn test_get_all_empty() {
        let mut mock_repo = MockCustomerRepo::new();
        let ctx = create_test_context();

        mock_repo
            .expect_get_all()
            .times(1)
            .returning(|_, _, _| Ok(vec![]));

        let service = CustomerService::new(mock_repo, create_mock_id_gen(1));
        let filter = create_default_filter();
        let pagination = create_default_pagination();
        let result = service.get_all(&ctx, &filter, &pagination).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_get_all_no_permission() {
        let ctx = create_no_permission_context();
        let service = CustomerService::new(MockCustomerRepo::new(), create_mock_id_gen(1));
        let filter = create_default_filter();
        let pagination = create_default_pagination();

        let result = service.get_all(&ctx, &filter, &pagination).await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_get_all_repo_error() {
        let mut mock_repo = MockCustomerRepo::new();
        let ctx = create_test_context();

        mock_repo
            .expect_get_all()
            .times(1)
            .returning(|_, _, _| Err(Error::Database("DB Error".to_string())));

        let service = CustomerService::new(mock_repo, create_mock_id_gen(1));
        let filter = create_default_filter();
        let pagination = create_default_pagination();
        let result = service.get_all(&ctx, &filter, &pagination).await;
        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }

    #[tokio::test]
    async fn test_get_all_with_filter() {
        let mut mock_repo = MockCustomerRepo::new();
        let ctx = create_test_context();

        let customers = vec![create_full_customer()];
        let customers_clone = customers.clone();

        mock_repo
            .expect_get_all()
            .withf(|_, filter, _| filter.name == Some("Test".to_string()))
            .times(1)
            .returning(move |_, _, _| Ok(customers_clone.clone()));

        let service = CustomerService::new(mock_repo, create_mock_id_gen(1));
        let filter = CustomerFilter {
            number: None,
            name: Some("Test".to_string()),
            email: None,
            phone: None,
            level: None,
        };
        let pagination = create_default_pagination();
        let result = service.get_all(&ctx, &filter, &pagination).await;

        assert!(result.is_ok());
        let result_customers = result.unwrap();
        assert_eq!(result_customers.len(), 1);
    }

    #[tokio::test]
    async fn test_get_all_with_pagination() {
        let mut mock_repo = MockCustomerRepo::new();
        let ctx = create_test_context();

        let customers = vec![create_full_customer()];
        let customers_clone = customers.clone();

        mock_repo
            .expect_get_all()
            .withf(|_, _, pagination| pagination.page == 2 && pagination.page_size == 20)
            .times(1)
            .returning(move |_, _, _| Ok(customers_clone.clone()));

        let service = CustomerService::new(mock_repo, create_mock_id_gen(1));
        let filter = create_default_filter();
        let pagination = PaginationOptions::new(2, 20, None);
        let result = service.get_all(&ctx, &filter, &pagination).await;

        assert!(result.is_ok());
    }
}
