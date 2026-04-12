use async_trait::async_trait;
use sea_orm::DatabaseConnection;

use crate::{
    application::ServiceDbHelper,
    domain::{
        Context, DomainResult,
        model::{
            payment_channel::{
                PaymentChannel, PaymentChannelCreate, PaymentChannelFilter,
                PaymentChannelPriorityUpdate, PaymentChannelUpdate,
            },
            permission::{action, resource},
        },
    },
    snowflake::IdGenerator,
    storage::PaymentChannelRepository,
};

#[async_trait]
pub trait PaymentChannelServiceTrait: Send + Sync {
    async fn create(&self, ctx: &Context, data: &PaymentChannelCreate) -> DomainResult<i64>;
    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<PaymentChannel>>;
    async fn get_all(
        &self,
        ctx: &Context,
        filter: &PaymentChannelFilter,
    ) -> DomainResult<Vec<PaymentChannel>>;
    async fn update(&self, ctx: &Context, id: i64, data: &PaymentChannelUpdate)
    -> DomainResult<()>;
    async fn delete(&self, ctx: &Context, id: i64) -> DomainResult<()>;
    async fn update_priorities(
        &self,
        ctx: &Context,
        updates: &[PaymentChannelPriorityUpdate],
    ) -> DomainResult<()>;
}

pub struct PaymentChannelService<R, I> {
    repo: R,
    id_generator: I,
    db: DatabaseConnection,
}

impl<R, I> PaymentChannelService<R, I>
where
    R: PaymentChannelRepository,
    I: IdGenerator,
{
    pub fn new(repo: R, id_generator: I, db: DatabaseConnection) -> Self {
        Self {
            repo,
            id_generator,
            db,
        }
    }
}

