use crate::{
    domain::{
        Context,
        model::{
            Update,
            category::{CategoryCreate, CategoryUpdate},
        },
    },
    storage::CategoryRepository,
};

pub async fn create_sqlite_category_repo() -> (Context, impl CategoryRepository) {
    let pool = super::init_sqlite_pool().await;
    (
        Context::new(),
        crate::storage::sqlite::category::SqliteCategoryRepository::new(pool),
    )
}

// =============================================================================
// Basic CRUD Tests
// =============================================================================

pub async fn category_test_create<C: CategoryRepository>(ctx: &Context, repo: C) {
    let id = super::generate_test_id().await;
    let category = CategoryCreate {
        name: "Electronics".to_string(),
        description: Some("Electronic devices".to_string()),
        parent_id: None,
    };
    repo.create(ctx, id, &category)
        .await
        .expect("Failed to create category");

    let category = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get category")
        .expect("Category not found");

    assert_eq!(category.id, id);
    assert_eq!(category.name, "Electronics".to_string());
    assert_eq!(category.description, Some("Electronic devices".to_string()));
    assert!(!category.is_deleted);
}

pub async fn category_test_create_without_description<C: CategoryRepository>(
    ctx: &Context,
    repo: C,
) {
    let id = super::generate_test_id().await;
    let category = CategoryCreate {
        name: "Simple Category".to_string(),
        description: None,
        parent_id: None,
    };
    repo.create(ctx, id, &category)
        .await
        .expect("Failed to create category");

    let category = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get category")
        .expect("Category not found");

    assert_eq!(category.name, "Simple Category".to_string());
    assert_eq!(category.description, None);
}

pub async fn category_test_update_name<C: CategoryRepository>(ctx: &Context, repo: C) {
    let id = super::generate_test_id().await;
    let category = CategoryCreate {
        name: "Original Name".to_string(),
        description: Some("Original description".to_string()),
        parent_id: None,
    };
    repo.create(ctx, id, &category)
        .await
        .expect("Failed to create category");

    let update = CategoryUpdate {
        name: Some("Updated Name".to_string()),
        description: Update::Unchanged,
        parent_id: Update::Unchanged,
    };
    repo.update(ctx, id, &update)
        .await
        .expect("Failed to update category");

    let category = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get category")
        .expect("Category not found");

    assert_eq!(category.name, "Updated Name".to_string());
    // Description should remain unchanged
    assert_eq!(
        category.description,
        Some("Original description".to_string())
    );
}

pub async fn category_test_update_description<C: CategoryRepository>(ctx: &Context, repo: C) {
    let id = super::generate_test_id().await;
    let category = CategoryCreate {
        name: "Category".to_string(),
        description: Some("Old description".to_string()),
        parent_id: None,
    };
    repo.create(ctx, id, &category)
        .await
        .expect("Failed to create category");

    let update = CategoryUpdate {
        name: None,
        description: Update::Set("New description".to_string()),
        parent_id: Update::Unchanged,
    };
    repo.update(ctx, id, &update)
        .await
        .expect("Failed to update category");

    let category = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get category")
        .expect("Category not found");

    assert_eq!(category.name, "Category".to_string());
    assert_eq!(category.description, Some("New description".to_string()));
}

pub async fn category_test_update_non_existent<C: CategoryRepository>(ctx: &Context, repo: C) {
    let update = CategoryUpdate {
        name: Some("Name".to_string()),
        description: Update::Unchanged,
        parent_id: Update::Unchanged,
    };
    let result = repo.update(ctx, 999999, &update).await;

    assert!(matches!(result, Err(crate::domain::Error::NotFound(_))));
}

