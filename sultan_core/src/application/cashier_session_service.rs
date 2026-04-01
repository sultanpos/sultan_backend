use async_trait::async_trait;
use sea_orm::DatabaseConnection;

use crate::{
    application::ServiceDbHelper,
    domain::{
        Context, DomainResult,
        error::Error,
        model::{
            cashier_session::{
                CashierSession, CashierSessionClose, CashierSessionCreate, CashierSessionPage,
                CashierSessionQuery,
            },
            permission::{action, resource},
        },
    },
    snowflake::IdGenerator,
    storage::CashierSessionRepository,
};

#[async_trait]
pub trait CashierSessionServiceTrait: Send + Sync {
    /// Opens a new cashier session. Returns a `ValidationError` if the user
    /// already has an open session on the given branch.
    async fn open_session(&self, ctx: &Context, data: &CashierSessionCreate) -> DomainResult<i64>;

    /// Closes an open session. Returns `NotFound` if there is no open session
    /// with the given id.
    async fn close_session(
        &self,
        ctx: &Context,
        id: i64,
        data: &CashierSessionClose,
    ) -> DomainResult<()>;

    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<CashierSession>>;

    /// Returns the currently open session for a user on a branch, if any.
    async fn get_current_session(
        &self,
        ctx: &Context,
        branch_id: i64,
        user_id: i64,
    ) -> DomainResult<Option<CashierSession>>;

    async fn get_all(
        &self,
        ctx: &Context,
        query: &CashierSessionQuery,
    ) -> DomainResult<CashierSessionPage>;

    async fn delete(&self, ctx: &Context, id: i64) -> DomainResult<()>;
}

pub struct CashierSessionService<R, I> {
    repository: R,
    id_generator: I,
    db: DatabaseConnection,
}

impl<R, I> CashierSessionService<R, I>
where
    R: CashierSessionRepository,
    I: IdGenerator,
{
    pub fn new(repository: R, id_generator: I, db: DatabaseConnection) -> Self {
        Self {
            repository,
            id_generator,
            db,
        }
    }
}

