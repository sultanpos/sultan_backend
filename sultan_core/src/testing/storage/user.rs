use crate::{
    domain::{
        Context,
        model::{
            Update,
            pagination::PaginationOptions,
            user::{UserCreate, UserFilter, UserUpdate},
        },
    },
    storage::UserRepository,
};

pub async fn create_sqlite_user_repo() -> (Context, impl UserRepository) {
    let pool = super::init_sqlite_pool().await;
    (
        Context::new(),
        crate::storage::sqlite::user::SqliteUserRepository::new(pool),
    )
}

// =============================================================================
// Basic CRUD Tests
// =============================================================================

pub async fn user_test_create_and_get_integration<U: UserRepository>(ctx: &Context, repo: U) {
    let username = "integration_user";
    let name = "Integration User";
    let email = "integration@example.com";
    let password_hash = "hashed_password";

    let user = UserCreate {
        username: username.to_string(),
        name: name.to_string(),
        email: Some(email.to_string()),
        password: password_hash.to_string(),
        photo: None,
        pin: None,
        address: None,
        phone: None,
    };

    repo.create_user(ctx, super::generate_test_id().await, &user)
        .await
        .expect("Failed to create user");

    let user = repo
        .get_user_by_username(ctx, username)
        .await
        .expect("Failed to get user")
        .expect("User not found");

    assert_eq!(user.username, username);
    assert_eq!(user.name, name);
    assert_eq!(user.email, Some(email.to_string()));
    assert_eq!(user.password, password_hash);
}

pub async fn user_test_create_duplicate<U: UserRepository>(ctx: &Context, repo: U) {
    let user = UserCreate {
        username: "duplicate".to_string(),
        name: "Duplicate".to_string(),
        email: None,
        password: "pass".to_string(),
        photo: None,
        pin: None,
        address: None,
        phone: None,
    };

    repo.create_user(ctx, super::generate_test_id().await, &user)
        .await
        .expect("Failed to create user");

    let result = repo
        .create_user(ctx, super::generate_test_id().await, &user)
        .await;
    assert!(result.is_err());
}

pub async fn user_test_update<U: UserRepository>(ctx: &Context, repo: U) {
    let user = UserCreate {
        username: "update_test".to_string(),
        name: "Original".to_string(),
        email: None,
        password: "pass".to_string(),
        photo: None,
        pin: None,
        address: None,
        phone: None,
    };

    repo.create_user(ctx, super::generate_test_id().await, &user)
        .await
        .expect("Failed to create user");

    let saved_user = repo
        .get_user_by_username(ctx, "update_test")
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
    repo.update_user(ctx, saved_user.id, &updated_user)
        .await
        .expect("Failed to update user");

    let updated_user = repo
        .get_user_by_username(ctx, "update_test")
        .await
        .expect("Failed to get user")
        .expect("User not found");

    assert_eq!(updated_user.name, "Updated");
}

pub async fn user_test_update_not_found<U: UserRepository>(ctx: &Context, repo: U) {
    let user = UserUpdate {
        username: Some("non_existent".to_string()),
        name: Some("Non Existent".to_string()),
        email: Update::Unchanged,
        photo: Update::Unchanged,
        pin: Update::Unchanged,
        address: Update::Unchanged,
        phone: Update::Unchanged,
    };

    let result = repo.update_user(ctx, 999, &user).await;
    assert!(matches!(result, Err(crate::domain::Error::NotFound(_))));
}

pub async fn user_test_update_password<U: UserRepository>(ctx: &Context, repo: U) {
    let user = UserCreate {
        username: "password_test".to_string(),
        name: "Password Test".to_string(),
        email: None,
        password: "old_pass".to_string(),
        photo: None,
        pin: None,
        address: None,
        phone: None,
    };

    repo.create_user(ctx, super::generate_test_id().await, &user)
        .await
        .expect("Failed to create user");

    let saved_user = repo
        .get_user_by_username(ctx, "password_test")
        .await
        .expect("Failed to get user")
        .expect("User not found");

    repo.update_password(ctx, saved_user.id, "new_pass")
        .await
        .expect("Failed to update password");

    let _updated_user = repo
        .get_user_by_username(ctx, "password_test")
        .await
        .expect("Failed to get user")
        .expect("User not found");
}