pub async fn category_test_delete<C: CategoryRepository>(ctx: &Context, repo: C) {
    let id = super::generate_test_id().await;
    let category = CategoryCreate {
        name: "To Delete".to_string(),
        description: None,
        parent_id: None,
    };
    repo.create(ctx, id, &category)
        .await
        .expect("Failed to create category");

    repo.delete(ctx, id)
        .await
        .expect("Failed to delete category");

    let category = repo.get_by_id(ctx, id).await.expect("Failed to query");
    assert!(category.is_none(), "Deleted category should not be found");
}

pub async fn category_test_delete_non_existent<C: CategoryRepository>(ctx: &Context, repo: C) {
    let result = repo.delete(ctx, 999999).await;

    assert!(matches!(result, Err(crate::domain::Error::NotFound(_))));
}

pub async fn category_test_get_by_id_not_found<C: CategoryRepository>(ctx: &Context, repo: C) {
    let result = repo
        .get_by_id(ctx, 999999)
        .await
        .expect("Query should succeed");

    assert!(result.is_none());
}

pub async fn category_test_get_all_empty<C: CategoryRepository>(ctx: &Context, repo: C) {
    let categories = repo.get_all(ctx).await.expect("Failed to get all");
    // Fresh database should have no categories
    assert!(categories.is_empty());
}

pub async fn category_test_get_all_multiple<C: CategoryRepository>(ctx: &Context, repo: C) {
    let id1 = super::generate_test_id().await;
    let id2 = super::generate_test_id().await;
    let id3 = super::generate_test_id().await;

    repo.create(
        ctx,
        id1,
        &CategoryCreate {
            name: "Category 1".to_string(),
            description: None,
            parent_id: None,
        },
    )
    .await
    .expect("Failed to create category 1");
    repo.create(
        ctx,
        id2,
        &CategoryCreate {
            name: "Category 2".to_string(),
            description: None,
            parent_id: None,
        },
    )
    .await
    .expect("Failed to create category 2");
    repo.create(
        ctx,
        id3,
        &CategoryCreate {
            name: "Category 3".to_string(),
            description: None,
            parent_id: None,
        },
    )
    .await
    .expect("Failed to create category 3");

    let categories = repo.get_all(ctx).await.expect("Failed to get all");
    assert_eq!(categories.len(), 3);
}

// =============================================================================
// Parent-Child Relationship Tests
// =============================================================================

pub async fn category_test_create_with_parent<C: CategoryRepository>(ctx: &Context, repo: C) {
    let parent_id = super::generate_test_id().await;
    let child_id = super::generate_test_id().await;

    repo.create(
        ctx,
        parent_id,
        &CategoryCreate {
            name: "Parent".to_string(),
            description: None,
            parent_id: None,
        },
    )
    .await
    .expect("Failed to create parent");

    repo.create(
        ctx,
        child_id,
        &CategoryCreate {
            name: "Child".to_string(),
            description: None,
            parent_id: Some(parent_id),
        },
    )
    .await
    .expect("Failed to create child");

    // Get parent with children
    let parent = repo
        .get_by_id(ctx, parent_id)
        .await
        .expect("Failed to get parent")
        .expect("Parent not found");

    assert!(parent.children.is_some());
    let children = parent.children.unwrap();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].id, child_id);
    assert_eq!(children[0].name, "Child".to_string());
}

pub async fn category_test_create_nested<C: CategoryRepository>(ctx: &Context, repo: C) {
    let level1 = super::generate_test_id().await;
    let level2 = super::generate_test_id().await;
    let level3 = super::generate_test_id().await;

    repo.create(
        ctx,
        level1,
        &CategoryCreate {
            name: "Level 1".to_string(),
            description: None,
            parent_id: None,
        },
    )
    .await
    .expect("Failed to create level 1");
    repo.create(
        ctx,
        level2,
        &CategoryCreate {
            name: "Level 2".to_string(),
            description: None,
            parent_id: Some(level1),
        },
    )
    .await
    .expect("Failed to create level 2");
    repo.create(
        ctx,
        level3,
        &CategoryCreate {
            name: "Level 3".to_string(),
            description: None,
            parent_id: Some(level2),
        },
    )
    .await
    .expect("Failed to create level 3");

    let root = repo
        .get_by_id(ctx, level1)
        .await
        .expect("Failed to get root")
        .expect("Root not found");

    // Verify nested structure
    assert!(root.children.is_some());
    let level2_cat = &root.children.as_ref().unwrap()[0];
    assert_eq!(level2_cat.name, "Level 2".to_string());

    assert!(level2_cat.children.is_some());
    let level3_cat = &level2_cat.children.as_ref().unwrap()[0];
    assert_eq!(level3_cat.name, "Level 3".to_string());
}

