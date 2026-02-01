use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::{
    domain::model::{
        Update,
        pagination::PaginationOptions,
        user::{UserCreate, UserFilter, UserUpdate},
    },
    storage::{RepoCtx, UserRepository},
};

/// Runs all user repository tests with the given repository and context factory.
///
/// # Example
///
/// ```rust,ignore
/// #[tokio::test]
/// async fn test_sqlite_user_repo() {
///     let repo = SqliteUserRepository::new();
///     let ctx_factory = || async { init_sqlite_repo_ctx().await };
///     user_test_all(&repo, ctx_factory).await;
/// }
/// ```
pub async fn user_test_all<C, F, Fut>(repo: &C, ctx_factory: F)
where
    C: UserRepository,
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = RepoCtx<DatabaseConnection>>,
{
    // Basic CRUD tests
    user_test_create_and_get_integration(&ctx_factory().await, repo).await;
    user_test_create_duplicate(&ctx_factory().await, repo).await;
    user_test_update(&ctx_factory().await, repo).await;
    user_test_update_not_found(&ctx_factory().await, repo).await;
    user_test_update_password(&ctx_factory().await, repo).await;
    user_test_delete(&ctx_factory().await, repo).await;
    user_test_delete_not_found(&ctx_factory().await, repo).await;
    user_test_update_password_not_found(&ctx_factory().await, repo).await;

    // Pagination tests
    user_test_get_all_pagination(&ctx_factory().await, repo).await;

    // Filter tests
    user_test_filter_by_username(&ctx_factory().await, repo).await;
    user_test_filter_by_name(&ctx_factory().await, repo).await;
    user_test_filter_combined(&ctx_factory().await, repo).await;
    user_test_filter_by_email(&ctx_factory().await, repo).await;

    // Get tests
    user_test_get_by_id(&ctx_factory().await, repo).await;
    user_test_get_by_id_not_found(&ctx_factory().await, repo).await;
    user_test_get_by_username_not_found(&ctx_factory().await, repo).await;

    // Permission tests
    user_test_save_permissions(&ctx_factory().await, repo).await;
    user_test_save_permissions_updates_existing(&ctx_factory().await, repo).await;
    user_test_delete_permission_by_user_id(&ctx_factory().await, repo).await;
    user_test_delete_permission_by_user_id_no_permissions(&ctx_factory().await, repo).await;
    user_test_get_permissions_empty(&ctx_factory().await, repo).await;
}

// =============================================================================
// Basic CRUD Tests
// =============================================================================

pub async fn user_test_create_and_get_integration<U: UserRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
) {
    let username = Uuid::new_v4().to_string();
    let name = "Integration User";
    let email = "integration@example.com";
    let password_hash = "hashed_password";

    let user = UserCreate {
        username: username.clone(),
        name: name.to_string(),
        email: Some(email.to_string()),
        password: password_hash.to_string(),
        photo: None,
        pin: None,
        address: None,
        phone: None,
    };

    repo.create(ctx, super::generate_test_id().await, &user)
        .await
        .expect("Failed to create user");

    let user = repo
        .get_by_username(ctx, &username)
        .await
        .expect("Failed to get user")
        .expect("User not found");

    assert_eq!(user.username, username);
    assert_eq!(user.name, name);
    assert_eq!(user.email, Some(email.to_string()));
    assert_eq!(user.password, password_hash);
}

pub async fn user_test_create_duplicate<U: UserRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
) {
    let user = UserCreate {
        username: Uuid::new_v4().to_string(),
        name: "Duplicate".to_string(),
        email: None,
        password: "pass".to_string(),
        photo: None,
        pin: None,
        address: None,
        phone: None,
    };

    repo.create(ctx, super::generate_test_id().await, &user)
        .await
        .expect("Failed to create user");

    let result = repo
        .create(ctx, super::generate_test_id().await, &user)
        .await;
    assert!(result.is_err());
}

