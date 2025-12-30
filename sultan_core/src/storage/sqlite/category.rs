use std::collections::HashMap;

use async_trait::async_trait;
use serde::Serialize;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::{
    domain::{
        Context, DomainResult, Error,
        model::category::{Category, CategoryCreate, CategoryUpdate},
    },
    storage::CategoryRepository,
};

#[derive(Clone)]
pub struct SqliteCategoryRepository {
    pool: SqlitePool,
}

impl SqliteCategoryRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Maximum allowed depth for category nesting (1-indexed, so 5 means 5 levels)
    const MAX_DEPTH: i32 = 5;

    /// Calculate the depth of a category by traversing up the parent chain.
    /// Returns the depth (1 for root categories, 2 for their children, etc.)
    async fn get_category_depth(&self, category_id: i64) -> DomainResult<i32> {
        // Use a recursive CTE to count the depth
        let query = sqlx::query_scalar::<_, i32>(
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
            SELECT MAX(depth) FROM category_path
            "#,
        )
        .bind(category_id)
        .fetch_one(&self.pool);

        let depth = query.await?;
        Ok(depth)
    }

    /// Get the maximum depth of children under a category.
    /// Returns 0 if the category has no children.
    async fn get_max_child_depth(&self, category_id: i64) -> DomainResult<i32> {
        // Use a recursive CTE to find the maximum depth of descendants
        let query = sqlx::query_scalar::<_, i32>(
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
            SELECT COALESCE(MAX(depth), 0) FROM category_children
            "#,
        )
        .bind(category_id)
        .fetch_one(&self.pool);

        let depth = query.await?;
        Ok(depth)
    }

    /// Convert CategoryDbSqlite to Category domain model
    fn to_category(c: &CategoryDbSqlite) -> Category {
        Category {
            id: c.id,
            created_at: super::parse_sqlite_date(&c.created_at),
            updated_at: super::parse_sqlite_date(&c.updated_at),
            deleted_at: c.deleted_at.as_ref().map(|d| super::parse_sqlite_date(d)),
            is_deleted: c.is_deleted,
            name: c.name.clone(),
            description: c.description.clone(),
            children: Some(Vec::new()),
        }
    }

    /// Build maps needed for tree construction from a list of categories
    fn build_tree_maps(
        categories: &[CategoryDbSqlite],
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
    fn build_category_tree(categories: Vec<CategoryDbSqlite>) -> Vec<Category> {
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
    async fn get_category_with_children(&self, category_id: i64) -> DomainResult<Option<Category>> {
        // Fetch the category and all its descendants using recursive CTE
        let query = sqlx::query_as::<_, CategoryDbSqlite>(
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
        )
        .bind(category_id)
        .fetch_all(&self.pool);

        let categories = query.await?;

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

#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct CategoryDbSqlite {
    pub id: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub is_deleted: bool,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<i64>,
}

impl From<CategoryDbSqlite> for Category {
    fn from(category_db: CategoryDbSqlite) -> Self {
        Category {
            id: category_db.id,
            created_at: super::parse_sqlite_date(&category_db.created_at),
            updated_at: super::parse_sqlite_date(&category_db.updated_at),
            deleted_at: category_db.deleted_at.map(|d| super::parse_sqlite_date(&d)),
            is_deleted: category_db.is_deleted,
            name: category_db.name,
            description: category_db.description,
            children: None, // Children can be populated later if needed
        }
    }
}

#[async_trait]
impl CategoryRepository for SqliteCategoryRepository {
    async fn create(&self, _: &Context, id: i64, category: &CategoryCreate) -> DomainResult<()> {
        // Check depth limit if parent_id is provided
        if let Some(pid) = category.parent_id {
            let parent_depth = self.get_category_depth(pid).await?;
            if parent_depth >= Self::MAX_DEPTH {
                return Err(Error::Database(format!(
                    "Cannot create category: maximum nesting depth of {} exceeded",
                    Self::MAX_DEPTH
                )));
            }
        }

        let query = sqlx::query(
            r#"
            INSERT INTO categories (id, name, description, parent_id)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(&category.name)
        .bind(&category.description)
        .bind(category.parent_id)
        .execute(&self.pool);

        query.await?;
        Ok(())
    }

    async fn update(&self, _: &Context, id: i64, category: &CategoryUpdate) -> DomainResult<()> {
        // Check depth limit if parent_id is provided
        if category.parent_id.should_update()
            && let Some(pid) = category.parent_id.as_value()
        {
            // First, get the depth of children under this category
            let max_child_depth = self.get_max_child_depth(id).await?;
            // Get the depth of the new parent
            let new_parent_depth = self.get_category_depth(*pid).await?;
            // Total depth would be: new_parent_depth + 1 (this category) + max_child_depth
            let total_depth = new_parent_depth + 1 + max_child_depth;
            if total_depth > Self::MAX_DEPTH {
                return Err(Error::Database(format!(
                    "Cannot move category: maximum nesting depth of {} would be exceeded",
                    Self::MAX_DEPTH
                )));
            }
        }

        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new("UPDATE categories SET ");
        let mut separated = builder.separated(", ");

        if let Some(name) = &category.name {
            separated.push("name = ").push_bind_unseparated(name);
        }
        if category.description.should_update() {
            separated
                .push("description = ")
                .push_bind_unseparated(category.description.to_bind_value());
        }
        if category.parent_id.should_update() {
            separated
                .push("parent_id = ")
                .push_bind_unseparated(category.parent_id.to_bind_value());
        }
        separated.push("updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')");
        builder.push(" WHERE id = ").push_bind(id);
        builder.push(" AND is_deleted = 0");

        let query = builder.build();
        let result = query.execute(&self.pool).await?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound(format!(
                "Category with id {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn delete(&self, _: &Context, id: i64) -> DomainResult<()> {
        let query = sqlx::query(
            r#"
            UPDATE categories SET
                is_deleted = 1,
                deleted_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ? AND is_deleted = 0
            "#,
        )
        .bind(id)
        .execute(&self.pool);

        let result = query.await?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound(format!(
                "Category with id {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn get_all(&self, _: &Context) -> DomainResult<Vec<Category>> {
        let query = sqlx::query_as::<_, CategoryDbSqlite>(
            r#"
            SELECT id, created_at, updated_at, deleted_at, is_deleted, name, description, parent_id
            FROM categories WHERE is_deleted = 0
            "#,
        )
        .fetch_all(&self.pool);

        let categories = query.await?;

        // Build tree structure with children populated
        Ok(Self::build_category_tree(categories))
    }

    async fn get_by_id(&self, _: &Context, id: i64) -> DomainResult<Option<Category>> {
        self.get_category_with_children(id).await
    }
}