pub async fn category_test_get_all_tree_structure<C: CategoryRepository>(ctx: &Context, repo: C) {
    let root1 = super::generate_test_id().await;
    let root2 = super::generate_test_id().await;
    let child1 = super::generate_test_id().await;
    let child2 = super::generate_test_id().await;

    repo.create(
        ctx,
        root1,
        &CategoryCreate {
            name: "Root 1".to_string(),
            description: None,
            parent_id: None,
        },
    )
    .await
    .expect("Failed to create root 1");
    repo.create(
        ctx,
        root2,
        &CategoryCreate {
            name: "Root 2".to_string(),
            description: None,
            parent_id: None,
        },
    )
    .await
    .expect("Failed to create root 2");
    repo.create(
        ctx,
        child1,
        &CategoryCreate {
            name: "Child of Root 1".to_string(),
            description: None,
            parent_id: Some(root1),
        },
    )
    .await
    .expect("Failed to create child 1");
    repo.create(
        ctx,
        child2,
        &CategoryCreate {
            name: "Child of Root 2".to_string(),
            description: None,
            parent_id: Some(root2),
        },
    )
    .await
    .expect("Failed to create child 2");

    let categories = repo.get_all(ctx).await.expect("Failed to get all");

    // Should only return root categories
    assert_eq!(categories.len(), 2);

    // Each root should have its children populated
    for cat in &categories {
        assert!(cat.children.is_some());
        assert_eq!(cat.children.as_ref().unwrap().len(), 1);
    }
}

pub async fn category_test_update_parent<C: CategoryRepository>(ctx: &Context, repo: C) {
    let parent1 = super::generate_test_id().await;
    let parent2 = super::generate_test_id().await;
    let child = super::generate_test_id().await;

    repo.create(
        ctx,
        parent1,
        &CategoryCreate {
            name: "Parent 1".to_string(),
            description: None,
            parent_id: None,
        },
    )
    .await
    .expect("Failed to create parent 1");
    repo.create(
        ctx,
        parent2,
        &CategoryCreate {
            name: "Parent 2".to_string(),
            description: None,
            parent_id: None,
        },
    )
    .await
    .expect("Failed to create parent 2");
    repo.create(
        ctx,
        child,
        &CategoryCreate {
            name: "Child".to_string(),
            description: None,
            parent_id: Some(parent1),
        },
    )
    .await
    .expect("Failed to create child");

    // Move child to parent2
    let update = CategoryUpdate {
        name: None,
        description: Update::Unchanged,
        parent_id: Update::Set(parent2),
    };
    repo.update(ctx, child, &update)
        .await
        .expect("Failed to update child");

    // Verify child is now under parent2
    let parent2_cat = repo
        .get_by_id(ctx, parent2)
        .await
        .expect("Failed to get parent 2")
        .expect("Parent 2 not found");

    assert!(parent2_cat.children.is_some());
    assert_eq!(parent2_cat.children.as_ref().unwrap().len(), 1);
    assert_eq!(parent2_cat.children.as_ref().unwrap()[0].id, child);

    // Verify parent1 has no children
    let parent1_cat = repo
        .get_by_id(ctx, parent1)
        .await
        .expect("Failed to get parent 1")
        .expect("Parent 1 not found");

    assert!(parent1_cat.children.is_none());
}

