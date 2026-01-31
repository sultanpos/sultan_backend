use std::collections::HashMap;

use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};

use crate::{
    domain::{
        DomainResult, Error,
        model::category::{Category, CategoryCreate, CategoryUpdate},
    },
    storage::{CategoryRepository, RepoCtx},
};

use super::entity::{CategoryActiveModel, CategoryColumn, CategoryEntity, CategoryModel};

/// SQLite implementation of CategoryRepository using SeaORM.
///
/// This repository uses SeaORM's `ConnectionTrait` which allows it to work
/// with both direct database connections and transactions seamlessly.
///
/// Categories support hierarchical structure with a maximum depth limit of 5 levels.
///
/// # Example
///
/// ```rust,ignore
/// // Using with direct connection
/// let repo = SqliteCategoryRepository::new();
/// let ctx = RepoCtx { ctx: Context::new(), db: &db_connection };
/// repo.create(&ctx, id, &category).await?;
///
/// // Using within a transaction
/// let txn = db.begin().await?;
/// let ctx = RepoCtx { ctx: Context::new(), db: &txn };
/// repo.create(&ctx, id, &category).await?;
/// txn.commit().await?;
/// ```
#[derive(Clone, Default)]
#[allow(dead_code)] // db field is used via Clone trait in services
pub struct SqliteCategoryRepository {}

impl SqliteCategoryRepository {
    pub fn new() -> Self {
        Self {}
    }

    /// Maximum allowed depth for category nesting (1-indexed, so 5 means 5 levels)
    const MAX_DEPTH: i32 = 5;

    /// Calculate the depth of a category by traversing up the parent chain.
    /// Returns the depth (1 for root categories, 2 for their children, etc.)
    async fn get_category_depth(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        category_id: i64,
    ) -> DomainResult<i32> {
        use sea_orm::FromQueryResult;

        // Use a recursive CTE to count the depth
        let query = sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            r#"
            WITH RECURSIVE category_path AS (
                SELECT id, parent_id, 1 as depth
                FROM categories
                WHERE id = ? AND is_deleted = 0
                
                UNION ALL
                
                SELECT c.id, c.parent_id, cp.depth + 1
                FROM categories c
                INNER JOIN category_path cp ON c.id = cp.parent_id
                WHERE c.is_deleted = 0
            )
            SELECT MAX(depth) as max_depth FROM category_path
            "#,
            vec![category_id.into()],
        );

        #[derive(FromQueryResult)]
        struct DepthResult {
            max_depth: Option<i32>,
        }

        let result = DepthResult::find_by_statement(query).one(&ctx.db).await?;

        match result {
            Some(r) => r.max_depth.ok_or_else(|| {
                Error::NotFound(format!("Parent category with id {} not found", category_id))
            }),
            None => Err(Error::NotFound(format!(
                "Parent category with id {} not found",
                category_id
            ))),
        }
    }

    /// Get the maximum depth of children under a category.
    /// Returns 0 if the category has no children.
    async fn get_max_child_depth(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        category_id: i64,
    ) -> DomainResult<i32> {
        use sea_orm::FromQueryResult;

        // Use a recursive CTE to find the maximum depth of descendants
        let query = sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            r#"
            WITH RECURSIVE category_children AS (
                SELECT id, 0 as depth
                FROM categories
                WHERE id = ? AND is_deleted = 0
                
                UNION ALL
                
                SELECT c.id, cc.depth + 1
                FROM categories c
                INNER JOIN category_children cc ON c.parent_id = cc.id
                WHERE c.is_deleted = 0
            )
            SELECT COALESCE(MAX(depth), 0) as max_depth FROM category_children
            "#,
            vec![category_id.into()],
        );

        #[derive(FromQueryResult)]
        struct DepthResult {
            max_depth: i32,
        }

        let result = DepthResult::find_by_statement(query).one(&ctx.db).await?;

        Ok(result.map(|r| r.max_depth).unwrap_or(0))
    }

    /// Convert CategoryModel to Category domain model
    fn to_category(c: &CategoryModel) -> Category {
        c.to_domain()
    }

    /// Build maps needed for tree construction from a list of categories
    fn build_tree_maps(
        categories: &[CategoryModel],
    ) -> (HashMap<i64, Category>, HashMap<i64, Vec<i64>>) {
        let category_map: HashMap<i64, Category> = categories
            .iter()
            .map(|c| (c.id, Self::to_category(c)))
            .collect();

        let mut children_map: HashMap<i64, Vec<i64>> = HashMap::new();
        for c in categories {
            if let Some(parent_id) = c.parent_id {
                children_map.entry(parent_id).or_default().push(c.id);
            }
        }

        (category_map, children_map)
    }

    /// Recursively build a subtree starting from a given category id
    fn build_subtree(
        id: i64,
        category_map: &mut HashMap<i64, Category>,
        children_map: &HashMap<i64, Vec<i64>>,
    ) -> Option<Category> {
        let mut category = category_map.remove(&id)?;

        let child_ids = children_map.get(&id).cloned().unwrap_or_default();
        let children: Vec<Category> = child_ids
            .into_iter()
            .filter_map(|child_id| Self::build_subtree(child_id, category_map, children_map))
            .collect();

        category.children = if children.is_empty() {
            None
        } else {
            Some(children)
        };

        Some(category)
    }

    /// Build a tree structure from a flat list of categories.
    /// Returns only root categories (those with no parent) with their children populated.
    fn build_category_tree(categories: Vec<CategoryModel>) -> Vec<Category> {
        let root_ids: Vec<i64> = categories
            .iter()
            .filter(|c| c.parent_id.is_none())
            .map(|c| c.id)
            .collect();

        let (mut category_map, children_map) = Self::build_tree_maps(&categories);

        root_ids
            .into_iter()
            .filter_map(|id| Self::build_subtree(id, &mut category_map, &children_map))
            .collect()
    }

    /// Fetch all descendants of a category and build the subtree.
    async fn get_category_with_children(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        category_id: i64,
    ) -> DomainResult<Option<Category>> {
        // Fetch the category and all its descendants using recursive CTE
        let query = sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            r#"
            WITH RECURSIVE category_tree AS (
                SELECT id, created_at, updated_at, deleted_at, is_deleted, name, description, parent_id
                FROM categories
                WHERE id = ? AND is_deleted = 0
                
                UNION ALL
                
                SELECT c.id, c.created_at, c.updated_at, c.deleted_at, c.is_deleted, c.name, c.description, c.parent_id
                FROM categories c
                INNER JOIN category_tree ct ON c.parent_id = ct.id
                WHERE c.is_deleted = 0
            )
            SELECT * FROM category_tree
            "#,
            vec![category_id.into()],
        );

        let categories = CategoryEntity::find()
            .from_raw_sql(query)
            .all(&ctx.db)
            .await?;

        if categories.is_empty() {
            return Ok(None);
        }

        let (mut category_map, children_map) = Self::build_tree_maps(&categories);
        Ok(Self::build_subtree(
            category_id,
            &mut category_map,
            &children_map,
        ))
    }
}