pub async fn user_test_update<U: UserRepository>(ctx: &RepoCtx<DatabaseConnection>, repo: &U) {
    let user = UserCreate {
        username: Uuid::new_v4().to_string(),
        name: "Original".to_string(),
        email: None,
        password: "pass".to_string(),
        photo: None,
        pin: None,
        address: None,
        phone: None,
    };

    let id = super::generate_test_id().await;
    repo.create(ctx, id, &user)
        .await
        .expect("Failed to create user");

    let saved_user = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get user")
        .expect("User not found");

    let updated_user = UserUpdate {
        username: None,
        name: Some("Updated".to_string()),
        email: Update::Unchanged,
        photo: Update::Unchanged,
        pin: Update::Unchanged,
        address: Update::Unchanged,
        phone: Update::Unchanged,
    };
    repo.update(ctx, saved_user.id, &updated_user)
        .await
        .expect("Failed to update user");

    let updated_user = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get user")
        .expect("User not found");

    assert_eq!(updated_user.name, "Updated");
}

pub async fn user_test_update_not_found<U: UserRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
) {
    let user = UserUpdate {
        username: Some("non_existent".to_string()),
        name: Some("Non Existent".to_string()),
        email: Update::Unchanged,
        photo: Update::Unchanged,
        pin: Update::Unchanged,
        address: Update::Unchanged,
        phone: Update::Unchanged,
    };

    let result = repo.update(ctx, 999, &user).await;
    assert!(matches!(result, Err(crate::domain::Error::NotFound(_))));
}

pub async fn user_test_update_password<U: UserRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
) {
    let user = UserCreate {
        username: Uuid::new_v4().to_string(),
        name: "Password Test".to_string(),
        email: None,
        password: "old_pass".to_string(),
        photo: None,
        pin: None,
        address: None,
        phone: None,
    };

    let id = super::generate_test_id().await;
    repo.create(ctx, id, &user)
        .await
        .expect("Failed to create user");

    let saved_user = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get user")
        .expect("User not found");

    repo.update_password(ctx, saved_user.id, "new_pass")
        .await
        .expect("Failed to update password");

    let _updated_user = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get user")
        .expect("User not found");
}

pub async fn user_test_delete<U: UserRepository>(ctx: &RepoCtx<DatabaseConnection>, repo: &U) {
    let user = UserCreate {
        username: Uuid::new_v4().to_string(),
        name: "Delete Test".to_string(),
        email: None,
        password: "pass".to_string(),
        photo: None,
        pin: None,
        address: None,
        phone: None,
    };

    let id = super::generate_test_id().await;
    repo.create(ctx, id, &user)
        .await
        .expect("Failed to create user");

    let saved_user = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get user")
        .expect("User not found");

    repo.delete(ctx, saved_user.id)
        .await
        .expect("Failed to delete user");

    let deleted_user = repo.get_by_id(ctx, id).await.expect("Failed to get user");
    assert!(deleted_user.is_none());
}

pub async fn user_test_delete_not_found<U: UserRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
) {
    let result = repo.delete(ctx, 9999).await;
    assert!(matches!(result, Err(crate::domain::Error::NotFound(_))));
}

pub async fn user_test_update_password_not_found<U: UserRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
) {
    let result = repo.update_password(ctx, 9999, "new_pass").await;
    assert!(matches!(result, Err(crate::domain::Error::NotFound(_))));
}

// =============================================================================
// Pagination Tests
// =============================================================================

pub async fn user_test_get_all_pagination<U: UserRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
) {
    for i in 0..15 {
        let user = UserCreate {
            username: format!("user_{}", Uuid::new_v4()),
            name: format!("User {}", i),
            email: None,
            password: "pass".to_string(),
            photo: None,
            pin: None,
            address: None,
            phone: None,
        };
        repo.create(ctx, super::generate_test_id().await, &user)
            .await
            .expect("Failed to create user");
    }

    let pagination = PaginationOptions::new(1, 10, None);
    let users = repo
        .get_all(ctx, &UserFilter::new(), &pagination)
        .await
        .expect("Failed to get users");
    assert_eq!(users.len(), 10);

    let pagination = PaginationOptions::new(2, 10, None);
    let users = repo
        .get_all(ctx, &UserFilter::new(), &pagination)
        .await
        .expect("Failed to get users");
    assert!(!users.is_empty());
}

// =============================================================================
// Filter Tests
// =============================================================================