pub async fn category_test_multiple_children<C: CategoryRepository>(ctx: &Context, repo: C) {
    let parent = super::generate_test_id().await;
    let child1 = super::generate_test_id().await;
    let child2 = super::generate_test_id().await;
    let child3 = super::generate_test_id().await;

    repo.create(
        ctx,
        parent,
        &CategoryCreate {
            name: "Parent".to_string(),
            description: None,
            parent_id: None,
        },
    )
    .await
    .expect("Failed to create parent");
    repo.create(
        ctx,
        child1,
        &CategoryCreate {
            name: "Child 1".to_string(),
            description: None,
            parent_id: Some(parent),
        },
    )
    .await
    .expect("Failed to create child 1");
    repo.create(
        ctx,
        child2,
        &CategoryCreate {
            name: "Child 2".to_string(),
            description: None,
            parent_id: Some(parent),
        },
    )
    .await
    .expect("Failed to create child 2");
    repo.create(
        ctx,
        child3,
        &CategoryCreate {
            name: "Child 3".to_string(),
            description: None,
            parent_id: Some(parent),
        },
    )
    .await
    .expect("Failed to create child 3");

    let parent_cat = repo
        .get_by_id(ctx, parent)
        .await
        .expect("Failed to get parent")
        .expect("Parent not found");

    assert!(parent_cat.children.is_some());
    assert_eq!(parent_cat.children.as_ref().unwrap().len(), 3);
}

// =============================================================================
// Depth Limit Tests (MAX_DEPTH = 5)
// =============================================================================

pub async fn category_test_create_at_max_depth<C: CategoryRepository>(ctx: &Context, repo: C) {
    // Create 5 levels (max allowed)
    let mut parent_id = None;
    let mut ids = Vec::new();

    for i in 1..=5 {
        let id = super::generate_test_id().await;
        ids.push(id);
        repo.create(
            ctx,
            id,
            &CategoryCreate {
                name: format!("Level {}", i),
                description: None,
                parent_id,
            },
        )
        .await
        .unwrap_or_else(|_| panic!("Failed to create level {}", i));
        parent_id = Some(id);
    }

    // All 5 levels should be created successfully
    for id in &ids {
        let cat = repo
            .get_by_id(ctx, *id)
            .await
            .expect("Failed to get category")
            .expect("Category not found");
        assert_eq!(
            cat.name,
            format!("Level {}", ids.iter().position(|&x| x == *id).unwrap() + 1)
        );
    }
}

pub async fn category_test_create_exceeds_max_depth<C: CategoryRepository>(ctx: &Context, repo: C) {
    // Create 5 levels (max allowed)
    let mut parent_id = None;

    for i in 1..=5 {
        let id = super::generate_test_id().await;
        repo.create(
            ctx,
            id,
            &CategoryCreate {
                name: format!("Level {}", i),
                description: None,
                parent_id,
            },
        )
        .await
        .unwrap_or_else(|_| panic!("Failed to create level {}", i));
        parent_id = Some(id);
    }

    // Try to create 6th level - should fail
    let id6 = super::generate_test_id().await;
    let result = repo
        .create(
            ctx,
            id6,
            &CategoryCreate {
                name: "Level 6".to_string(),
                description: None,
                parent_id,
            },
        )
        .await;

    assert!(result.is_err());
    assert!(matches!(result, Err(crate::domain::Error::Database(_))));
}