pub async fn user_test_delete<U: UserRepository>(ctx: &Context, repo: U) {
    let user = UserCreate {
        username: "delete_test".to_string(),
        name: "Delete Test".to_string(),
        email: None,
        password: "pass".to_string(),
        photo: None,
        pin: None,
        address: None,
        phone: None,
    };

    repo.create_user(ctx, super::generate_test_id().await, &user)
        .await
        .expect("Failed to create user");

    let saved_user = repo
        .get_user_by_username(ctx, "delete_test")
        .await
        .expect("Failed to get user")
        .expect("User not found");

    repo.delete_user(ctx, saved_user.id)
        .await
        .expect("Failed to delete user");

    let deleted_user = repo
        .get_user_by_username(ctx, "delete_test")
        .await
        .expect("Failed to get user");

    assert!(deleted_user.is_none());
}

// =============================================================================
// Pagination Tests
// =============================================================================

pub async fn user_test_get_all_pagination<U: UserRepository>(ctx: &Context, repo: U) {
    for i in 0..15 {
        let user = UserCreate {
            username: format!("user_{}", i),
            name: format!("User {}", i),
            email: None,
            password: "pass".to_string(),
            photo: None,
            pin: None,
            address: None,
            phone: None,
        };
        repo.create_user(ctx, super::generate_test_id().await, &user)
            .await
            .expect("Failed to create user");
    }

    let pagination = PaginationOptions::new(1, 10, None);
    let users = repo
        .get_all(ctx, UserFilter::new(), pagination)
        .await
        .expect("Failed to get users");
    assert_eq!(users.len(), 10);

    let pagination = PaginationOptions::new(2, 10, None);
    let users = repo
        .get_all(ctx, UserFilter::new(), pagination)
        .await
        .expect("Failed to get users");
    assert!(!users.is_empty());
}

// =============================================================================
// Filter Tests
// =============================================================================

pub async fn user_test_filter_by_username<U: UserRepository>(ctx: &Context, repo: U) {
    let users_data = vec![
        ("filter_admin_user1", "Admin User One"),
        ("filter_admin_super", "Super Admin"),
        ("filter_regular_user", "Regular User"),
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
        repo.create_user(ctx, super::generate_test_id().await, &user)
            .await
            .unwrap();
    }

    let filter = UserFilter::new().with_username("filter_admin");
    let pagination = PaginationOptions::new(1, 10, None);
    let users = repo.get_all(ctx, filter, pagination).await.unwrap();

    assert_eq!(users.len(), 2);
    assert!(users.iter().all(|u| u.username.starts_with("filter_admin")));
}

pub async fn user_test_filter_by_name<U: UserRepository>(ctx: &Context, repo: U) {
    let users_data = vec![
        ("filter_name_user1", "John FilterSmith"),
        ("filter_name_user2", "Jane FilterSmith"),
        ("filter_name_user3", "Bob Johnson"),
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
        repo.create_user(ctx, super::generate_test_id().await, &user)
            .await
            .unwrap();
    }

    let filter = UserFilter::new().with_name("FilterSmith");
    let pagination = PaginationOptions::new(1, 10, None);
    let users = repo.get_all(ctx, filter, pagination).await.unwrap();

    assert_eq!(users.len(), 2);
    assert!(users.iter().all(|u| u.name.contains("FilterSmith")));
}

