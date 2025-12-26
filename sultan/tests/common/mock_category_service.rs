use async_trait::async_trait;
use sultan_core::application::CategoryServiceTrait;
use sultan_core::domain::{
    DomainResult, Error,
    context::BranchContext,
    model::category::{Category, CategoryCreate, CategoryUpdate},
};

pub struct MockCategoryService {
    pub should_succeed: bool,
}

impl MockCategoryService {
    pub fn new_success() -> Self {
        Self {
            should_succeed: true,
        }
    }

    #[allow(dead_code)]
    pub fn new_failure() -> Self {
        Self {
            should_succeed: false,
        }
    }
}

#[async_trait]
impl CategoryServiceTrait<BranchContext> for MockCategoryService {
    async fn create(&self, _ctx: &BranchContext, _category: &CategoryCreate) -> DomainResult<()> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to create category".to_string()));
        }
        Ok(())
    }

    async fn update(
        &self,
        _ctx: &BranchContext,
        _id: i64,
        _category: &CategoryUpdate,
    ) -> DomainResult<()> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to update category".to_string()));
        }
        Ok(())
    }

    async fn delete(&self, _ctx: &BranchContext, _id: i64) -> DomainResult<()> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to delete category".to_string()));
        }
        Ok(())
    }

    async fn get_all(&self, _ctx: &BranchContext) -> DomainResult<Vec<Category>> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to get categories".to_string()));
        }

        // Return mock categories
        Ok(vec![
            Category {
                id: 1,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                deleted_at: None,
                is_deleted: false,
                name: "Electronics".to_string(),
                description: Some("Electronic devices".to_string()),
                children: None,
            },
            Category {
                id: 2,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                deleted_at: None,
                is_deleted: false,
                name: "Books".to_string(),
                description: Some("Books and magazines".to_string()),
                children: None,
            },
        ])
    }

    async fn get_by_id(&self, _ctx: &BranchContext, id: i64) -> DomainResult<Option<Category>> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to get category".to_string()));
        }

        // Return mock category if id is 1
        if id == 1 {
            Ok(Some(Category {
                id: 1,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                deleted_at: None,
                is_deleted: false,
                name: "Electronics".to_string(),
                description: Some("Electronic devices".to_string()),
                children: None,
            }))
        } else {
            Ok(None)
        }
    }
}