impl<R, I> ServiceDbHelper for CashierSessionService<R, I>
where
    R: CashierSessionRepository,
    I: IdGenerator,
{
    fn database(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl<R, I> CashierSessionServiceTrait for CashierSessionService<R, I>
where
    R: CashierSessionRepository,
    I: IdGenerator,
{
    async fn open_session(&self, ctx: &Context, data: &CashierSessionCreate) -> DomainResult<i64> {
        ctx.require_access(None, resource::CASHIER_SESSION, action::CREATE)?;

        let repo_ctx = self.repo_ctx(ctx);
        let existing = self
            .repository
            .get_open_by_user(&repo_ctx, data.branch_id, data.user_id)
            .await?;

        if existing.is_some() {
            return Err(Error::ValidationError(
                "User already has an open cashier session on this branch".to_string(),
            ));
        }

        let id = self.id_generator.generate()?;
        self.repository.create(&repo_ctx, id, data).await?;
        Ok(id)
    }

    async fn close_session(
        &self,
        ctx: &Context,
        id: i64,
        data: &CashierSessionClose,
    ) -> DomainResult<()> {
        ctx.require_access(None, resource::CASHIER_SESSION, action::UPDATE)?;
        let repo_ctx = self.repo_ctx(ctx);
        self.repository.close(&repo_ctx, id, data).await
    }

    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<CashierSession>> {
        ctx.require_access(None, resource::CASHIER_SESSION, action::READ)?;
        let repo_ctx = self.repo_ctx(ctx);
        self.repository.get_by_id(&repo_ctx, id).await
    }

    async fn get_current_session(
        &self,
        ctx: &Context,
        branch_id: i64,
        user_id: i64,
    ) -> DomainResult<Option<CashierSession>> {
        ctx.require_access(None, resource::CASHIER_SESSION, action::READ)?;
        let repo_ctx = self.repo_ctx(ctx);
        self.repository
            .get_open_by_user(&repo_ctx, branch_id, user_id)
            .await
    }

    async fn get_all(
        &self,
        ctx: &Context,
        query: &CashierSessionQuery,
    ) -> DomainResult<CashierSessionPage> {
        ctx.require_access(None, resource::CASHIER_SESSION, action::READ)?;
        let repo_ctx = self.repo_ctx(ctx);
        self.repository.get_all(&repo_ctx, query).await
    }

    async fn delete(&self, ctx: &Context, id: i64) -> DomainResult<()> {
        ctx.require_access(None, resource::CASHIER_SESSION, action::DELETE)?;
        let repo_ctx = self.repo_ctx(ctx);
        self.repository.delete(&repo_ctx, id).await
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::type_complexity)]
mod tests {
    use super::*;
    use crate::application::create_mock_id_gen;
    use crate::domain::Error;
    use crate::domain::model::cashier_session::{
        CashierSessionFilter, CashierSessionPage, CashierSessionQuery, CashierSessionSortField,
        SessionStatus,
    };
    use crate::domain::model::product::SortDirection;
    use crate::storage::RepoCtx;
    use async_trait::async_trait;
    use chrono::Utc;
    use sea_orm::{ConnectionTrait, Database, DatabaseConnection};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // ── Manual mock ──────────────────────────────────────────────────────────

    #[derive(Clone)]
    struct MockCashierSessionRepo {
        create_fn:
            Arc<Mutex<Option<Box<dyn Fn(i64, CashierSessionCreate) -> DomainResult<()> + Send>>>>,
        get_by_id_fn:
            Arc<Mutex<Option<Box<dyn Fn(i64) -> DomainResult<Option<CashierSession>> + Send>>>>,
        get_open_by_user_fn: Arc<
            Mutex<Option<Box<dyn Fn(i64, i64) -> DomainResult<Option<CashierSession>> + Send>>>,
        >,
        close_fn:
            Arc<Mutex<Option<Box<dyn Fn(i64, CashierSessionClose) -> DomainResult<()> + Send>>>>,
        get_all_fn: Arc<
            Mutex<
                Option<Box<dyn Fn(CashierSessionQuery) -> DomainResult<CashierSessionPage> + Send>>,
            >,
        >,
        delete_fn: Arc<Mutex<Option<Box<dyn Fn(i64) -> DomainResult<()> + Send>>>>,
    }

    impl MockCashierSessionRepo {
        fn new() -> Self {
            Self {
                create_fn: Arc::new(Mutex::new(None)),
                get_by_id_fn: Arc::new(Mutex::new(None)),
                get_open_by_user_fn: Arc::new(Mutex::new(None)),
                close_fn: Arc::new(Mutex::new(None)),
                get_all_fn: Arc::new(Mutex::new(None)),
                delete_fn: Arc::new(Mutex::new(None)),
            }
        }

        fn expect_create<F>(&mut self, f: F)
        where
            F: Fn(i64, CashierSessionCreate) -> DomainResult<()> + Send + 'static,
        {
            *self.create_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_get_by_id<F>(&mut self, f: F)
        where
            F: Fn(i64) -> DomainResult<Option<CashierSession>> + Send + 'static,
        {
            *self.get_by_id_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_get_open_by_user<F>(&mut self, f: F)
        where
            F: Fn(i64, i64) -> DomainResult<Option<CashierSession>> + Send + 'static,
        {
            *self.get_open_by_user_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_close<F>(&mut self, f: F)
        where
            F: Fn(i64, CashierSessionClose) -> DomainResult<()> + Send + 'static,
        {
            *self.close_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_get_all<F>(&mut self, f: F)
        where
            F: Fn(CashierSessionQuery) -> DomainResult<CashierSessionPage> + Send + 'static,
        {
            *self.get_all_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_delete<F>(&mut self, f: F)
        where
            F: Fn(i64) -> DomainResult<()> + Send + 'static,
        {
            *self.delete_fn.lock().unwrap() = Some(Box::new(f));
        }
    }

    #[async_trait]
    impl CashierSessionRepository for MockCashierSessionRepo {
        async fn create(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
            data: &CashierSessionCreate,
        ) -> DomainResult<()> {
            let func = self.create_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(id, data.clone())
            } else {
                panic!("create not mocked")
            }
        }

        async fn get_by_id(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
        ) -> DomainResult<Option<CashierSession>> {
            let func = self.get_by_id_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(id)
            } else {
                panic!("get_by_id not mocked")
            }
        }

        async fn get_open_by_user(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            branch_id: i64,
            user_id: i64,
        ) -> DomainResult<Option<CashierSession>> {
            let func = self.get_open_by_user_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(branch_id, user_id)
            } else {
                panic!("get_open_by_user not mocked")
            }
        }

        async fn close(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
            data: &CashierSessionClose,
        ) -> DomainResult<()> {
            let func = self.close_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(id, data.clone())
            } else {
                panic!("close not mocked")
            }
        }

        async fn get_all(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            query: &CashierSessionQuery,
        ) -> DomainResult<CashierSessionPage> {
            let func = self.get_all_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(query.clone())
            } else {
                panic!("get_all not mocked")
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
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    async fn create_test_db() -> DatabaseConnection {
        Database::connect("sqlite::memory:").await.unwrap()
    }

    fn ctx_with_all_permissions() -> Context {
        let mut permissions = HashMap::new();
        permissions.insert((resource::CASHIER_SESSION, None), 0b1111);
        Context::new_with_all(None, permissions, HashMap::new())
    }

    fn ctx_no_permissions() -> Context {
        Context::new_with_all(None, HashMap::new(), HashMap::new())
    }

    fn sample_session() -> CashierSession {
        CashierSession {
            id: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            is_deleted: false,
            branch_id: 10,
            user_id: 20,
            opened_at: Utc::now(),
            closed_at: None,
            status: SessionStatus::Open,
            opening_cash: 100_000,
            closing_cash: None,
            notes: None,
        }
    }

    fn sample_create() -> CashierSessionCreate {
        CashierSessionCreate {
            branch_id: 10,
            user_id: 20,
            opening_cash: 100_000,
            notes: None,
        }
    }

    fn sample_close() -> CashierSessionClose {
        CashierSessionClose {
            closing_cash: 200_000,
            notes: None,
        }
    }

    fn sample_query() -> CashierSessionQuery {
        CashierSessionQuery {
            filter: CashierSessionFilter::default(),
            sort_field: CashierSessionSortField::OpenedAt,
            sort_direction: SortDirection::Asc,
            cursor: None,
            limit: 20,
        }
    }

    // ── open_session ─────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_open_session_success() {
        let mut mock = MockCashierSessionRepo::new();
        mock.expect_get_open_by_user(|_, _| Ok(None));
        mock.expect_create(|_, _| Ok(()));
        let svc = CashierSessionService::new(mock, create_mock_id_gen(42), create_test_db().await);
        let result = svc
            .open_session(&ctx_with_all_permissions(), &sample_create())
            .await;
        assert!(matches!(result, Ok(42)));
    }

    #[tokio::test]
    async fn test_open_session_already_open() {
        let mut mock = MockCashierSessionRepo::new();
        mock.expect_get_open_by_user(|_, _| Ok(Some(sample_session())));
        let svc = CashierSessionService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc
            .open_session(&ctx_with_all_permissions(), &sample_create())
            .await;
        assert!(matches!(result, Err(Error::ValidationError(_))));
    }

    #[tokio::test]
    async fn test_open_session_forbidden() {
        let mock = MockCashierSessionRepo::new();
        let svc = CashierSessionService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc
            .open_session(&ctx_no_permissions(), &sample_create())
            .await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_open_session_repo_error() {
        let mut mock = MockCashierSessionRepo::new();
        mock.expect_get_open_by_user(|_, _| Err(Error::Database("db error".to_string())));
        let svc = CashierSessionService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc
            .open_session(&ctx_with_all_permissions(), &sample_create())
            .await;
        assert!(matches!(result, Err(Error::Database(_))));
    }

    // ── close_session ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_close_session_success() {
        let mut mock = MockCashierSessionRepo::new();
        mock.expect_close(|_, _| Ok(()));
        let svc = CashierSessionService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc
            .close_session(&ctx_with_all_permissions(), 1, &sample_close())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_close_session_not_found() {
        let mut mock = MockCashierSessionRepo::new();
        mock.expect_close(|_, _| Err(Error::NotFound("not found".to_string())));
        let svc = CashierSessionService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc
            .close_session(&ctx_with_all_permissions(), 999, &sample_close())
            .await;
        assert!(matches!(result, Err(Error::NotFound(_))));
    }

    #[tokio::test]
    async fn test_close_session_forbidden() {
        let mock = MockCashierSessionRepo::new();
        let svc = CashierSessionService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc
            .close_session(&ctx_no_permissions(), 1, &sample_close())
            .await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    // ── get_by_id ────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_by_id_found() {
        let mut mock = MockCashierSessionRepo::new();
        mock.expect_get_by_id(|_| Ok(Some(sample_session())));
        let svc = CashierSessionService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc.get_by_id(&ctx_with_all_permissions(), 1).await.unwrap();
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_get_by_id_not_found() {
        let mut mock = MockCashierSessionRepo::new();
        mock.expect_get_by_id(|_| Ok(None));
        let svc = CashierSessionService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc
            .get_by_id(&ctx_with_all_permissions(), 999)
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_by_id_forbidden() {
        let mock = MockCashierSessionRepo::new();
        let svc = CashierSessionService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc.get_by_id(&ctx_no_permissions(), 1).await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    // ── get_current_session ──────────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_current_session_found() {
        let mut mock = MockCashierSessionRepo::new();
        mock.expect_get_open_by_user(|_, _| Ok(Some(sample_session())));
        let svc = CashierSessionService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc
            .get_current_session(&ctx_with_all_permissions(), 10, 20)
            .await
            .unwrap();
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_get_current_session_none() {
        let mut mock = MockCashierSessionRepo::new();
        mock.expect_get_open_by_user(|_, _| Ok(None));
        let svc = CashierSessionService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc
            .get_current_session(&ctx_with_all_permissions(), 10, 20)
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_current_session_forbidden() {
        let mock = MockCashierSessionRepo::new();
        let svc = CashierSessionService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc.get_current_session(&ctx_no_permissions(), 10, 20).await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    // ── get_all ──────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_all_success() {
        let mut mock = MockCashierSessionRepo::new();
        mock.expect_get_all(|_| {
            Ok(CashierSessionPage {
                items: vec![sample_session()],
                next_cursor: None,
            })
        });
        let svc = CashierSessionService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc
            .get_all(&ctx_with_all_permissions(), &sample_query())
            .await
            .unwrap();
        assert_eq!(result.items.len(), 1);
    }

    #[tokio::test]
    async fn test_get_all_forbidden() {
        let mock = MockCashierSessionRepo::new();
        let svc = CashierSessionService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc.get_all(&ctx_no_permissions(), &sample_query()).await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_get_all_repo_error() {
        let mut mock = MockCashierSessionRepo::new();
        mock.expect_get_all(|_| Err(Error::Database("db error".to_string())));
        let svc = CashierSessionService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc
            .get_all(&ctx_with_all_permissions(), &sample_query())
            .await;
        assert!(matches!(result, Err(Error::Database(_))));
    }

    // ── delete ────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_delete_success() {
        let mut mock = MockCashierSessionRepo::new();
        mock.expect_delete(|_| Ok(()));
        let svc = CashierSessionService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc.delete(&ctx_with_all_permissions(), 1).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_not_found() {
        let mut mock = MockCashierSessionRepo::new();
        mock.expect_delete(|_| Err(Error::NotFound("not found".to_string())));
        let svc = CashierSessionService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc.delete(&ctx_with_all_permissions(), 999).await;
        assert!(matches!(result, Err(Error::NotFound(_))));
    }

    #[tokio::test]
    async fn test_delete_forbidden() {
        let mock = MockCashierSessionRepo::new();
        let svc = CashierSessionService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc.delete(&ctx_no_permissions(), 1).await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }
}