pub async fn category_test_move_exceeds_max_depth<C: CategoryRepository>(ctx: &Context, repo: C) {
    // Create chain of 4 levels
    let level1 = super::generate_test_id().await;
    let level2 = super::generate_test_id().await;
    let level3 = super::generate_test_id().await;
    let level4 = super::generate_test_id().await;

    repo.create(
        ctx,
        level1,
        &CategoryCreate {
            name: "Level 1".to_string(),
            description: None,
            parent_id: None,
        },
    )
    .await
    .expect("Failed to create level 1");
    repo.create(
        ctx,
        level2,
        &CategoryCreate {
            name: "Level 2".to_string(),
            description: None,
            parent_id: Some(level1),
        },
    )
    .await
    .expect("Failed to create level 2");
    repo.create(
        ctx,
        level3,
        &CategoryCreate {
            name: "Level 3".to_string(),
            description: None,
            parent_id: Some(level2),
        },
    )
    .await
    .expect("Failed to create level 3");
    repo.create(
        ctx,
        level4,
        &CategoryCreate {
            name: "Level 4".to_string(),
            description: None,
            parent_id: Some(level3),
        },
    )
    .await
    .expect("Failed to create level 4");

    // Create another chain with 2 levels
    let other1 = super::generate_test_id().await;
    let other2 = super::generate_test_id().await;

    repo.create(
        ctx,
        other1,
        &CategoryCreate {
            name: "Other 1".to_string(),
            description: None,
            parent_id: None,
        },
    )
    .await
    .expect("Failed to create other 1");
    repo.create(
        ctx,
        other2,
        &CategoryCreate {
            name: "Other 2".to_string(),
            description: None,
            parent_id: Some(other1),
        },
    )
    .await
    .expect("Failed to create other 2");

    // Try to move other1 (which has 1 child) under level4
    // This would create: level1 -> level2 -> level3 -> level4 -> other1 -> other2
    // Total depth = 6, which exceeds MAX_DEPTH = 5
    let update = CategoryUpdate {
        name: None,
        description: Update::Unchanged,
        parent_id: Update::Set(level4),
    };
    let result = repo.update(ctx, other1, &update).await;

    assert!(result.is_err());
    assert!(matches!(result, Err(crate::domain::Error::Database(_))));
}

pub async fn category_test_move_within_depth_limit<C: CategoryRepository>(ctx: &Context, repo: C) {
    // Create chain of 3 levels
    let level1 = super::generate_test_id().await;
    let level2 = super::generate_test_id().await;
    let level3 = super::generate_test_id().await;

    repo.create(
        ctx,
        level1,
        &CategoryCreate {
            name: "Level 1".to_string(),
            description: None,
            parent_id: None,
        },
    )
    .await
    .expect("Failed to create level 1");
    repo.create(
        ctx,
        level2,
        &CategoryCreate {
            name: "Level 2".to_string(),
            description: None,
            parent_id: Some(level1),
        },
    )
    .await
    .expect("Failed to create level 2");
    repo.create(
        ctx,
        level3,
        &CategoryCreate {
            name: "Level 3".to_string(),
            description: None,
            parent_id: Some(level2),
        },
    )
    .await
    .expect("Failed to create level 3");

    // Create standalone category with 1 child
    let other1 = super::generate_test_id().await;
    let other2 = super::generate_test_id().await;

    repo.create(
        ctx,
        other1,
        &CategoryCreate {
            name: "Other 1".to_string(),
            description: None,
            parent_id: None,
        },
    )
    .await
    .expect("Failed to create other 1");
    repo.create(
        ctx,
        other2,
        &CategoryCreate {
            name: "Other 2".to_string(),
            description: None,
            parent_id: Some(other1),
        },
    )
    .await
    .expect("Failed to create other 2");

    // Move other1 under level3
    // This creates: level1 -> level2 -> level3 -> other1 -> other2
    // Total depth = 5, which is exactly at MAX_DEPTH
    let update = CategoryUpdate {
        name: None,
        description: Update::Unchanged,
        parent_id: Update::Set(level3),
    };
    repo.update(ctx, other1, &update)
        .await
        .expect("Move should succeed");

    // Verify the structure
    let root = repo
        .get_by_id(ctx, level1)
        .await
        .expect("Failed to get root")
        .expect("Root not found");

    // Navigate through the tree to verify
    let l2 = &root.children.as_ref().unwrap()[0];
    let l3 = &l2.children.as_ref().unwrap()[0];
    let o1 = &l3.children.as_ref().unwrap()[0];
    let o2 = &o1.children.as_ref().unwrap()[0];

    assert_eq!(o2.name, "Other 2".to_string());
}

