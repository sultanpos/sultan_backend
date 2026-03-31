use async_trait::async_trait;
use sea_orm::DatabaseConnection;

use crate::{
    application::ServiceDbHelper,
    domain::{
        Context, DomainResult,
        model::{
            machine::{Machine, MachineCreate, MachinePage, MachineQuery, MachineUpdate},
            permission::{action, resource},
        },
    },
    snowflake::IdGenerator,
    storage::MachineRepository,
};

#[async_trait]
pub trait MachineServiceTrait: Send + Sync {
    async fn create(&self, ctx: &Context, machine: &MachineCreate) -> DomainResult<i64>;
    async fn update(&self, ctx: &Context, id: i64, machine: &MachineUpdate) -> DomainResult<()>;
    async fn delete(&self, ctx: &Context, id: i64) -> DomainResult<()>;
    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<Machine>>;
    async fn get_all(&self, ctx: &Context, query: &MachineQuery) -> DomainResult<MachinePage>;
}

pub struct MachineService<R, I> {
    repository: R,
    id_generator: I,
    db: DatabaseConnection,
}

impl<R, I> MachineService<R, I>
where
    R: MachineRepository,
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

impl<R, I> ServiceDbHelper for MachineService<R, I>
where
    R: MachineRepository,
    I: IdGenerator,
{
    fn database(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl<R, I> MachineServiceTrait for MachineService<R, I>
where
    R: MachineRepository,
    I: IdGenerator,
{
    async fn create(&self, ctx: &Context, machine: &MachineCreate) -> DomainResult<i64> {
        ctx.require_access(None, resource::MACHINE, action::CREATE)?;
        let id = self.id_generator.generate()?;
        let repo_ctx = self.repo_ctx(ctx);
        self.repository.create(&repo_ctx, id, machine).await?;
        Ok(id)
    }

    async fn update(&self, ctx: &Context, id: i64, machine: &MachineUpdate) -> DomainResult<()> {
        ctx.require_access(None, resource::MACHINE, action::UPDATE)?;
        let repo_ctx = self.repo_ctx(ctx);
        self.repository.update(&repo_ctx, id, machine).await
    }

    async fn delete(&self, ctx: &Context, id: i64) -> DomainResult<()> {
        ctx.require_access(None, resource::MACHINE, action::DELETE)?;
        let repo_ctx = self.repo_ctx(ctx);
        self.repository.delete(&repo_ctx, id).await
    }

    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<Machine>> {
        ctx.require_access(None, resource::MACHINE, action::READ)?;
        let repo_ctx = self.repo_ctx(ctx);
        self.repository.get_by_id(&repo_ctx, id).await
    }

    async fn get_all(&self, ctx: &Context, query: &MachineQuery) -> DomainResult<MachinePage> {
        ctx.require_access(None, resource::MACHINE, action::READ)?;
        let repo_ctx = self.repo_ctx(ctx);
        self.repository.get_all(&repo_ctx, query).await
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
    use crate::domain::model::Update;
    use crate::domain::model::machine::{MachineCursor, MachineFilter, MachineSortField};
    use crate::domain::model::product::SortDirection;
    use crate::storage::RepoCtx;
    use async_trait::async_trait;
    use chrono::Utc;
    use sea_orm::{ConnectionTrait, Database, DatabaseConnection};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // ── Manual mock ──────────────────────────────────────────────────────────

    #[derive(Clone)]
    struct MockMachineRepo {
        create_fn: Arc<Mutex<Option<Box<dyn Fn(i64, MachineCreate) -> DomainResult<()> + Send>>>>,
        update_fn: Arc<Mutex<Option<Box<dyn Fn(i64, MachineUpdate) -> DomainResult<()> + Send>>>>,
        delete_fn: Arc<Mutex<Option<Box<dyn Fn(i64) -> DomainResult<()> + Send>>>>,
        get_by_id_fn: Arc<Mutex<Option<Box<dyn Fn(i64) -> DomainResult<Option<Machine>> + Send>>>>,
        get_all_fn:
            Arc<Mutex<Option<Box<dyn Fn(MachineQuery) -> DomainResult<MachinePage> + Send>>>>,
    }

    impl MockMachineRepo {
        fn new() -> Self {
            Self {
                create_fn: Arc::new(Mutex::new(None)),
                update_fn: Arc::new(Mutex::new(None)),
                delete_fn: Arc::new(Mutex::new(None)),
                get_by_id_fn: Arc::new(Mutex::new(None)),
                get_all_fn: Arc::new(Mutex::new(None)),
            }
        }

        fn expect_create<F>(&mut self, f: F)
        where
            F: Fn(i64, MachineCreate) -> DomainResult<()> + Send + 'static,
        {
            *self.create_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_update<F>(&mut self, f: F)
        where
            F: Fn(i64, MachineUpdate) -> DomainResult<()> + Send + 'static,
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
            F: Fn(i64) -> DomainResult<Option<Machine>> + Send + 'static,
        {
            *self.get_by_id_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_get_all<F>(&mut self, f: F)
        where
            F: Fn(MachineQuery) -> DomainResult<MachinePage> + Send + 'static,
        {
            *self.get_all_fn.lock().unwrap() = Some(Box::new(f));
        }
    }

    #[async_trait]
    impl MachineRepository for MockMachineRepo {
        async fn create(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
            machine: &MachineCreate,
        ) -> DomainResult<()> {
            let func = self.create_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(id, machine.clone())
            } else {
                panic!("create not mocked")
            }
        }

        async fn update(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
            machine: &MachineUpdate,
        ) -> DomainResult<()> {
            let func = self.update_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(id, machine.clone())
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
        ) -> DomainResult<Option<Machine>> {
            let func = self.get_by_id_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(id)
            } else {
                panic!("get_by_id not mocked")
            }
        }

        async fn get_all(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            query: &MachineQuery,
        ) -> DomainResult<MachinePage> {
            let func = self.get_all_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(query.clone())
            } else {
                panic!("get_all not mocked")
            }
        }
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    async fn create_test_db() -> DatabaseConnection {
        Database::connect("sqlite::memory:").await.unwrap()
    }

    fn ctx_with_all_permissions() -> Context {
        let mut permissions = HashMap::new();
        permissions.insert((resource::MACHINE, None), 0b1111);
        Context::new_with_all(None, permissions, HashMap::new())
    }

    fn ctx_no_permissions() -> Context {
        Context::new()
    }

    fn sample_machine_create() -> MachineCreate {
        MachineCreate {
            branch_id: 1,
            key: "POS-01".to_string(),
            name: "Counter 1".to_string(),
            description: Some("Main counter".to_string()),
            metadata: None,
        }
    }

    fn sample_machine() -> Machine {
        Machine {
            id: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            is_deleted: false,
            branch_id: 1,
            key: "POS-01".to_string(),
            name: "Counter 1".to_string(),
            description: Some("Main counter".to_string()),
            metadata: None,
        }
    }

    fn sample_machine_update() -> MachineUpdate {
        MachineUpdate {
            name: Some("Counter 1 Updated".to_string()),
            description: Update::Unchanged,
            metadata: Update::Unchanged,
        }
    }

    fn default_query() -> MachineQuery {
        MachineQuery {
            filter: MachineFilter {
                branch_id: None,
                name: None,
            },
            sort_field: MachineSortField::Name,
            sort_direction: SortDirection::Asc,
            cursor: None,
            limit: 20,
        }
    }

    fn empty_page() -> MachinePage {
        MachinePage {
            items: vec![],
            next_cursor: None,
        }
    }

    // ── create ────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_create_success() {
        let mut mock = MockMachineRepo::new();
        mock.expect_create(|id, m| {
            assert_eq!(id, 1);
            assert_eq!(m.key, "POS-01");
            Ok(())
        });
        let svc = MachineService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc
            .create(&ctx_with_all_permissions(), &sample_machine_create())
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_create_no_permission() {
        let svc = MachineService::new(
            MockMachineRepo::new(),
            create_mock_id_gen(1),
            create_test_db().await,
        );
        let result = svc
            .create(&ctx_no_permissions(), &sample_machine_create())
            .await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_create_conflict() {
        let mut mock = MockMachineRepo::new();
        mock.expect_create(|_, _| {
            Err(Error::Conflict(
                "Machine with key 'POS-01' already exists in this branch".to_string(),
            ))
        });
        let svc = MachineService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc
            .create(&ctx_with_all_permissions(), &sample_machine_create())
            .await;
        assert!(matches!(result, Err(Error::Conflict(_))));
    }

    #[tokio::test]
    async fn test_create_db_error() {
        let mut mock = MockMachineRepo::new();
        mock.expect_create(|_, _| Err(Error::Database("db down".to_string())));
        let svc = MachineService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc
            .create(&ctx_with_all_permissions(), &sample_machine_create())
            .await;
        assert!(matches!(result, Err(Error::Database(_))));
    }

    // ── update ────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_update_success() {
        let mut mock = MockMachineRepo::new();
        mock.expect_update(|id, u| {
            assert_eq!(id, 42);
            assert_eq!(u.name, Some("Counter 1 Updated".to_string()));
            Ok(())
        });
        let svc = MachineService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc
            .update(&ctx_with_all_permissions(), 42, &sample_machine_update())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_no_permission() {
        let svc = MachineService::new(
            MockMachineRepo::new(),
            create_mock_id_gen(1),
            create_test_db().await,
        );
        let result = svc
            .update(&ctx_no_permissions(), 1, &sample_machine_update())
            .await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_update_not_found() {
        let mut mock = MockMachineRepo::new();
        mock.expect_update(|_, _| Err(Error::NotFound("Machine not found".to_string())));
        let svc = MachineService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc
            .update(&ctx_with_all_permissions(), 999, &sample_machine_update())
            .await;
        assert!(matches!(result, Err(Error::NotFound(_))));
    }

    #[tokio::test]
    async fn test_update_db_error() {
        let mut mock = MockMachineRepo::new();
        mock.expect_update(|_, _| Err(Error::Database("db error".to_string())));
        let svc = MachineService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc
            .update(&ctx_with_all_permissions(), 1, &sample_machine_update())
            .await;
        assert!(matches!(result, Err(Error::Database(_))));
    }

    // ── delete ────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_delete_success() {
        let mut mock = MockMachineRepo::new();
        mock.expect_delete(|id| {
            assert_eq!(id, 7);
            Ok(())
        });
        let svc = MachineService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc.delete(&ctx_with_all_permissions(), 7).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_no_permission() {
        let svc = MachineService::new(
            MockMachineRepo::new(),
            create_mock_id_gen(1),
            create_test_db().await,
        );
        let result = svc.delete(&ctx_no_permissions(), 1).await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_delete_not_found() {
        let mut mock = MockMachineRepo::new();
        mock.expect_delete(|_| Err(Error::NotFound("Machine not found".to_string())));
        let svc = MachineService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc.delete(&ctx_with_all_permissions(), 999).await;
        assert!(matches!(result, Err(Error::NotFound(_))));
    }

    #[tokio::test]
    async fn test_delete_db_error() {
        let mut mock = MockMachineRepo::new();
        mock.expect_delete(|_| Err(Error::Database("db error".to_string())));
        let svc = MachineService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc.delete(&ctx_with_all_permissions(), 1).await;
        assert!(matches!(result, Err(Error::Database(_))));
    }

    // ── get_by_id ─────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_by_id_found() {
        let expected = sample_machine();
        let expected_clone = expected.clone();
        let mut mock = MockMachineRepo::new();
        mock.expect_get_by_id(move |id| {
            assert_eq!(id, 1);
            Ok(Some(expected_clone.clone()))
        });
        let svc = MachineService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc
            .get_by_id(&ctx_with_all_permissions(), 1)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(result.key, expected.key);
        assert_eq!(result.name, expected.name);
        assert_eq!(result.branch_id, expected.branch_id);
    }

    #[tokio::test]
    async fn test_get_by_id_not_found() {
        let mut mock = MockMachineRepo::new();
        mock.expect_get_by_id(|_| Ok(None));
        let svc = MachineService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc.get_by_id(&ctx_with_all_permissions(), 999).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_by_id_no_permission() {
        let svc = MachineService::new(
            MockMachineRepo::new(),
            create_mock_id_gen(1),
            create_test_db().await,
        );
        let result = svc.get_by_id(&ctx_no_permissions(), 1).await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_get_by_id_db_error() {
        let mut mock = MockMachineRepo::new();
        mock.expect_get_by_id(|_| Err(Error::Database("db error".to_string())));
        let svc = MachineService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc.get_by_id(&ctx_with_all_permissions(), 1).await;
        assert!(matches!(result, Err(Error::Database(_))));
    }

    // ── get_all ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_all_success() {
        let machines = vec![sample_machine()];
        let machines_clone = machines.clone();
        let mut mock = MockMachineRepo::new();
        mock.expect_get_all(move |_| {
            Ok(MachinePage {
                items: machines_clone.clone(),
                next_cursor: None,
            })
        });
        let svc = MachineService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc
            .get_all(&ctx_with_all_permissions(), &default_query())
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().items.len(), 1);
    }

    #[tokio::test]
    async fn test_get_all_empty() {
        let mut mock = MockMachineRepo::new();
        mock.expect_get_all(|_| Ok(empty_page()));
        let svc = MachineService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc
            .get_all(&ctx_with_all_permissions(), &default_query())
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().items.len(), 0);
    }

    #[tokio::test]
    async fn test_get_all_no_permission() {
        let svc = MachineService::new(
            MockMachineRepo::new(),
            create_mock_id_gen(1),
            create_test_db().await,
        );
        let result = svc.get_all(&ctx_no_permissions(), &default_query()).await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_get_all_with_next_cursor() {
        let cursor = MachineCursor {
            field_value: "Counter 1".to_string(),
            id: 1,
        };
        let cursor_clone = cursor.clone();
        let mut mock = MockMachineRepo::new();
        mock.expect_get_all(move |_| {
            Ok(MachinePage {
                items: vec![sample_machine()],
                next_cursor: Some(cursor_clone.clone()),
            })
        });
        let svc = MachineService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc
            .get_all(&ctx_with_all_permissions(), &default_query())
            .await
            .unwrap();
        assert!(result.next_cursor.is_some());
        assert_eq!(result.next_cursor.unwrap().field_value, cursor.field_value);
    }

    #[tokio::test]
    async fn test_get_all_passes_query_filter() {
        let mut mock = MockMachineRepo::new();
        mock.expect_get_all(|query| {
            assert_eq!(query.filter.branch_id, Some(5));
            assert_eq!(query.filter.name, Some("Counter".to_string()));
            assert_eq!(query.limit, 10);
            Ok(empty_page())
        });
        let svc = MachineService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let query = MachineQuery {
            filter: MachineFilter {
                branch_id: Some(5),
                name: Some("Counter".to_string()),
            },
            sort_field: MachineSortField::Name,
            sort_direction: SortDirection::Asc,
            cursor: None,
            limit: 10,
        };
        let result = svc.get_all(&ctx_with_all_permissions(), &query).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_all_db_error() {
        let mut mock = MockMachineRepo::new();
        mock.expect_get_all(|_| Err(Error::Database("db error".to_string())));
        let svc = MachineService::new(mock, create_mock_id_gen(1), create_test_db().await);
        let result = svc
            .get_all(&ctx_with_all_permissions(), &default_query())
            .await;
        assert!(matches!(result, Err(Error::Database(_))));
    }
}