pub async fn user_test_filter_by_username<U: UserRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
) {
    let users_data = vec![
        (
            format!("filter_admin_user1_{}", Uuid::new_v4()),
            "Admin User One",
        ),
        (
            format!("filter_admin_super_{}", Uuid::new_v4()),
            "Super Admin",
        ),
        (
            format!("filter_regular_user_{}", Uuid::new_v4()),
            "Regular User",
        ),
    ];

    for (username, name) in &users_data {
        let user = UserCreate {
            username: username.to_string(),
            password: "hash".to_string(),
            name: name.to_string(),
            email: None,
            photo: None,
            pin: None,
            address: None,
            phone: None,
        };
        repo.create(ctx, super::generate_test_id().await, &user)
            .await
            .unwrap();
    }

    let filter = UserFilter::new().with_username(users_data[0].0.as_str());
    let pagination = PaginationOptions::new(1, 10, None);
    let users = repo.get_all(ctx, &filter, &pagination).await.unwrap();

    assert_eq!(users.len(), 1);
}

pub async fn user_test_filter_by_name<U: UserRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
) {
    let users_data = vec![
        (
            format!("filter_name_user1_{}", Uuid::new_v4()),
            "John FilterSmith",
        ),
        (
            format!("filter_name_user2_{}", Uuid::new_v4()),
            "Jane FilterSmith",
        ),
        (
            format!("filter_name_user3_{}", Uuid::new_v4()),
            "Bob Johnson",
        ),
    ];

    for (username, name) in users_data {
        let user = UserCreate {
            username: username.to_string(),
            password: "hash".to_string(),
            name: name.to_string(),
            email: None,
            photo: None,
            pin: None,
            address: None,
            phone: None,
        };
        repo.create(ctx, super::generate_test_id().await, &user)
            .await
            .unwrap();
    }

    let filter = UserFilter::new().with_name("FilterSmith");
    let pagination = PaginationOptions::new(1, 10, None);
    let users = repo.get_all(ctx, &filter, &pagination).await.unwrap();

    assert_eq!(users.len(), 2);
    assert!(users.iter().all(|u| u.name.contains("FilterSmith")));
}

pub async fn user_test_filter_combined<U: UserRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
) {
    let users_data = vec![
        (
            format!("combined_admin_john_{}", Uuid::new_v4()),
            "John CombinedTest",
        ),
        (
            format!("combined_admin_jane_{}", Uuid::new_v4()),
            "Jane Doe",
        ),
        (
            format!("combined_user_john_{}", Uuid::new_v4()),
            "John Johnson",
        ),
    ];

    for (username, name) in &users_data {
        let user = UserCreate {
            username: username.to_string(),
            password: "hash".to_string(),
            name: name.to_string(),
            email: None,
            photo: None,
            pin: None,
            address: None,
            phone: None,
        };
        repo.create(ctx, super::generate_test_id().await, &user)
            .await
            .unwrap();
    }

    let filter = UserFilter::new()
        .with_username(users_data[0].0.as_str())
        .with_name("CombinedTest");
    let pagination = PaginationOptions::new(1, 10, None);
    let users = repo.get_all(ctx, &filter, &pagination).await.unwrap();

    assert_eq!(users.len(), 1);
    assert_eq!(users[0].username, users_data[0].0);
    assert_eq!(users[0].name, "John CombinedTest");
}

pub async fn user_test_filter_by_email<U: UserRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
) {
    let users_data = vec![
        (
            format!("email_user1_{}", Uuid::new_v4()),
            "User One",
            Some("user1@company.com"),
        ),
        (
            format!("email_user2_{}", Uuid::new_v4()),
            "User Two",
            Some("user2@company.com"),
        ),
        (
            format!("email_user3_{}", Uuid::new_v4()),
            "User Three",
            Some("user3@other.org"),
        ),
        (format!("email_user4_{}", Uuid::new_v4()), "User Four", None),
    ];

    for (username, name, email) in &users_data {
        let user = UserCreate {
            username: username.to_string(),
            password: "hash".to_string(),
            name: name.to_string(),
            email: email.map(|e| e.to_string()),
            photo: None,
            pin: None,
            address: None,
            phone: None,
        };
        repo.create(ctx, super::generate_test_id().await, &user)
            .await
            .unwrap();
    }

    let filter = UserFilter::new().with_email("user1@company.com");
    let pagination = PaginationOptions::new(1, 10, None);
    let users = repo.get_all(ctx, &filter, &pagination).await.unwrap();

    assert_eq!(users.len(), 1);
    assert_eq!(users[0].email, Some("user1@company.com".to_string()));
    assert_eq!(users[0].username, users_data[0].0);
}

