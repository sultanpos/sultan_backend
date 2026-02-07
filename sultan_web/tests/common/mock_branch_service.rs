use async_trait::async_trait;
use sultan_core::application::BranchServiceTrait;
use sultan_core::domain::{
    DomainResult, Error,
    context::Context,
    model::branch::{Branch, BranchCreate, BranchUpdate},
};

pub struct MockBranchService {
    pub should_succeed: bool,
    pub id: i64,
}

impl MockBranchService {
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

#[async_trait]
impl BranchServiceTrait for MockBranchService {
    async fn create(&self, _ctx: &Context, _branch: &BranchCreate) -> DomainResult<i64> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to create branch".to_string()));
        }
        Ok(self.id)
    }

    async fn update(
        &self,
        _ctx: &Context,
        id: i64,
        _branch: &BranchUpdate,
    ) -> DomainResult<()> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to update branch".to_string()));
        }
        if id != 1 {
            return Err(Error::NotFound(format!(
                "Branch with id {} not found",
                id
            )));
        }
        Ok(())
    }

    async fn delete(&self, _ctx: &Context, id: i64) -> DomainResult<()> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to delete branch".to_string()));
        }
        if id != 1 {
            return Err(Error::NotFound(format!(
                "Branch with id {} not found",
                id
            )));
        }
        Ok(())
    }

    async fn get_by_id(&self, _ctx: &Context, id: i64) -> DomainResult<Option<Branch>> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to get branch".to_string()));
        }

        if id == 1 {
            Ok(Some(Branch {
                id: 1,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                deleted_at: None,
                is_deleted: false,
                is_main: true,
                name: "Sultan".to_string(),
                code: "SULTAN".to_string(),
                address: None,
                phone: None,
                npwp: None,
                image: None,
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_all(&self, _ctx: &Context) -> DomainResult<Vec<Branch>> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to get branches".to_string()));
        }

        Ok(vec![Branch {
            id: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
            is_deleted: false,
            is_main: true,
            name: "Sultan".to_string(),
            code: "SULTAN".to_string(),
            address: None,
            phone: None,
            npwp: None,
            image: None,
        }])
    }
}