pub async fn user_test_filter_combined<U: UserRepository>(ctx: &Context, repo: U) {
    let users_data = vec![
        ("combined_admin_john", "John CombinedTest"),
        ("combined_admin_jane", "Jane Doe"),
        ("combined_user_john", "John Johnson"),
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
        repo.create_user(ctx, super::generate_test_id().await, &user)
            .await
            .unwrap();
    }

    let filter = UserFilter::new()
        .with_username("combined_admin")
        .with_name("John");
    let pagination = PaginationOptions::new(1, 10, None);
    let users = repo.get_all(ctx, filter, pagination).await.unwrap();

    assert_eq!(users.len(), 1);
    assert_eq!(users[0].username, "combined_admin_john");
    assert_eq!(users[0].name, "John CombinedTest");
}

pub async fn user_test_filter_by_email<U: UserRepository>(ctx: &Context, repo: U) {
    let users_data = vec![
        ("email_user1", "User One", Some("user1@company.com")),
        ("email_user2", "User Two", Some("user2@company.com")),
        ("email_user3", "User Three", Some("user3@other.org")),
        ("email_user4", "User Four", None),
    ];

    for (username, name, email) in users_data {
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
        repo.create_user(ctx, super::generate_test_id().await, &user)
            .await
            .unwrap();
    }

    let filter = UserFilter::new().with_email("user1@company.com");
    let pagination = PaginationOptions::new(1, 10, None);
    let users = repo.get_all(ctx, filter, pagination).await.unwrap();

    assert_eq!(users.len(), 1);
    assert_eq!(users[0].email, Some("user1@company.com".to_string()));
    assert_eq!(users[0].username, "email_user1");
}

// =============================================================================
// Get Tests
// =============================================================================

pub async fn user_test_get_by_id<U: UserRepository>(ctx: &Context, repo: U) {
    let user = UserCreate {
        username: "id_test".to_string(),
        name: "ID Test".to_string(),
        email: Some("id@test.com".to_string()),
        password: "pass".to_string(),
        photo: Some("photo.jpg".to_string()),
        pin: Some("1234".to_string()),
        address: Some("123 Test St".to_string()),
        phone: Some("555-0000".to_string()),
    };

    repo.create_user(ctx, super::generate_test_id().await, &user)
        .await
        .expect("Failed to create user");

    let saved_user = repo
        .get_user_by_username(ctx, "id_test")
        .await
        .expect("Failed to get user by username")
        .expect("User not found by username");

    let fetched_user = repo
        .get_by_id(ctx, saved_user.id)
        .await
        .expect("Failed to get user by ID")
        .expect("User not found by ID");

    assert_eq!(fetched_user.username, "id_test");
    assert_eq!(fetched_user.email, Some("id@test.com".to_string()));
    assert_eq!(fetched_user.photo, Some("photo.jpg".to_string()));
    assert_eq!(fetched_user.pin, Some("1234".to_string()));
    assert_eq!(fetched_user.address, Some("123 Test St".to_string()));
    assert_eq!(fetched_user.phone, Some("555-0000".to_string()));
}

pub async fn user_test_get_by_id_not_found<U: UserRepository>(ctx: &Context, repo: U) {
    let result = repo.get_by_id(ctx, 9999).await.expect("Failed to query");
    assert!(result.is_none());
}

pub async fn user_test_delete_not_found<U: UserRepository>(ctx: &Context, repo: U) {
    let result = repo.delete_user(ctx, 9999).await;
    assert!(matches!(result, Err(crate::domain::Error::NotFound(_))));
}

pub async fn user_test_update_password_not_found<U: UserRepository>(ctx: &Context, repo: U) {
    let result = repo.update_password(ctx, 9999, "new_pass").await;
    assert!(matches!(result, Err(crate::domain::Error::NotFound(_))));
}

pub async fn user_test_get_by_username_not_found<U: UserRepository>(ctx: &Context, repo: U) {
    let result = repo
        .get_user_by_username(ctx, "nonexistent_user_xyz")
        .await
        .expect("Failed to query");

    assert!(result.is_none());
}

// =============================================================================
// Permission Tests
// =============================================================================