// =============================================================================
// Get Tests
// =============================================================================

pub async fn user_test_get_by_id<U: UserRepository>(ctx: &RepoCtx<DatabaseConnection>, repo: &U) {
    let user = UserCreate {
        username: Uuid::new_v4().to_string(),
        name: "ID Test".to_string(),
        email: Some("id@test.com".to_string()),
        password: "pass".to_string(),
        photo: Some("photo.jpg".to_string()),
        pin: Some("1234".to_string()),
        address: Some("123 Test St".to_string()),
        phone: Some("555-0000".to_string()),
    };

    let id = super::generate_test_id().await;
    repo.create(ctx, id, &user)
        .await
        .expect("Failed to create user");

    let fetched_user = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get user by ID")
        .expect("User not found by ID");

    assert_eq!(fetched_user.username, user.username);
    assert_eq!(fetched_user.email, user.email);
    assert_eq!(fetched_user.photo, user.photo);
    assert_eq!(fetched_user.pin, user.pin);
    assert_eq!(fetched_user.address, user.address);
    assert_eq!(fetched_user.phone, user.phone);
}

pub async fn user_test_get_by_id_not_found<U: UserRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
) {
    let result = repo.get_by_id(ctx, 9999).await.expect("Failed to query");
    assert!(result.is_none());
}

pub async fn user_test_get_by_username_not_found<U: UserRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
) {
    let result = repo
        .get_by_username(ctx, "nonexistent_user_xyz")
        .await
        .expect("Failed to query");

    assert!(result.is_none());
}

// =============================================================================
// Permission Tests
// =============================================================================

/// Tests saving multiple permissions for a user.
///
/// Verifies that:
/// - Multiple permissions can be saved at once
/// - Each permission is correctly stored with user_id, branch_id, resource, and action
pub async fn user_test_save_permissions<U: UserRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
) {
    use crate::domain::model::permission::Permission;

    let user = UserCreate {
        username: Uuid::new_v4().to_string(),
        name: "Permission Save User".to_string(),
        email: None,
        password: "pass".to_string(),
        photo: None,
        pin: None,
        address: None,
        phone: None,
    };

    let id = super::generate_test_id().await;
    repo.create(ctx, id, &user)
        .await
        .expect("Failed to create user");

    let saved_user = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get user")
        .expect("User not found");

    let permissions = vec![
        Permission {
            user_id: saved_user.id,
            branch_id: None,
            resource: 2,
            action: 3,
        },
        Permission {
            user_id: saved_user.id,
            branch_id: None,
            resource: 3,
            action: 7,
        },
        Permission {
            user_id: saved_user.id,
            branch_id: None,
            resource: 4,
            action: 15,
        },
    ];

    repo.save_permissions(ctx, saved_user.id, &permissions)
        .await
        .expect("Failed to save permissions");

    let result = repo
        .get_permissions(ctx, saved_user.id)
        .await
        .expect("Failed to get permissions");

    assert_eq!(result.len(), 3);

    // Verify each permission exists
    let has_perm_1 = result
        .iter()
        .any(|p| p.branch_id.is_none() && p.resource == 2 && p.action == 3);
    let has_perm_2 = result
        .iter()
        .any(|p| p.branch_id.is_none() && p.resource == 3 && p.action == 7);
    let has_perm_3 = result
        .iter()
        .any(|p| p.branch_id.is_none() && p.resource == 4 && p.action == 15);

    assert!(has_perm_1, "Permission 1 not found");
    assert!(has_perm_2, "Permission 2 not found");
    assert!(has_perm_3, "Permission 3 not found");
}

