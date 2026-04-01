use async_trait::async_trait;
use chrono::Utc;
use sultan_core::application::CashierSessionServiceTrait;
use sultan_core::domain::{
    DomainResult, Error,
    context::Context,
    model::cashier_session::{
        CashierSession, CashierSessionClose, CashierSessionCreate, CashierSessionCursor,
        CashierSessionFilter, CashierSessionPage, CashierSessionQuery, CashierSessionSortField,
        SessionStatus,
    },
    model::product::SortDirection,
};

pub struct MockCashierSessionService {
    pub should_succeed: bool,
    pub id: i64,
}

impl MockCashierSessionService {
    pub fn new_success() -> Self {
        Self {
            should_succeed: true,
            id: 1,
        }
    }

    #[allow(dead_code)]
    pub fn new_failure() -> Self {
        Self {
            should_succeed: false,
            id: 1,
        }
    }
}

fn sample_session(id: i64) -> CashierSession {
    CashierSession {
        id,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
        is_deleted: false,
        branch_id: 1,
        user_id: 1,
        opened_at: Utc::now(),
        closed_at: None,
        status: SessionStatus::Open,
        opening_cash: 100_000,
        closing_cash: None,
        notes: None,
    }
}

#[async_trait]
impl CashierSessionServiceTrait for MockCashierSessionService {
    async fn open_session(
        &self,
        _ctx: &Context,
        _data: &CashierSessionCreate,
    ) -> DomainResult<i64> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to open session".to_string()));
        }
        Ok(self.id)
    }

    async fn close_session(
        &self,
        _ctx: &Context,
        id: i64,
        _data: &CashierSessionClose,
    ) -> DomainResult<()> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to close session".to_string()));
        }
        if id != 1 {
            return Err(Error::NotFound(format!(
                "Cashier session with id {} not found",
                id
            )));
        }
        Ok(())
    }

    async fn get_by_id(&self, _ctx: &Context, id: i64) -> DomainResult<Option<CashierSession>> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to get session".to_string()));
        }
        if id == 1 {
            Ok(Some(sample_session(1)))
        } else {
            Ok(None)
        }
    }

    async fn get_current_session(
        &self,
        _ctx: &Context,
        _branch_id: i64,
        _user_id: i64,
    ) -> DomainResult<Option<CashierSession>> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to get current session".to_string()));
        }
        Ok(Some(sample_session(1)))
    }

    async fn get_all(
        &self,
        _ctx: &Context,
        _query: &CashierSessionQuery,
    ) -> DomainResult<CashierSessionPage> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to list sessions".to_string()));
        }
        Ok(CashierSessionPage {
            items: vec![sample_session(1), sample_session(2)],
            next_cursor: None,
        })
    }

    async fn delete(&self, _ctx: &Context, id: i64) -> DomainResult<()> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to delete session".to_string()));
        }
        if id != 1 {
            return Err(Error::NotFound(format!(
                "Cashier session with id {} not found",
                id
            )));
        }
        Ok(())
    }
}

/// Default mock query for tests
#[allow(dead_code)]
pub fn default_cashier_session_query() -> CashierSessionQuery {
    CashierSessionQuery {
        filter: CashierSessionFilter {
            branch_id: None,
            user_id: None,
            status: None,
        },
        sort_field: CashierSessionSortField::OpenedAt,
        sort_direction: SortDirection::Desc,
        cursor: None,
        limit: 20,
    }
}

/// Sample cursor for pagination tests
#[allow(dead_code)]
pub fn sample_cursor() -> CashierSessionCursor {
    CashierSessionCursor {
        field_value: "2026-01-01T00:00:00.000Z".to_string(),
        id: 1,
    }
}