impl<R, I> ServiceDbHelper for PaymentChannelService<R, I>
where
    R: PaymentChannelRepository,
    I: IdGenerator,
{
    fn database(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl<R, I> PaymentChannelServiceTrait for PaymentChannelService<R, I>
where
    R: PaymentChannelRepository,
    I: IdGenerator,
{
    async fn create(&self, ctx: &Context, data: &PaymentChannelCreate) -> DomainResult<i64> {
        ctx.require_access(None, resource::PAYMENT_CHANNEL, action::CREATE)?;
        let id = self.id_generator.generate()?;
        let repo_ctx = self.repo_ctx(ctx);
        self.repo.create(&repo_ctx, id, data).await?;
        Ok(id)
    }

    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<PaymentChannel>> {
        ctx.require_access(None, resource::PAYMENT_CHANNEL, action::READ)?;
        let repo_ctx = self.repo_ctx(ctx);
        self.repo.get_by_id(&repo_ctx, id).await
    }

    async fn get_all(
        &self,
        ctx: &Context,
        filter: &PaymentChannelFilter,
    ) -> DomainResult<Vec<PaymentChannel>> {
        ctx.require_access(None, resource::PAYMENT_CHANNEL, action::READ)?;
        let repo_ctx = self.repo_ctx(ctx);
        self.repo.get_all(&repo_ctx, filter).await
    }

    async fn update(
        &self,
        ctx: &Context,
        id: i64,
        data: &PaymentChannelUpdate,
    ) -> DomainResult<()> {
        ctx.require_access(None, resource::PAYMENT_CHANNEL, action::UPDATE)?;
        let repo_ctx = self.repo_ctx(ctx);
        self.repo.update(&repo_ctx, id, data).await
    }

    async fn delete(&self, ctx: &Context, id: i64) -> DomainResult<()> {
        ctx.require_access(None, resource::PAYMENT_CHANNEL, action::DELETE)?;
        let repo_ctx = self.repo_ctx(ctx);
        self.repo.delete(&repo_ctx, id).await
    }

    async fn update_priorities(
        &self,
        ctx: &Context,
        updates: &[PaymentChannelPriorityUpdate],
    ) -> DomainResult<()> {
        ctx.require_access(None, resource::PAYMENT_CHANNEL, action::UPDATE)?;
        let repo_ctx = self.repo_ctx(ctx);
        self.repo.update_priorities(&repo_ctx, updates).await
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
    use sea_orm::{ConnectionTrait, Database};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct MockPaymentChannelRepo {
        create_fn:
            Arc<Mutex<Option<Box<dyn Fn(i64, PaymentChannelCreate) -> DomainResult<()> + Send>>>>,
        get_by_id_fn:
            Arc<Mutex<Option<Box<dyn Fn(i64) -> DomainResult<Option<PaymentChannel>> + Send>>>>,
        get_all_fn: Arc<
            Mutex<
                Option<
                    Box<dyn Fn(PaymentChannelFilter) -> DomainResult<Vec<PaymentChannel>> + Send>,
                >,
            >,
        >,
        update_fn:
            Arc<Mutex<Option<Box<dyn Fn(i64, PaymentChannelUpdate) -> DomainResult<()> + Send>>>>,
        delete_fn: Arc<Mutex<Option<Box<dyn Fn(i64) -> DomainResult<()> + Send>>>>,
        update_priorities_fn: Arc<
            Mutex<
                Option<Box<dyn Fn(Vec<PaymentChannelPriorityUpdate>) -> DomainResult<()> + Send>>,
            >,
        >,
    }

    impl MockPaymentChannelRepo {
        fn new() -> Self {
            Self {
                create_fn: Arc::new(Mutex::new(None)),
                get_by_id_fn: Arc::new(Mutex::new(None)),
                get_all_fn: Arc::new(Mutex::new(None)),
                update_fn: Arc::new(Mutex::new(None)),
                delete_fn: Arc::new(Mutex::new(None)),
                update_priorities_fn: Arc::new(Mutex::new(None)),
            }
        }

        fn expect_create<F>(&mut self, f: F)
        where
            F: Fn(i64, PaymentChannelCreate) -> DomainResult<()> + Send + 'static,
        {
            *self.create_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_get_by_id<F>(&mut self, f: F)
        where
            F: Fn(i64) -> DomainResult<Option<PaymentChannel>> + Send + 'static,
        {
            *self.get_by_id_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_get_all<F>(&mut self, f: F)
        where
            F: Fn(PaymentChannelFilter) -> DomainResult<Vec<PaymentChannel>> + Send + 'static,
        {
            *self.get_all_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_update<F>(&mut self, f: F)
        where
            F: Fn(i64, PaymentChannelUpdate) -> DomainResult<()> + Send + 'static,
        {
            *self.update_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_delete<F>(&mut self, f: F)
        where
            F: Fn(i64) -> DomainResult<()> + Send + 'static,
        {
            *self.delete_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_update_priorities<F>(&mut self, f: F)
        where
            F: Fn(Vec<PaymentChannelPriorityUpdate>) -> DomainResult<()> + Send + 'static,
        {
            *self.update_priorities_fn.lock().unwrap() = Some(Box::new(f));
        }
    }

    #[async_trait]
    impl PaymentChannelRepository for MockPaymentChannelRepo {
        async fn create(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
            data: &PaymentChannelCreate,
        ) -> DomainResult<()> {
            let f = self.create_fn.lock().unwrap();
            f.as_ref().expect("create not mocked")(id, data.clone())
        }

        async fn get_by_id(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
        ) -> DomainResult<Option<PaymentChannel>> {
            let f = self.get_by_id_fn.lock().unwrap();
            f.as_ref().expect("get_by_id not mocked")(id)
        }

        async fn get_all(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            filter: &PaymentChannelFilter,
        ) -> DomainResult<Vec<PaymentChannel>> {
            let f = self.get_all_fn.lock().unwrap();
            f.as_ref().expect("get_all not mocked")(filter.clone())
        }

        async fn update(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
            data: &PaymentChannelUpdate,
        ) -> DomainResult<()> {
            let f = self.update_fn.lock().unwrap();
            f.as_ref().expect("update not mocked")(id, data.clone())
        }

        async fn delete(&self, _ctx: &RepoCtx<impl ConnectionTrait>, id: i64) -> DomainResult<()> {
            let f = self.delete_fn.lock().unwrap();
            f.as_ref().expect("delete not mocked")(id)
        }

        async fn update_priorities(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            updates: &[PaymentChannelPriorityUpdate],
        ) -> DomainResult<()> {
            let f = self.update_priorities_fn.lock().unwrap();
            f.as_ref().expect("update_priorities not mocked")(updates.to_vec())
        }
    }

    async fn create_test_db() -> DatabaseConnection {
        Database::connect("sqlite::memory:").await.unwrap()
    }

    fn create_test_context() -> Context {
        let mut permissions = HashMap::new();
        permissions.insert((resource::PAYMENT_CHANNEL, None), 0b1111);
        Context::new_with_all(None, permissions, HashMap::new())
    }

    fn create_no_permission_context() -> Context {
        Context::new()
    }

    fn create_channel() -> PaymentChannel {
        PaymentChannel {
            id: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
            is_deleted: false,
            branch_id: None,
            name: "Cash".to_string(),
            priority: 1,
            metadata: None,
        }
    }

    fn create_channel_create() -> PaymentChannelCreate {
        PaymentChannelCreate {
            branch_id: None,
            name: "Cash".to_string(),
            priority: 1,
            metadata: None,
        }
    }

    // ==================== Create ====================

    #[tokio::test]
    async fn test_create_success() {
        let mut mock_repo = MockPaymentChannelRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_create(|id, data| {
            assert_eq!(id, 1);
            assert_eq!(data.name, "Cash");
            Ok(())
        });

        let service = PaymentChannelService::new(mock_repo, mock_id_gen, db);
        let result = service.create(&ctx, &create_channel_create()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_create_no_permission() {
        let mock_repo = MockPaymentChannelRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let ctx = create_no_permission_context();

        let service = PaymentChannelService::new(mock_repo, mock_id_gen, db);
        let result = service.create(&ctx, &create_channel_create()).await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    // ==================== Get By ID ====================

    #[tokio::test]
    async fn test_get_by_id_found() {
        let mut mock_repo = MockPaymentChannelRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_get_by_id(|id| {
            assert_eq!(id, 1);
            Ok(Some(create_channel()))
        });

        let service = PaymentChannelService::new(mock_repo, mock_id_gen, db);
        let result = service.get_by_id(&ctx, 1).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_get_by_id_not_found() {
        let mut mock_repo = MockPaymentChannelRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_get_by_id(|_| Ok(None));

        let service = PaymentChannelService::new(mock_repo, mock_id_gen, db);
        let result = service.get_by_id(&ctx, 99).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_by_id_no_permission() {
        let mock_repo = MockPaymentChannelRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let ctx = create_no_permission_context();

        let service = PaymentChannelService::new(mock_repo, mock_id_gen, db);
        let result = service.get_by_id(&ctx, 1).await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    // ==================== Get All ====================

    #[tokio::test]
    async fn test_get_all_success() {
        let mut mock_repo = MockPaymentChannelRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_get_all(|_| Ok(vec![create_channel()]));

        let service = PaymentChannelService::new(mock_repo, mock_id_gen, db);
        let result = service
            .get_all(&ctx, &PaymentChannelFilter::default())
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_get_all_no_permission() {
        let mock_repo = MockPaymentChannelRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let ctx = create_no_permission_context();

        let service = PaymentChannelService::new(mock_repo, mock_id_gen, db);
        let result = service
            .get_all(&ctx, &PaymentChannelFilter::default())
            .await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    // ==================== Update ====================

    #[tokio::test]
    async fn test_update_success() {
        let mut mock_repo = MockPaymentChannelRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_update(|id, data| {
            assert_eq!(id, 1);
            assert_eq!(data.name, Some("QRIS".to_string()));
            Ok(())
        });

        let service = PaymentChannelService::new(mock_repo, mock_id_gen, db);
        let update = PaymentChannelUpdate {
            name: Some("QRIS".to_string()),
            ..Default::default()
        };
        let result = service.update(&ctx, 1, &update).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_not_found() {
        let mut mock_repo = MockPaymentChannelRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_update(|id, _| {
            Err(Error::NotFound(format!(
                "Payment channel with id {} not found",
                id
            )))
        });

        let service = PaymentChannelService::new(mock_repo, mock_id_gen, db);
        let result = service
            .update(&ctx, 99, &PaymentChannelUpdate::default())
            .await;
        assert!(matches!(result, Err(Error::NotFound(_))));
    }

    #[tokio::test]
    async fn test_update_no_permission() {
        let mock_repo = MockPaymentChannelRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let ctx = create_no_permission_context();

        let service = PaymentChannelService::new(mock_repo, mock_id_gen, db);
        let result = service
            .update(&ctx, 1, &PaymentChannelUpdate::default())
            .await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    // ==================== Delete ====================

    #[tokio::test]
    async fn test_delete_success() {
        let mut mock_repo = MockPaymentChannelRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_delete(|id| {
            assert_eq!(id, 1);
            Ok(())
        });

        let service = PaymentChannelService::new(mock_repo, mock_id_gen, db);
        let result = service.delete(&ctx, 1).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_not_found() {
        let mut mock_repo = MockPaymentChannelRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_delete(|id| {
            Err(Error::NotFound(format!(
                "Payment channel with id {} not found",
                id
            )))
        });

        let service = PaymentChannelService::new(mock_repo, mock_id_gen, db);
        let result = service.delete(&ctx, 99).await;
        assert!(matches!(result, Err(Error::NotFound(_))));
    }

    #[tokio::test]
    async fn test_delete_no_permission() {
        let mock_repo = MockPaymentChannelRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let ctx = create_no_permission_context();

        let service = PaymentChannelService::new(mock_repo, mock_id_gen, db);
        let result = service.delete(&ctx, 1).await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    // ==================== Update Priorities ====================

    #[tokio::test]
    async fn test_update_priorities_success() {
        let mut mock_repo = MockPaymentChannelRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_update_priorities(|updates| {
            assert_eq!(updates.len(), 2);
            assert_eq!(updates[0].id, 1);
            assert_eq!(updates[0].priority, 2);
            assert_eq!(updates[1].id, 2);
            assert_eq!(updates[1].priority, 1);
            Ok(())
        });

        let service = PaymentChannelService::new(mock_repo, mock_id_gen, db);
        let updates = vec![
            PaymentChannelPriorityUpdate { id: 1, priority: 2 },
            PaymentChannelPriorityUpdate { id: 2, priority: 1 },
        ];
        let result = service.update_priorities(&ctx, &updates).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_priorities_no_permission() {
        let mock_repo = MockPaymentChannelRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let ctx = create_no_permission_context();

        let service = PaymentChannelService::new(mock_repo, mock_id_gen, db);
        let result = service.update_priorities(&ctx, &[]).await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }
}