/// Tests that save_permissions updates existing permissions.
///
/// Verifies that:
/// - Saving a permission with the same user_id, branch_id, and resource replaces the action
pub async fn user_test_save_permissions_updates_existing<U: UserRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
) {
    use crate::domain::model::permission::Permission;

    let user = UserCreate {
        username: Uuid::new_v4().to_string(),
        name: "Permission Update User".to_string(),
        email: None,
        password: "pass".to_string(),
        photo: None,
        pin: None,
        address: None,
        phone: None,
    };

    let id = super::generate_test_id().await;
    repo.create(ctx, id, &user)
        .await
        .expect("Failed to create user");

    let saved_user = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get user")
        .expect("User not found");

    // Save initial permissions
    let initial_permissions = vec![Permission {
        user_id: saved_user.id,
        branch_id: None,
        resource: 5,
        action: 1,
    }];

    repo.save_permissions(ctx, saved_user.id, &initial_permissions)
        .await
        .expect("Failed to save initial permissions");

    let result = repo
        .get_permissions(ctx, saved_user.id)
        .await
        .expect("Failed to get permissions");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].action, 1);
}

/// Tests deleting all permissions for a user.
///
/// Verifies that:
/// - All permissions for a user are deleted
/// - Permissions for other users are not affected
pub async fn user_test_delete_permission_by_user_id<U: UserRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
) {
    use crate::domain::model::permission::Permission;

    let user = UserCreate {
        username: Uuid::new_v4().to_string(),
        name: "Permission Delete User".to_string(),
        email: None,
        password: "pass".to_string(),
        photo: None,
        pin: None,
        address: None,
        phone: None,
    };

    let id = super::generate_test_id().await;
    repo.create(ctx, id, &user)
        .await
        .expect("Failed to create user");

    let saved_user = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get user")
        .expect("User not found");

    // Save some permissions
    let permissions = vec![
        Permission {
            user_id: saved_user.id,
            branch_id: None,
            resource: 2,
            action: 3,
        },
        Permission {
            user_id: saved_user.id,
            branch_id: None,
            resource: 3,
            action: 7,
        },
    ];

    repo.save_permissions(ctx, saved_user.id, &permissions)
        .await
        .expect("Failed to save permissions");

    let result = repo
        .get_permissions(ctx, saved_user.id)
        .await
        .expect("Failed to get permissions");
    assert_eq!(result.len(), 2);

    // Delete all permissions
    repo.delete_permission_by_user_id(ctx, saved_user.id)
        .await
        .expect("Failed to delete permissions by user_id");

    let result = repo
        .get_permissions(ctx, saved_user.id)
        .await
        .expect("Failed to get permissions");
    assert_eq!(result.len(), 0);
}

/// Tests deleting permissions for a user that has no permissions.
///
/// Verifies that:
/// - No error is returned when deleting permissions for a user with no permissions
pub async fn user_test_delete_permission_by_user_id_no_permissions<U: UserRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
) {
    let user = UserCreate {
        username: Uuid::new_v4().to_string(),
        name: "Permission Delete No Perm User".to_string(),
        email: None,
        password: "pass".to_string(),
        photo: None,
        pin: None,
        address: None,
        phone: None,
    };

    let id = super::generate_test_id().await;
    repo.create(ctx, id, &user)
        .await
        .expect("Failed to create user");

    let saved_user = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get user")
        .expect("User not found");

    // Should not error even when there are no permissions
    repo.delete_permission_by_user_id(ctx, saved_user.id)
        .await
        .expect("Failed to delete permissions by user_id (no permissions)");

    let result = repo
        .get_permissions(ctx, saved_user.id)
        .await
        .expect("Failed to get permissions");
    assert_eq!(result.len(), 0);
}

/// Tests getting permissions for a user with no permissions.
///
/// Verifies that:
/// - An empty list is returned when a user has no permissions
pub async fn user_test_get_permissions_empty<U: UserRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &U,
) {
    let user = UserCreate {
        username: Uuid::new_v4().to_string(),
        name: "Permission Empty User".to_string(),
        email: None,
        password: "pass".to_string(),
        photo: None,
        pin: None,
        address: None,
        phone: None,
    };

    let id = super::generate_test_id().await;
    repo.create(ctx, id, &user)
        .await
        .expect("Failed to create user");

    let saved_user = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get user")
        .expect("User not found");

    let permissions = repo
        .get_permissions(ctx, saved_user.id)
        .await
        .expect("Failed to get permissions");

    assert_eq!(permissions.len(), 0);
}
