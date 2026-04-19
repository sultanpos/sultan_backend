use std::sync::Arc;

use async_trait::async_trait;
use chrono::Datelike;
use chrono::Local;
use sea_orm::DatabaseConnection;

use crate::application::NumberServiceTrait;
use crate::application::ServiceDbHelper;
use crate::domain::Context;
use crate::domain::DomainResult;
use crate::domain::model::permission::{action, resource};
use crate::domain::model::purchase_order::PurchaseOrderCreate;
use crate::domain::model::purchase_order::PurchaseOrderUpdate;
use crate::snowflake::IdGenerator;
use crate::storage::PurchaseOrderRepository;

#[async_trait]
pub trait PurchaseOrderServiceTrait: Send + Sync {
    async fn create(&self, ctx: &Context, data: &PurchaseOrderCreate) -> DomainResult<i64>;
    async fn update(
        &self,
        ctx: &Context,
        branch_id: i64,
        id: i64,
        data: &PurchaseOrderUpdate,
    ) -> DomainResult<()>;
}

pub struct PurchaseOrderService<R, I> {
    repository: R,
    id_generator: I,
    number_service: Arc<dyn NumberServiceTrait>,
    db: DatabaseConnection,
}

impl<R: PurchaseOrderRepository, I: IdGenerator> PurchaseOrderService<R, I> {
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

impl<R: PurchaseOrderRepository, I: IdGenerator> ServiceDbHelper for PurchaseOrderService<R, I> {
    fn database(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl<R: PurchaseOrderRepository, I: IdGenerator> PurchaseOrderServiceTrait
    for PurchaseOrderService<R, I>
{
    async fn create(&self, ctx: &Context, data: &PurchaseOrderCreate) -> DomainResult<i64> {
        ctx.require_access(
            Some(data.branch_id),
            resource::PURCHASE_ORDER,
            action::CREATE,
        )?;
        let id = self.id_generator.generate()?;
        let mut create_with_number = data.clone();
        let now = Local::now();
        let month = now.month() as i32;
        create_with_number.number = self
            .number_service
            .generate(ctx, "PO", Some(data.branch_id), Some(month))
            .await?;
        let repo_ctx = self.repo_ctx(ctx);
        self.repository.create(&repo_ctx, id, data).await?;
        Ok(id)
    }

    async fn update(
        &self,
        ctx: &Context,
        branch_id: i64,
        id: i64,
        data: &PurchaseOrderUpdate,
    ) -> DomainResult<()> {
        ctx.require_access(Some(branch_id), resource::PURCHASE_ORDER, action::UPDATE)?;
        let repo_ctx = self.repo_ctx(ctx);
        self.repository.update(&repo_ctx, id, data).await?;
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::type_complexity)]
mod tests {
    use super::*;
    use crate::application::create_mock_id_gen;
    use crate::domain::Error;
    use crate::storage::RepoCtx;
    use async_trait::async_trait;
    use sea_orm::{ConnectionTrait, Database, DatabaseConnection};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    fn create_test_context() -> Context {
        let mut permissions = HashMap::new();
        permissions.insert((resource::PURCHASE_ORDER, Some(1i64)), 0b1111);
        Context::new_with_all(None, permissions, HashMap::new())
    }

    fn create_unauthorized_context() -> Context {
        Context::new_with_all(None, HashMap::new(), HashMap::new())
    }

    struct MockPurchaseOrderRepo {
        create_fn:
            Arc<Mutex<Option<Box<dyn Fn(i64, PurchaseOrderCreate) -> DomainResult<()> + Send>>>>,
        update_fn:
            Arc<Mutex<Option<Box<dyn Fn(i64, PurchaseOrderUpdate) -> DomainResult<()> + Send>>>>,
    }

    impl MockPurchaseOrderRepo {
        fn new() -> Self {
            Self {
                create_fn: Arc::new(Mutex::new(None)),
                update_fn: Arc::new(Mutex::new(None)),
            }
        }

        fn expect_create<F>(&mut self, f: F)
        where
            F: Fn(i64, PurchaseOrderCreate) -> DomainResult<()> + Send + 'static,
        {
            *self.create_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_update<F>(&mut self, f: F)
        where
            F: Fn(i64, PurchaseOrderUpdate) -> DomainResult<()> + Send + 'static,
        {
            *self.update_fn.lock().unwrap() = Some(Box::new(f));
        }
    }

    use crate::application::test_helpers::MockNumberService;

    #[async_trait]
    impl PurchaseOrderRepository for MockPurchaseOrderRepo {
        async fn create(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
            data: &PurchaseOrderCreate,
        ) -> DomainResult<()> {
            let func = self.create_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(id, data.clone())
            } else {
                panic!("create not mocked")
            }
        }

        async fn add_payment(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            _purchase_order_id: i64,
            _payment_id: i64,
            _data: &crate::domain::model::purchase_order::PurchasePaymentCreate,
        ) -> DomainResult<()> {
            panic!("add_payment not mocked")
        }

        async fn get_by_id(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            _id: i64,
        ) -> DomainResult<Option<crate::domain::model::purchase_order::PurchaseOrder>> {
            panic!("get_by_id not mocked")
        }

        async fn update(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
            data: &crate::domain::model::purchase_order::PurchaseOrderUpdate,
        ) -> DomainResult<()> {
            let func = self.update_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(id, data.clone())
            } else {
                panic!("update not mocked")
            }
        }

        async fn delete(&self, _ctx: &RepoCtx<impl ConnectionTrait>, _id: i64) -> DomainResult<()> {
            panic!("delete not mocked")
        }

        async fn get_all(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            _query: &crate::domain::model::purchase_order::PurchaseOrderQuery,
        ) -> DomainResult<crate::domain::model::purchase_order::PurchaseOrderPage> {
            panic!("get_all not mocked")
        }

        async fn add_item(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            _purchase_order_id: i64,
            _item_id: i64,
            _data: &crate::domain::model::purchase_order::PurchaseOrderItemCreate,
        ) -> DomainResult<()> {
            panic!("add_item not mocked")
        }

        async fn update_item(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            _item_id: i64,
            _data: &crate::domain::model::purchase_order::PurchaseOrderItemUpdate,
        ) -> DomainResult<()> {
            panic!("update_item not mocked")
        }

        async fn delete_item(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            _item_id: i64,
        ) -> DomainResult<()> {
            panic!("delete_item not mocked")
        }

        async fn get_items(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            _purchase_order_id: i64,
        ) -> DomainResult<Vec<crate::domain::model::purchase_order::PurchaseOrderItem>> {
            panic!("get_items not mocked")
        }

        async fn get_payments(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            _purchase_order_id: i64,
        ) -> DomainResult<Vec<crate::domain::model::purchase_order::PurchasePayment>> {
            panic!("get_payments not mocked")
        }

        async fn update_payment(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            _payment_id: i64,
            _data: &crate::domain::model::purchase_order::PurchasePaymentUpdate,
        ) -> DomainResult<()> {
            panic!("update_payment not mocked")
        }

        async fn delete_payment(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            _payment_id: i64,
        ) -> DomainResult<()> {
            panic!("delete_payment not mocked")
        }
    }

    async fn create_test_db() -> DatabaseConnection {
        Database::connect("sqlite::memory:").await.unwrap()
    }

    fn make_create_data() -> PurchaseOrderCreate {
        PurchaseOrderCreate {
            branch_id: 1,
            supplier_id: None,
            number: "PO-0001".to_string(),
            reference_number: None,
            order_date: None,
            expected_date: None,
            payment_due_date: None,
            discount_amount: 0,
            notes: None,
            metadata: None,
        }
    }

    #[tokio::test]
    async fn test_create_success() {
        let db = create_test_db().await;
        let mut repo = MockPurchaseOrderRepo::new();
        repo.expect_create(|_id, _data| Ok(()));
        let id_gen = create_mock_id_gen(42);
        let mut mock_number_service = MockNumberService::new();
        mock_number_service.expect_generate(|prefix, _, _| {
            assert_eq!(prefix, "PO");
            Ok("PO001".to_string())
        });

        let service = PurchaseOrderService::new(repo, id_gen, Arc::new(mock_number_service), db);
        let ctx = create_test_context();

        let result = service.create(&ctx, &make_create_data()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_create_unauthorized() {
        let db = create_test_db().await;
        let repo = MockPurchaseOrderRepo::new();
        let id_gen = create_mock_id_gen(42);
        let mut mock_number_service = MockNumberService::new();
        mock_number_service.expect_generate(|prefix, _, _| {
            assert_eq!(prefix, "PO");
            Ok("PO001".to_string())
        });

        let service = PurchaseOrderService::new(repo, id_gen, Arc::new(mock_number_service), db);
        let ctx = create_unauthorized_context();

        let result = service.create(&ctx, &make_create_data()).await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_create_repo_error_propagated() {
        let db = create_test_db().await;
        let mut repo = MockPurchaseOrderRepo::new();
        repo.expect_create(|_id, _data| Err(Error::Internal("db error".to_string())));
        let id_gen = create_mock_id_gen(99);
        let mut mock_number_service = MockNumberService::new();
        mock_number_service.expect_generate(|prefix, _, _| {
            assert_eq!(prefix, "PO");
            Ok("PO001".to_string())
        });

        let service = PurchaseOrderService::new(repo, id_gen, Arc::new(mock_number_service), db);
        let ctx = create_test_context();

        let result = service.create(&ctx, &make_create_data()).await;
        assert!(matches!(result, Err(Error::Internal(_))));
    }

    #[tokio::test]
    async fn test_create_id_generator_error() {
        let db = create_test_db().await;
        let repo = MockPurchaseOrderRepo::new();
        let mut mock_id_gen = crate::application::MockIdGen::new();
        mock_id_gen
            .expect_generate()
            .returning(|| Err(crate::snowflake::SnowflakeError::InvalidNode(999)));
        let mut mock_number_service = MockNumberService::new();
        mock_number_service.expect_generate(|prefix, _, _| {
            assert_eq!(prefix, "PO");
            Ok("PO001".to_string())
        });

        let service =
            PurchaseOrderService::new(repo, mock_id_gen, Arc::new(mock_number_service), db);
        let ctx = create_test_context();

        let result = service.create(&ctx, &make_create_data()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_success_with_optional_fields() {
        let db = create_test_db().await;
        let mut repo = MockPurchaseOrderRepo::new();
        repo.expect_create(|_id, data| {
            assert_eq!(data.number, "PO-0002");
            assert_eq!(data.supplier_id, Some(10));
            assert_eq!(data.discount_amount, 500);
            assert!(data.notes.is_some());
            Ok(())
        });
        let id_gen = create_mock_id_gen(100);
        let mut mock_number_service = MockNumberService::new();
        mock_number_service.expect_generate(|prefix, _, _| {
            assert_eq!(prefix, "PO");
            Ok("PO001".to_string())
        });

        let service = PurchaseOrderService::new(repo, id_gen, Arc::new(mock_number_service), db);
        let ctx = create_test_context();

        let data = PurchaseOrderCreate {
            branch_id: 1,
            supplier_id: Some(10),
            number: "PO-0002".to_string(),
            reference_number: Some("REF-001".to_string()),
            order_date: Some("2026-04-15T00:00:00.000Z".to_string()),
            expected_date: Some("2026-04-30T00:00:00.000Z".to_string()),
            payment_due_date: Some("2026-05-15T00:00:00.000Z".to_string()),
            discount_amount: 500,
            notes: Some("Test notes".to_string()),
            metadata: None,
        };

        let result = service.create(&ctx, &data).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100);
    }

    fn make_update_data() -> PurchaseOrderUpdate {
        PurchaseOrderUpdate {
            supplier_id: None,
            reference_number: crate::domain::model::update::Update::Unchanged,
            status: None,
            order_date: crate::domain::model::update::Update::Unchanged,
            expected_date: crate::domain::model::update::Update::Unchanged,
            received_date: crate::domain::model::update::Update::Unchanged,
            subtotal: None,
            discount_amount: None,
            total_amount: None,
            payment_status: None,
            payment_due_date: crate::domain::model::update::Update::Unchanged,
            paid_amount: None,
            returned_amount: None,
            notes: crate::domain::model::update::Update::Unchanged,
            metadata: crate::domain::model::update::Update::Unchanged,
        }
    }

    #[tokio::test]
    async fn test_update_success() {
        let db = create_test_db().await;
        let mut repo = MockPurchaseOrderRepo::new();
        repo.expect_update(|id, _data| {
            assert_eq!(id, 10);
            Ok(())
        });
        let id_gen = create_mock_id_gen(0);
        let mock_number_service = MockNumberService::new();

        let service = PurchaseOrderService::new(repo, id_gen, Arc::new(mock_number_service), db);
        let ctx = create_test_context();

        let result = service.update(&ctx, 1, 10, &make_update_data()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_unauthorized() {
        let db = create_test_db().await;
        let repo = MockPurchaseOrderRepo::new();
        let id_gen = create_mock_id_gen(0);
        let mock_number_service = MockNumberService::new();

        let service = PurchaseOrderService::new(repo, id_gen, Arc::new(mock_number_service), db);
        let ctx = create_unauthorized_context();

        let result = service.update(&ctx, 1, 10, &make_update_data()).await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_update_repo_error_propagated() {
        let db = create_test_db().await;
        let mut repo = MockPurchaseOrderRepo::new();
        repo.expect_update(|_id, _data| Err(Error::NotFound("not found".to_string())));
        let id_gen = create_mock_id_gen(0);
        let mock_number_service = MockNumberService::new();

        let service = PurchaseOrderService::new(repo, id_gen, Arc::new(mock_number_service), db);
        let ctx = create_test_context();

        let result = service.update(&ctx, 1, 99, &make_update_data()).await;
        assert!(matches!(result, Err(Error::NotFound(_))));
    }
}