pub async fn user_test_save_permission_with_branch<U: UserRepository>(ctx: &Context, repo: U) {
    let user = UserCreate {
        username: "perm_user_1".to_string(),
        name: "Permission User 1".to_string(),
        email: None,
        password: "pass".to_string(),
        photo: None,
        pin: None,
        address: None,
        phone: None,
    };

    repo.create_user(ctx, super::generate_test_id().await, &user)
        .await
        .expect("Failed to create user");

    let saved_user = repo
        .get_user_by_username(ctx, "perm_user_1")
        .await
        .expect("Failed to get user")
        .expect("User not found");

    repo.save_user_permission(ctx, saved_user.id, None, 2, 3)
        .await
        .expect("Failed to save permission");

    let permissions = repo
        .get_user_permission(ctx, saved_user.id)
        .await
        .expect("Failed to get permissions");

    assert_eq!(permissions.len(), 1);
    assert_eq!(permissions[0].user_id, saved_user.id);
    assert_eq!(permissions[0].branch_id, None);
    assert_eq!(permissions[0].resource, 2);
    assert_eq!(permissions[0].action, 3);
}

pub async fn user_test_save_permission_without_branch<U: UserRepository>(ctx: &Context, repo: U) {
    let user = UserCreate {
        username: "perm_user_2".to_string(),
        name: "Permission User 2".to_string(),
        email: None,
        password: "pass".to_string(),
        photo: None,
        pin: None,
        address: None,
        phone: None,
    };

    repo.create_user(ctx, super::generate_test_id().await, &user)
        .await
        .expect("Failed to create user");

    let saved_user = repo
        .get_user_by_username(ctx, "perm_user_2")
        .await
        .expect("Failed to get user")
        .expect("User not found");

    repo.save_user_permission(ctx, saved_user.id, None, 2, 3)
        .await
        .expect("Failed to save permission");

    let permissions = repo
        .get_user_permission(ctx, saved_user.id)
        .await
        .expect("Failed to get permissions");

    assert_eq!(permissions.len(), 1);
    assert_eq!(permissions[0].user_id, saved_user.id);
    assert_eq!(permissions[0].branch_id, None);
    assert_eq!(permissions[0].resource, 2);
    assert_eq!(permissions[0].action, 3);
}

pub async fn user_test_save_multiple_permissions<U: UserRepository>(ctx: &Context, repo: U) {
    let user = UserCreate {
        username: "perm_user_3".to_string(),
        name: "Permission User 3".to_string(),
        email: None,
        password: "pass".to_string(),
        photo: None,
        pin: None,
        address: None,
        phone: None,
    };

    repo.create_user(ctx, super::generate_test_id().await, &user)
        .await
        .expect("Failed to create user");

    let saved_user = repo
        .get_user_by_username(ctx, "perm_user_3")
        .await
        .expect("Failed to get user")
        .expect("User not found");

    repo.save_user_permission(ctx, saved_user.id, None, 2, 3)
        .await
        .expect("Failed to save permission 1");

    repo.save_user_permission(ctx, saved_user.id, None, 3, 4)
        .await
        .expect("Failed to save permission 2");

    repo.save_user_permission(ctx, saved_user.id, None, 4, 5)
        .await
        .expect("Failed to save permission 3");

    let permissions = repo
        .get_user_permission(ctx, saved_user.id)
        .await
        .expect("Failed to get permissions");

    assert_eq!(permissions.len(), 3);
}

pub async fn user_test_delete_permission_with_branch<U: UserRepository>(ctx: &Context, repo: U) {
    let user = UserCreate {
        username: "perm_user_4".to_string(),
        name: "Permission User 4".to_string(),
        email: None,
        password: "pass".to_string(),
        photo: None,
        pin: None,
        address: None,
        phone: None,
    };

    repo.create_user(ctx, super::generate_test_id().await, &user)
        .await
        .expect("Failed to create user");

    let saved_user = repo
        .get_user_by_username(ctx, "perm_user_4")
        .await
        .expect("Failed to get user")
        .expect("User not found");

    repo.save_user_permission(ctx, saved_user.id, None, 2, 3)
        .await
        .expect("Failed to save permission");

    let permissions = repo
        .get_user_permission(ctx, saved_user.id)
        .await
        .expect("Failed to get permissions");
    assert_eq!(permissions.len(), 1);

    repo.delete_user_permission(ctx, saved_user.id, None, 2)
        .await
        .expect("Failed to delete permission");

    let permissions = repo
        .get_user_permission(ctx, saved_user.id)
        .await
        .expect("Failed to get permissions");
    assert_eq!(permissions.len(), 0);
}

