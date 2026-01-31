use std::sync::Arc;

use async_trait::async_trait;
use sea_orm::DatabaseConnection;

use crate::{
    application::NumberServiceTrait,
    domain::{
        Context, DomainResult,
        model::{
            customer::{Customer, CustomerCreate, CustomerFilter, CustomerUpdate},
            pagination::PaginationOptions,
            permission::{action, resource},
        },
    },
    snowflake::IdGenerator,
    storage::{CustomerRepository, RepoCtx},
};

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
    number_service: Arc<dyn NumberServiceTrait>,
    db: DatabaseConnection,
}

impl<R, I> CustomerService<R, I>
where
    R: CustomerRepository,
    I: IdGenerator,
{
    pub fn new(
        repository: R,
        id_generator: I,
        number_service: Arc<dyn NumberServiceTrait>,
        db: DatabaseConnection,
    ) -> Self {
        Self {
            repository,
            id_generator,
            number_service,
            db,
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
        let mut customer_with_number = customer.clone();
        if customer_with_number.number.is_empty() {
            let generated_number = self.number_service.generate(ctx, "CUS", None, None).await?;
            customer_with_number.number = generated_number;
        }
        let repo_ctx = RepoCtx {
            ctx: ctx.clone(),
            db: self.db.clone(),
        };
        self.repository
            .create(&repo_ctx, id, &customer_with_number)
            .await?;
        Ok(id)
    }

    async fn update(&self, ctx: &Context, id: i64, customer: &CustomerUpdate) -> DomainResult<()> {
        ctx.require_access(None, resource::CUSTOMER, action::UPDATE)?;
        let repo_ctx = RepoCtx {
            ctx: ctx.clone(),
            db: self.db.clone(),
        };
        self.repository.update(&repo_ctx, id, customer).await
    }

    async fn delete(&self, ctx: &Context, id: i64) -> DomainResult<()> {
        ctx.require_access(None, resource::CUSTOMER, action::DELETE)?;
        let repo_ctx = RepoCtx {
            ctx: ctx.clone(),
            db: self.db.clone(),
        };
        self.repository.delete(&repo_ctx, id).await
    }

    async fn get_by_number(&self, ctx: &Context, number: &str) -> DomainResult<Option<Customer>> {
        ctx.require_access(None, resource::CUSTOMER, action::READ)?;
        let repo_ctx = RepoCtx {
            ctx: ctx.clone(),
            db: self.db.clone(),
        };
        self.repository.get_by_number(&repo_ctx, number).await
    }

    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<Customer>> {
        ctx.require_access(None, resource::CUSTOMER, action::READ)?;
        let repo_ctx = RepoCtx {
            ctx: ctx.clone(),
            db: self.db.clone(),
        };
        self.repository.get_by_id(&repo_ctx, id).await
    }

    async fn get_all(
        &self,
        ctx: &Context,
        filter: &CustomerFilter,
        pagination: &PaginationOptions,
    ) -> DomainResult<Vec<Customer>> {
        ctx.require_access(None, resource::CUSTOMER, action::READ)?;
        let repo_ctx = RepoCtx {
            ctx: ctx.clone(),
            db: self.db.clone(),
        };
        self.repository.get_all(&repo_ctx, filter, pagination).await
    }
}

#[cfg(test)]
#[allow(clippy::type_complexity)]
mod tests {
    use super::*;
    use crate::application::create_mock_id_gen;
    use crate::domain::Error;
    use crate::domain::model::Update;
    use crate::storage::RepoCtx;
    use async_trait::async_trait;
    use chrono::Utc;
    use sea_orm::{ConnectionTrait, Database, DatabaseConnection};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // Manual mock implementation that works with impl Trait
    #[derive(Clone)]
    struct MockCustomerRepo {
        create_fn: Arc<Mutex<Option<Box<dyn Fn(i64, CustomerCreate) -> DomainResult<()> + Send>>>>,
        update_fn: Arc<Mutex<Option<Box<dyn Fn(i64, CustomerUpdate) -> DomainResult<()> + Send>>>>,
        delete_fn: Arc<Mutex<Option<Box<dyn Fn(i64) -> DomainResult<()> + Send>>>>,
        get_by_id_fn: Arc<Mutex<Option<Box<dyn Fn(i64) -> DomainResult<Option<Customer>> + Send>>>>,
        get_by_number_fn:
            Arc<Mutex<Option<Box<dyn Fn(String) -> DomainResult<Option<Customer>> + Send>>>>,
        get_all_fn: Arc<
            Mutex<
                Option<
                    Box<
                        dyn Fn(CustomerFilter, PaginationOptions) -> DomainResult<Vec<Customer>>
                            + Send,
                    >,
                >,
            >,
        >,
    }

    impl MockCustomerRepo {
        fn new() -> Self {
            Self {
                create_fn: Arc::new(Mutex::new(None)),
                update_fn: Arc::new(Mutex::new(None)),
                delete_fn: Arc::new(Mutex::new(None)),
                get_by_id_fn: Arc::new(Mutex::new(None)),
                get_by_number_fn: Arc::new(Mutex::new(None)),
                get_all_fn: Arc::new(Mutex::new(None)),
            }
        }

        fn expect_create<F>(&mut self, f: F)
        where
            F: Fn(i64, CustomerCreate) -> DomainResult<()> + Send + 'static,
        {
            *self.create_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_update<F>(&mut self, f: F)
        where
            F: Fn(i64, CustomerUpdate) -> DomainResult<()> + Send + 'static,
        {
            *self.update_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_delete<F>(&mut self, f: F)
        where
            F: Fn(i64) -> DomainResult<()> + Send + 'static,
        {
            *self.delete_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_get_by_id<F>(&mut self, f: F)
        where
            F: Fn(i64) -> DomainResult<Option<Customer>> + Send + 'static,
        {
            *self.get_by_id_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_get_by_number<F>(&mut self, f: F)
        where
            F: Fn(String) -> DomainResult<Option<Customer>> + Send + 'static,
        {
            *self.get_by_number_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_get_all<F>(&mut self, f: F)
        where
            F: Fn(CustomerFilter, PaginationOptions) -> DomainResult<Vec<Customer>>
                + Send
                + 'static,
        {
            *self.get_all_fn.lock().unwrap() = Some(Box::new(f));
        }
    }

    #[async_trait]
    impl CustomerRepository for MockCustomerRepo {
        async fn create(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
            customer: &CustomerCreate,
        ) -> DomainResult<()> {
            let func = self.create_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(id, customer.clone())
            } else {
                panic!("create not mocked")
            }
        }

        async fn update(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
            customer: &CustomerUpdate,
        ) -> DomainResult<()> {
            let func = self.update_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(id, customer.clone())
            } else {
                panic!("update not mocked")
            }
        }

        async fn delete(&self, _ctx: &RepoCtx<impl ConnectionTrait>, id: i64) -> DomainResult<()> {
            let func = self.delete_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(id)
            } else {
                panic!("delete not mocked")
            }
        }

        async fn get_by_id(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
        ) -> DomainResult<Option<Customer>> {
            let func = self.get_by_id_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(id)
            } else {
                panic!("get_by_id not mocked")
            }
        }

        async fn get_by_number(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            number: &str,
        ) -> DomainResult<Option<Customer>> {
            let func = self.get_by_number_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(number.to_string())
            } else {
                panic!("get_by_number not mocked")
            }
        }

        async fn get_all(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            filter: &CustomerFilter,
            pagination: &PaginationOptions,
        ) -> DomainResult<Vec<Customer>> {
            let func = self.get_all_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(filter.clone(), pagination.clone())
            } else {
                panic!("get_all not mocked")
            }
        }
    }

    // Mock NumberService
    #[derive(Clone)]
    struct MockNumberService {
        generate_fn: Arc<
            Mutex<
                Option<
                    Box<dyn Fn(String, Option<i64>, Option<i32>) -> DomainResult<String> + Send>,
                >,
            >,
        >,
    }

    impl MockNumberService {
        fn new() -> Self {
            Self {
                generate_fn: Arc::new(Mutex::new(None)),
            }
        }

        fn expect_generate<F>(&mut self, f: F)
        where
            F: Fn(String, Option<i64>, Option<i32>) -> DomainResult<String> + Send + 'static,
        {
            *self.generate_fn.lock().unwrap() = Some(Box::new(f));
        }
    }

    #[async_trait]
    impl NumberServiceTrait for MockNumberService {
        async fn generate(
            &self,
            _ctx: &Context,
            prefix: &str,
            branch_id: Option<i64>,
            month: Option<i32>,
        ) -> DomainResult<String> {
            let func = self.generate_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(prefix.to_string(), branch_id, month)
            } else {
                panic!("generate not mocked")
            }
        }
    }

    async fn create_test_db() -> DatabaseConnection {
        Database::connect("sqlite::memory:").await.unwrap()
    }

    /// Creates a test context with full permissions for CUSTOMER resource
    fn create_test_context() -> Context {
        let mut permissions = HashMap::new();
        permissions.insert((resource::CUSTOMER, None), 0b1111);
        Context::new_with_all(None, permissions, HashMap::new())
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
        let mut mock_number_service = MockNumberService::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_create(|_, customer| {
            assert_eq!(customer.name, "Test Customer");
            Ok(())
        });
        mock_number_service.expect_generate(|prefix, _, _| {
            assert_eq!(prefix, "CUS");
            Ok("CUST001".to_string())
        });

        let service = CustomerService::new(
            mock_repo,
            create_mock_id_gen(1),
            Arc::new(mock_number_service),
            db,
        );
        let mut customer = create_test_customer_create();
        customer.number = "".to_string(); // Trigger number generation
        let result = service.create(&ctx, &customer).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_customer_no_permission() {
        let db = create_test_db().await;
        let ctx = create_no_permission_context();
        let service = CustomerService::new(
            MockCustomerRepo::new(),
            create_mock_id_gen(1),
            Arc::new(MockNumberService::new()),
            db,
        );
        let customer = create_test_customer_create();

        let result = service.create(&ctx, &customer).await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_create_customer_repo_error() {
        let mut mock_repo = MockCustomerRepo::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_create(|_, _| Err(Error::Database("DB Error".to_string())));

        let service = CustomerService::new(
            mock_repo,
            create_mock_id_gen(1),
            Arc::new(MockNumberService::new()),
            db,
        );
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
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_update(|id, _| {
            assert_eq!(id, 1);
            Ok(())
        });

        let service = CustomerService::new(
            mock_repo,
            create_mock_id_gen(1),
            Arc::new(MockNumberService::new()),
            db,
        );
        let update = create_customer_update();
        let result = service.update(&ctx, 1, &update).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_customer_no_permission() {
        let db = create_test_db().await;
        let ctx = create_no_permission_context();
        let service = CustomerService::new(
            MockCustomerRepo::new(),
            create_mock_id_gen(1),
            Arc::new(MockNumberService::new()),
            db,
        );
        let update = create_customer_update();

        let result = service.update(&ctx, 1, &update).await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_update_customer_repo_error() {
        let mut mock_repo = MockCustomerRepo::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_update(|_, _| Err(Error::Database("DB Error".to_string())));

        let service = CustomerService::new(
            mock_repo,
            create_mock_id_gen(1),
            Arc::new(MockNumberService::new()),
            db,
        );
        let update = create_customer_update();
        let result = service.update(&ctx, 1, &update).await;

        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }

    #[tokio::test]
    async fn test_update_customer_not_found() {
        let mut mock_repo = MockCustomerRepo::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_update(|_, _| Err(Error::NotFound("Customer not found".to_string())));

        let service = CustomerService::new(
            mock_repo,
            create_mock_id_gen(1),
            Arc::new(MockNumberService::new()),
            db,
        );
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
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_delete(|id| {
            assert_eq!(id, 1);
            Ok(())
        });

        let service = CustomerService::new(
            mock_repo,
            create_mock_id_gen(1),
            Arc::new(MockNumberService::new()),
            db,
        );
        let result = service.delete(&ctx, 1).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_customer_no_permission() {
        let db = create_test_db().await;
        let ctx = create_no_permission_context();
        let service = CustomerService::new(
            MockCustomerRepo::new(),
            create_mock_id_gen(1),
            Arc::new(MockNumberService::new()),
            db,
        );

        let result = service.delete(&ctx, 1).await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_delete_customer_repo_error() {
        let mut mock_repo = MockCustomerRepo::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_delete(|_| Err(Error::Database("DB Error".to_string())));

        let service = CustomerService::new(
            mock_repo,
            create_mock_id_gen(1),
            Arc::new(MockNumberService::new()),
            db,
        );
        let result = service.delete(&ctx, 1).await;
        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }

    #[tokio::test]
    async fn test_delete_customer_not_found() {
        let mut mock_repo = MockCustomerRepo::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_delete(|_| Err(Error::NotFound("Customer not found".to_string())));

        let service = CustomerService::new(
            mock_repo,
            create_mock_id_gen(1),
            Arc::new(MockNumberService::new()),
            db,
        );
        let result = service.delete(&ctx, 999).await;
        assert!(matches!(result, Err(Error::NotFound(_))));
    }

    // =============================================================================
    // Get By Number Tests
    // =============================================================================

    #[tokio::test]
    async fn test_get_by_number_success() {
        let mut mock_repo = MockCustomerRepo::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        let expected_customer = create_full_customer();
        let customer_clone = expected_customer.clone();

        mock_repo.expect_get_by_number(move |number| {
            assert_eq!(number, "CUST001");
            Ok(Some(customer_clone.clone()))
        });

        let service = CustomerService::new(
            mock_repo,
            create_mock_id_gen(1),
            Arc::new(MockNumberService::new()),
            db,
        );
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
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_get_by_number(|_| Ok(None));

        let service = CustomerService::new(
            mock_repo,
            create_mock_id_gen(1),
            Arc::new(MockNumberService::new()),
            db,
        );
        let result = service.get_by_number(&ctx, "NONEXISTENT").await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_by_number_no_permission() {
        let db = create_test_db().await;
        let ctx = create_no_permission_context();
        let service = CustomerService::new(
            MockCustomerRepo::new(),
            create_mock_id_gen(1),
            Arc::new(MockNumberService::new()),
            db,
        );

        let result = service.get_by_number(&ctx, "CUST001").await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_get_by_number_repo_error() {
        let mut mock_repo = MockCustomerRepo::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_get_by_number(|_| Err(Error::Database("DB Error".to_string())));

        let service = CustomerService::new(
            mock_repo,
            create_mock_id_gen(1),
            Arc::new(MockNumberService::new()),
            db,
        );
        let result = service.get_by_number(&ctx, "CUST001").await;
        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }

    // =============================================================================
    // Get By ID Tests
    // =============================================================================

    #[tokio::test]
    async fn test_get_by_id_success() {
        let mut mock_repo = MockCustomerRepo::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        let expected_customer = create_full_customer();
        let customer_clone = expected_customer.clone();

        mock_repo.expect_get_by_id(move |id| {
            assert_eq!(id, 1);
            Ok(Some(customer_clone.clone()))
        });

        let service = CustomerService::new(
            mock_repo,
            create_mock_id_gen(1),
            Arc::new(MockNumberService::new()),
            db,
        );
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
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_get_by_id(|_| Ok(None));

        let service = CustomerService::new(
            mock_repo,
            create_mock_id_gen(1),
            Arc::new(MockNumberService::new()),
            db,
        );
        let result = service.get_by_id(&ctx, 999).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_by_id_no_permission() {
        let db = create_test_db().await;
        let ctx = create_no_permission_context();
        let service = CustomerService::new(
            MockCustomerRepo::new(),
            create_mock_id_gen(1),
            Arc::new(MockNumberService::new()),
            db,
        );

        let result = service.get_by_id(&ctx, 1).await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_get_by_id_repo_error() {
        let mut mock_repo = MockCustomerRepo::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_get_by_id(|_| Err(Error::Database("DB Error".to_string())));

        let service = CustomerService::new(
            mock_repo,
            create_mock_id_gen(1),
            Arc::new(MockNumberService::new()),
            db,
        );
        let result = service.get_by_id(&ctx, 1).await;
        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }

    // =============================================================================
    // Get All Tests
    // =============================================================================

    #[tokio::test]
    async fn test_get_all_success() {
        let mut mock_repo = MockCustomerRepo::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        let customers = vec![create_full_customer()];
        let customers_clone = customers.clone();

        mock_repo.expect_get_all(move |_, _| Ok(customers_clone.clone()));

        let service = CustomerService::new(
            mock_repo,
            create_mock_id_gen(1),
            Arc::new(MockNumberService::new()),
            db,
        );
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
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_get_all(|_, _| Ok(vec![]));

        let service = CustomerService::new(
            mock_repo,
            create_mock_id_gen(1),
            Arc::new(MockNumberService::new()),
            db,
        );
        let filter = create_default_filter();
        let pagination = create_default_pagination();
        let result = service.get_all(&ctx, &filter, &pagination).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_get_all_no_permission() {
        let db = create_test_db().await;
        let ctx = create_no_permission_context();
        let service = CustomerService::new(
            MockCustomerRepo::new(),
            create_mock_id_gen(1),
            Arc::new(MockNumberService::new()),
            db,
        );
        let filter = create_default_filter();
        let pagination = create_default_pagination();

        let result = service.get_all(&ctx, &filter, &pagination).await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_get_all_repo_error() {
        let mut mock_repo = MockCustomerRepo::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_get_all(|_, _| Err(Error::Database("DB Error".to_string())));

        let service = CustomerService::new(
            mock_repo,
            create_mock_id_gen(1),
            Arc::new(MockNumberService::new()),
            db,
        );
        let filter = create_default_filter();
        let pagination = create_default_pagination();
        let result = service.get_all(&ctx, &filter, &pagination).await;
        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }

    #[tokio::test]
    async fn test_get_all_with_filter() {
        let mut mock_repo = MockCustomerRepo::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        let customers = vec![create_full_customer()];
        let customers_clone = customers.clone();

        mock_repo.expect_get_all(move |filter, _| {
            assert_eq!(filter.name, Some("Test".to_string()));
            Ok(customers_clone.clone())
        });

        let service = CustomerService::new(
            mock_repo,
            create_mock_id_gen(1),
            Arc::new(MockNumberService::new()),
            db,
        );
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
        let db = create_test_db().await;
        let ctx = create_test_context();

        let customers = vec![create_full_customer()];
        let customers_clone = customers.clone();

        mock_repo.expect_get_all(move |_, pagination| {
            assert_eq!(pagination.page, 2);
            assert_eq!(pagination.page_size, 20);
            Ok(customers_clone.clone())
        });

        let service = CustomerService::new(
            mock_repo,
            create_mock_id_gen(1),
            Arc::new(MockNumberService::new()),
            db,
        );
        let filter = create_default_filter();
        let pagination = PaginationOptions::new(2, 20, None);
        let result = service.get_all(&ctx, &filter, &pagination).await;

        assert!(result.is_ok());
    }
}