// =============================================================================
// Soft Delete Tests
// =============================================================================

pub async fn category_test_deleted_not_in_get_all<C: CategoryRepository>(ctx: &Context, repo: C) {
    let id1 = super::generate_test_id().await;
    let id2 = super::generate_test_id().await;

    repo.create(
        ctx,
        id1,
        &CategoryCreate {
            name: "Category 1".to_string(),
            description: None,
            parent_id: None,
        },
    )
    .await
    .expect("Failed to create category 1");
    repo.create(
        ctx,
        id2,
        &CategoryCreate {
            name: "Category 2".to_string(),
            description: None,
            parent_id: None,
        },
    )
    .await
    .expect("Failed to create category 2");

    repo.delete(ctx, id1)
        .await
        .expect("Failed to delete category");

    let categories = repo.get_all(ctx).await.expect("Failed to get all");
    assert_eq!(categories.len(), 1);
    assert_eq!(categories[0].id, id2);
}

pub async fn category_test_deleted_child_not_returned<C: CategoryRepository>(
    ctx: &Context,
    repo: C,
) {
    let parent = super::generate_test_id().await;
    let child1 = super::generate_test_id().await;
    let child2 = super::generate_test_id().await;

    repo.create(
        ctx,
        parent,
        &CategoryCreate {
            name: "Parent".to_string(),
            description: None,
            parent_id: None,
        },
    )
    .await
    .expect("Failed to create parent");
    repo.create(
        ctx,
        child1,
        &CategoryCreate {
            name: "Child 1".to_string(),
            description: None,
            parent_id: Some(parent),
        },
    )
    .await
    .expect("Failed to create child 1");
    repo.create(
        ctx,
        child2,
        &CategoryCreate {
            name: "Child 2".to_string(),
            description: None,
            parent_id: Some(parent),
        },
    )
    .await
    .expect("Failed to create child 2");

    repo.delete(ctx, child1)
        .await
        .expect("Failed to delete child 1");

    let parent_cat = repo
        .get_by_id(ctx, parent)
        .await
        .expect("Failed to get parent")
        .expect("Parent not found");

    assert!(parent_cat.children.is_some());
    assert_eq!(parent_cat.children.as_ref().unwrap().len(), 1);
    assert_eq!(parent_cat.children.as_ref().unwrap()[0].id, child2);
}

pub async fn category_test_cannot_delete_already_deleted<C: CategoryRepository>(
    ctx: &Context,
    repo: C,
) {
    let id = super::generate_test_id().await;
    repo.create(
        ctx,
        id,
        &CategoryCreate {
            name: "Category".to_string(),
            description: None,
            parent_id: None,
        },
    )
    .await
    .expect("Failed to create category");

    repo.delete(ctx, id)
        .await
        .expect("Failed to delete category");

    // Try to delete again
    let result = repo.delete(ctx, id).await;
    assert!(matches!(result, Err(crate::domain::Error::NotFound(_))));
}

pub async fn category_test_cannot_update_deleted<C: CategoryRepository>(ctx: &Context, repo: C) {
    let id = super::generate_test_id().await;
    repo.create(
        ctx,
        id,
        &CategoryCreate {
            name: "Category".to_string(),
            description: None,
            parent_id: None,
        },
    )
    .await
    .expect("Failed to create category");

    repo.delete(ctx, id)
        .await
        .expect("Failed to delete category");

    let update = CategoryUpdate {
        name: Some("New Name".to_string()),
        description: Update::Unchanged,
        parent_id: Update::Unchanged,
    };
    let result = repo.update(ctx, id, &update).await;
    assert!(matches!(result, Err(crate::domain::Error::NotFound(_))));
}