pub async fn user_test_delete_permission_without_branch<U: UserRepository>(ctx: &Context, repo: U) {
    let user = UserCreate {
        username: "perm_user_5".to_string(),
        name: "Permission User 5".to_string(),
        email: None,
        password: "pass".to_string(),
        photo: None,
        pin: None,
        address: None,
        phone: None,
    };

    repo.create_user(ctx, super::generate_test_id().await, &user)
        .await
        .expect("Failed to create user");

    let saved_user = repo
        .get_user_by_username(ctx, "perm_user_5")
        .await
        .expect("Failed to get user")
        .expect("User not found");

    repo.save_user_permission(ctx, saved_user.id, None, 2, 3)
        .await
        .expect("Failed to save permission");

    let permissions = repo
        .get_user_permission(ctx, saved_user.id)
        .await
        .expect("Failed to get permissions");
    assert_eq!(permissions.len(), 1);

    repo.delete_user_permission(ctx, saved_user.id, None, 2)
        .await
        .expect("Failed to delete permission");

    let permissions = repo
        .get_user_permission(ctx, saved_user.id)
        .await
        .expect("Failed to get permissions");
    assert_eq!(permissions.len(), 0);
}

pub async fn user_test_delete_specific_permission_keeps_others<U: UserRepository>(
    ctx: &Context,
    repo: U,
) {
    let user = UserCreate {
        username: "perm_user_6".to_string(),
        name: "Permission User 6".to_string(),
        email: None,
        password: "pass".to_string(),
        photo: None,
        pin: None,
        address: None,
        phone: None,
    };

    repo.create_user(ctx, super::generate_test_id().await, &user)
        .await
        .expect("Failed to create user");

    let saved_user = repo
        .get_user_by_username(ctx, "perm_user_6")
        .await
        .expect("Failed to get user")
        .expect("User not found");

    repo.save_user_permission(ctx, saved_user.id, None, 2, 3)
        .await
        .expect("Failed to save permission 1");

    repo.save_user_permission(ctx, saved_user.id, None, 3, 4)
        .await
        .expect("Failed to save permission 2");

    let permissions = repo
        .get_user_permission(ctx, saved_user.id)
        .await
        .expect("Failed to get permissions");
    assert_eq!(permissions.len(), 2);

    repo.delete_user_permission(ctx, saved_user.id, None, 2)
        .await
        .expect("Failed to delete permission");

    let permissions = repo
        .get_user_permission(ctx, saved_user.id)
        .await
        .expect("Failed to get permissions");
    assert_eq!(permissions.len(), 1);
    assert_eq!(permissions[0].resource, 3);
}

pub async fn user_test_get_permission_not_found<U: UserRepository>(ctx: &Context, repo: U) {
    let user = UserCreate {
        username: "perm_user_7".to_string(),
        name: "Permission User 7".to_string(),
        email: None,
        password: "pass".to_string(),
        photo: None,
        pin: None,
        address: None,
        phone: None,
    };

    repo.create_user(ctx, super::generate_test_id().await, &user)
        .await
        .expect("Failed to create user");

    let saved_user = repo
        .get_user_by_username(ctx, "perm_user_7")
        .await
        .expect("Failed to get user")
        .expect("User not found");

    let permissions = repo
        .get_user_permission(ctx, saved_user.id)
        .await
        .expect("Failed to get permissions");

    assert_eq!(permissions.len(), 0);
}