#[async_trait]
impl CategoryRepository for SqliteCategoryRepository {
    async fn create(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        category: &CategoryCreate,
    ) -> DomainResult<()> {
        // Check depth limit if parent_id is provided
        if let Some(pid) = category.parent_id {
            let parent_depth = self.get_category_depth(ctx, pid).await?;
            if parent_depth >= Self::MAX_DEPTH {
                return Err(Error::Database(format!(
                    "Cannot create category: maximum nesting depth of {} exceeded",
                    Self::MAX_DEPTH
                )));
            }
        }

        let category_model = CategoryActiveModel {
            id: Set(id),
            name: Set(category.name.clone()),
            description: Set(category.description.clone()),
            parent_id: Set(category.parent_id),
            ..Default::default()
        };

        category_model.insert(&ctx.db).await?;
        Ok(())
    }

    async fn update(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        category: &CategoryUpdate,
    ) -> DomainResult<()> {
        // Check depth limit if parent_id is provided
        if category.parent_id.should_update()
            && let Some(pid) = category.parent_id.as_value()
        {
            // First, get the depth of children under this category
            let max_child_depth = self.get_max_child_depth(ctx, id).await?;
            // Get the depth of the new parent
            let new_parent_depth = self.get_category_depth(ctx, *pid).await?;
            // Total depth would be: new_parent_depth + 1 (this category) + max_child_depth
            let total_depth = new_parent_depth + 1 + max_child_depth;
            if total_depth > Self::MAX_DEPTH {
                return Err(Error::Database(format!(
                    "Cannot move category: maximum nesting depth of {} would be exceeded",
                    Self::MAX_DEPTH
                )));
            }
        }

        use sea_orm::{UpdateMany, sea_query::Expr};

        // Build update query with filters
        let mut update_query: UpdateMany<CategoryEntity> = CategoryEntity::update_many()
            .filter(CategoryColumn::Id.eq(id))
            .filter(CategoryColumn::IsDeleted.eq(false));

        // Update fields if provided
        if let Some(name) = &category.name {
            update_query = update_query.col_expr(CategoryColumn::Name, Expr::value(name.clone()));
        }

        if category.description.should_update() {
            update_query = update_query.col_expr(
                CategoryColumn::Description,
                Expr::value(category.description.to_bind_value()),
            );
        }

        if category.parent_id.should_update() {
            update_query = update_query.col_expr(
                CategoryColumn::ParentId,
                Expr::value(category.parent_id.to_bind_value()),
            );
        }

        // Always update the updated_at timestamp
        update_query = update_query.col_expr(
            CategoryColumn::UpdatedAt,
            Expr::value(
                chrono::Utc::now()
                    .format("%Y-%m-%dT%H:%M:%S%.fZ")
                    .to_string(),
            ),
        );

        // Execute the update
        let result = update_query.exec(&ctx.db).await?;

        // Check if any rows were affected
        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!(
                "Category with id {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn delete(&self, ctx: &RepoCtx<impl ConnectionTrait>, id: i64) -> DomainResult<()> {
        use sea_orm::sea_query::Expr;

        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.fZ")
            .to_string();

        // Soft delete: mark as deleted with a single UPDATE query
        let result = CategoryEntity::update_many()
            .filter(CategoryColumn::Id.eq(id))
            .filter(CategoryColumn::IsDeleted.eq(false))
            .col_expr(CategoryColumn::IsDeleted, Expr::value(true))
            .col_expr(CategoryColumn::DeletedAt, Expr::value(Some(now.clone())))
            .col_expr(CategoryColumn::UpdatedAt, Expr::value(now))
            .exec(&ctx.db)
            .await?;

        // Check if any rows were affected
        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!(
                "Category with id {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn get_all(&self, ctx: &RepoCtx<impl ConnectionTrait>) -> DomainResult<Vec<Category>> {
        let categories = CategoryEntity::find()
            .filter(CategoryColumn::IsDeleted.eq(false))
            .all(&ctx.db)
            .await?;

        // Build tree structure with children populated
        Ok(Self::build_category_tree(categories))
    }

    async fn get_by_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<Category>> {
        self.get_category_with_children(ctx, id).await
    }
}