// =============================================================================
// Edge Cases
// =============================================================================

pub async fn category_test_get_child_by_id<C: CategoryRepository>(ctx: &Context, repo: C) {
    let parent = super::generate_test_id().await;
    let child = super::generate_test_id().await;
    let grandchild = super::generate_test_id().await;

    repo.create(
        ctx,
        parent,
        &CategoryCreate {
            name: "Parent".to_string(),
            description: None,
            parent_id: None,
        },
    )
    .await
    .expect("Failed to create parent");
    repo.create(
        ctx,
        child,
        &CategoryCreate {
            name: "Child".to_string(),
            description: None,
            parent_id: Some(parent),
        },
    )
    .await
    .expect("Failed to create child");
    repo.create(
        ctx,
        grandchild,
        &CategoryCreate {
            name: "Grandchild".to_string(),
            description: None,
            parent_id: Some(child),
        },
    )
    .await
    .expect("Failed to create grandchild");

    // Get child by id - should include its children (grandchild)
    let child_cat = repo
        .get_by_id(ctx, child)
        .await
        .expect("Failed to get child")
        .expect("Child not found");

    assert_eq!(child_cat.id, child);
    assert!(child_cat.children.is_some());
    assert_eq!(child_cat.children.as_ref().unwrap().len(), 1);
    assert_eq!(child_cat.children.as_ref().unwrap()[0].id, grandchild);
}

pub async fn category_test_without_children<C: CategoryRepository>(ctx: &Context, repo: C) {
    let id = super::generate_test_id().await;
    repo.create(
        ctx,
        id,
        &CategoryCreate {
            name: "Leaf Category".to_string(),
            description: None,
            parent_id: None,
        },
    )
    .await
    .expect("Failed to create category");

    let cat = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get category")
        .expect("Category not found");

    // Category without children should have None for children field
    assert!(cat.children.is_none());
}

pub async fn category_test_deep_nested_tree_retrieval<C: CategoryRepository>(
    ctx: &Context,
    repo: C,
) {
    // Create max depth tree
    let level1 = super::generate_test_id().await;
    let level2 = super::generate_test_id().await;
    let level3 = super::generate_test_id().await;
    let level4 = super::generate_test_id().await;
    let level5 = super::generate_test_id().await;

    repo.create(
        ctx,
        level1,
        &CategoryCreate {
            name: "Level 1".to_string(),
            description: None,
            parent_id: None,
        },
    )
    .await
    .unwrap();
    repo.create(
        ctx,
        level2,
        &CategoryCreate {
            name: "Level 2".to_string(),
            description: None,
            parent_id: Some(level1),
        },
    )
    .await
    .unwrap();
    repo.create(
        ctx,
        level3,
        &CategoryCreate {
            name: "Level 3".to_string(),
            description: None,
            parent_id: Some(level2),
        },
    )
    .await
    .unwrap();
    repo.create(
        ctx,
        level4,
        &CategoryCreate {
            name: "Level 4".to_string(),
            description: None,
            parent_id: Some(level3),
        },
    )
    .await
    .unwrap();
    repo.create(
        ctx,
        level5,
        &CategoryCreate {
            name: "Level 5".to_string(),
            description: None,
            parent_id: Some(level4),
        },
    )
    .await
    .unwrap();

    // Get from root and verify entire tree is returned
    let root = repo
        .get_by_id(ctx, level1)
        .await
        .expect("Failed to get root")
        .expect("Root not found");

    // Navigate to level 5
    let l2 = &root.children.as_ref().unwrap()[0];
    let l3 = &l2.children.as_ref().unwrap()[0];
    let l4 = &l3.children.as_ref().unwrap()[0];
    let l5 = &l4.children.as_ref().unwrap()[0];

    assert_eq!(l5.name, "Level 5".to_string());
    assert!(l5.children.is_none()); // Level 5 has no children
}