pub async fn user_test_save_permission_null_branch_then_delete<U: UserRepository>(
    ctx: &Context,
    repo: U,
) {
    let user = UserCreate {
        username: "perm_null_branch_test".to_string(),
        name: "Permission Null Branch Test".to_string(),
        email: None,
        password: "pass".to_string(),
        photo: None,
        pin: None,
        address: None,
        phone: None,
    };

    repo.create_user(ctx, super::generate_test_id().await, &user)
        .await
        .expect("Failed to create user");

    let saved_user = repo
        .get_user_by_username(ctx, "perm_null_branch_test")
        .await
        .expect("Failed to get user")
        .expect("User not found");

    repo.save_user_permission(ctx, saved_user.id, None, 5, 10)
        .await
        .expect("Failed to save permission with NULL branch");

    let permissions = repo
        .get_user_permission(ctx, saved_user.id)
        .await
        .expect("Failed to get permissions");
    assert_eq!(permissions.len(), 1);
    assert_eq!(permissions[0].branch_id, None);

    repo.delete_user_permission(ctx, saved_user.id, None, 5)
        .await
        .expect("Failed to delete permission with NULL branch");

    let permissions = repo
        .get_user_permission(ctx, saved_user.id)
        .await
        .expect("Failed to get permissions");
    assert_eq!(permissions.len(), 0);
}

pub async fn user_test_save_and_update_permission_null_branch<U: UserRepository>(
    ctx: &Context,
    repo: U,
) {
    let user = UserCreate {
        username: "perm_update_null_branch".to_string(),
        name: "Permission Update Null Branch".to_string(),
        email: None,
        password: "pass".to_string(),
        photo: None,
        pin: None,
        address: None,
        phone: None,
    };

    repo.create_user(ctx, super::generate_test_id().await, &user)
        .await
        .expect("Failed to create user");

    let saved_user = repo
        .get_user_by_username(ctx, "perm_update_null_branch")
        .await
        .expect("Failed to get user")
        .expect("User not found");

    repo.save_user_permission(ctx, saved_user.id, None, 6, 1)
        .await
        .expect("Failed to save permission");

    let permissions = repo
        .get_user_permission(ctx, saved_user.id)
        .await
        .expect("Failed to get permissions");
    assert_eq!(permissions.len(), 1);
    assert_eq!(permissions[0].action, 1);

    repo.save_user_permission(ctx, saved_user.id, None, 6, 2)
        .await
        .expect("Failed to save permission again");

    let permissions = repo
        .get_user_permission(ctx, saved_user.id)
        .await
        .expect("Failed to get permissions");
    assert_eq!(permissions.len(), 1);
    assert_eq!(permissions[0].action, 2);
}

pub async fn user_test_delete_permission_null_vs_non_null_branch<U: UserRepository>(
    ctx: &Context,
    repo: U,
) {
    let user = UserCreate {
        username: "perm_null_vs_notnull".to_string(),
        name: "Permission Null vs Not Null".to_string(),
        email: None,
        password: "pass".to_string(),
        photo: None,
        pin: None,
        address: None,
        phone: None,
    };

    repo.create_user(ctx, super::generate_test_id().await, &user)
        .await
        .expect("Failed to create user");

    let saved_user = repo
        .get_user_by_username(ctx, "perm_null_vs_notnull")
        .await
        .expect("Failed to get user")
        .expect("User not found");

    repo.save_user_permission(ctx, saved_user.id, None, 7, 10)
        .await
        .expect("Failed to save permission with NULL branch");

    repo.save_user_permission(ctx, saved_user.id, None, 8, 20)
        .await
        .expect("Failed to save second permission with NULL branch");

    let permissions = repo
        .get_user_permission(ctx, saved_user.id)
        .await
        .expect("Failed to get permissions");
    assert_eq!(permissions.len(), 2);

    repo.delete_user_permission(ctx, saved_user.id, None, 7)
        .await
        .expect("Failed to delete permission");

    let permissions = repo
        .get_user_permission(ctx, saved_user.id)
        .await
        .expect("Failed to get permissions");
    assert_eq!(permissions.len(), 1);
    assert_eq!(permissions[0].resource, 8);
}
