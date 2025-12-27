mod common;

use sultan_core::domain::Context;
use sultan_core::domain::model::Update;
use sultan_core::domain::model::branch::{BranchCreate, BranchUpdate};
use sultan_core::snowflake::SnowflakeGenerator;
use sultan_core::storage::branch_repo::BranchRepository;
use sultan_core::storage::sqlite::branch::SqliteBranchRepository;

use common::init_sqlite_pool;

fn generate_test_id() -> i64 {
    // Use a simple incrementing ID for tests
    thread_local! {
        static GENERATOR: SnowflakeGenerator = SnowflakeGenerator::new(1).unwrap();
    }
    GENERATOR.with(|g| g.generate().unwrap())
}

#[tokio::test]
async fn test_branch_repo_integration() {
    let pool = init_sqlite_pool().await;
    let repo = SqliteBranchRepository::new(pool);
    let ctx = Context::new();

    let id = generate_test_id();
    let branch = BranchCreate {
        is_main: true,
        name: "Main Branch".to_string(),
        code: "MAIN".to_string(),
        address: Some("123 Main St".to_string()),
        phone: Some("555-1234".to_string()),
        npwp: None,
        image: None,
    };

    // Test Create
    repo.create(&ctx, id, &branch)
        .await
        .expect("Failed to create branch");

    // Test Get By ID
    let fetched_branch = repo
        .get_by_id(&ctx, id)
        .await
        .expect("Failed to get branch")
        .expect("Branch not found");
    assert_eq!(fetched_branch.name, branch.name);
    assert_eq!(fetched_branch.is_main, branch.is_main);

    // Test Update
    let update_data = BranchUpdate {
        name: Some("Updated Branch".to_string()),
        ..Default::default()
    };
    repo.update(&ctx, id, &update_data)
        .await
        .expect("Failed to update branch");

    let fetched_updated = repo
        .get_by_id(&ctx, id)
        .await
        .expect("Failed to get updated branch")
        .expect("Updated branch not found");
    assert_eq!(fetched_updated.name, "Updated Branch");

    // Test Get All
    let branches = repo
        .get_all(&ctx)
        .await
        .expect("Failed to get all branches");
    // Note: Other tests might have added branches, so we check if it contains at least our branch
    assert!(branches.iter().any(|b| b.id == id));

    // Test Delete
    repo.delete(&ctx, id)
        .await
        .expect("Failed to delete branch");
    let deleted_branch = repo
        .get_by_id(&ctx, id)
        .await
        .expect("Failed to get deleted branch");
    assert!(deleted_branch.is_none());
}

#[tokio::test]
async fn test_partial_update_branch() {
    let pool = init_sqlite_pool().await;
    let repo = SqliteBranchRepository::new(pool);
    let ctx = Context::new();

    let id = generate_test_id();
    let branch = BranchCreate {
        is_main: false,
        name: "Original Branch".to_string(),
        code: "ORIG".to_string(),
        address: Some("456 Elm St".to_string()),
        phone: Some("555-5678".to_string()),
        npwp: Some("98765432109876".to_string()),
        image: Some("original.png".to_string()),
    };

    // Create the branch
    repo.create(&ctx, id, &branch)
        .await
        .expect("Failed to create branch");

    // Partial update: only update name
    let partial_update = BranchUpdate {
        name: Some("Updated Name".to_string()),
        ..Default::default()
    };
    repo.update(&ctx, id, &partial_update)
        .await
        .expect("Failed to update branch");

    let fetched = repo
        .get_by_id(&ctx, id)
        .await
        .expect("Failed to get branch")
        .expect("Branch not found");

    // Name should be updated
    assert_eq!(fetched.name, "Updated Name");
    // Other fields should remain unchanged
    assert_eq!(fetched.code, "ORIG");
    assert_eq!(fetched.is_main, false);
    assert_eq!(fetched.address, Some("456 Elm St".to_string()));
    assert_eq!(fetched.phone, Some("555-5678".to_string()));
    assert_eq!(fetched.npwp, Some("98765432109876".to_string()));
    assert_eq!(fetched.image, Some("original.png".to_string()));

    // Partial update: only update code
    let partial_update2 = BranchUpdate {
        code: Some("NEW".to_string()),
        ..Default::default()
    };
    repo.update(&ctx, id, &partial_update2)
        .await
        .expect("Failed to update branch");

    let fetched2 = repo
        .get_by_id(&ctx, id)
        .await
        .expect("Failed to get branch")
        .expect("Branch not found");

    // Code should be updated
    assert_eq!(fetched2.name, "Updated Name"); // Should remain from previous update
    assert_eq!(fetched2.code, "NEW");
    assert_eq!(fetched2.address, Some("456 Elm St".to_string())); // Should remain unchanged
    assert_eq!(fetched2.phone, Some("555-5678".to_string())); // Should remain unchanged
    assert_eq!(fetched2.npwp, Some("98765432109876".to_string())); // Should remain unchanged
    assert_eq!(fetched2.image, Some("original.png".to_string())); // Should remain unchanged
}

#[tokio::test]
async fn test_update_non_existent_branch() {
    let pool = init_sqlite_pool().await;
    let repo = SqliteBranchRepository::new(pool);
    let ctx = Context::new();

    let update_data = BranchUpdate {
        name: Some("Non-existent".to_string()),
        ..Default::default()
    };

    let result = repo.update(&ctx, 999, &update_data).await;
    assert!(matches!(
        result,
        Err(sultan_core::domain::Error::NotFound(_))
    ));
}

#[tokio::test]
async fn test_delete_non_existent_branch() {
    let pool = init_sqlite_pool().await;
    let repo = SqliteBranchRepository::new(pool);
    let ctx = Context::new();

    let result = repo.delete(&ctx, 999).await;
    assert!(matches!(
        result,
        Err(sultan_core::domain::Error::NotFound(_))
    ));
}

#[tokio::test]
async fn test_get_deleted_branch() {
    let pool = init_sqlite_pool().await;
    let repo = SqliteBranchRepository::new(pool);
    let ctx = Context::new();

    let id = generate_test_id();
    let branch = BranchCreate {
        is_main: false,
        name: "To Delete".to_string(),
        code: "DEL".to_string(),
        address: None,
        phone: None,
        npwp: None,
        image: None,
    };

    repo.create(&ctx, id, &branch)
        .await
        .expect("Failed to create branch");
    repo.delete(&ctx, id)
        .await
        .expect("Failed to delete branch");

    let result = repo
        .get_by_id(&ctx, id)
        .await
        .expect("Failed to get branch");
    assert!(result.is_none());
}

#[tokio::test]
async fn test_get_by_id_not_found() {
    let pool = init_sqlite_pool().await;
    let repo = SqliteBranchRepository::new(pool);
    let ctx = Context::new();

    let result = repo
        .get_by_id(&ctx, 9999)
        .await
        .expect("Failed to get branch");
    assert!(result.is_none());
}

#[tokio::test]
async fn test_get_all_branches() {
    let pool = init_sqlite_pool().await;
    let repo = SqliteBranchRepository::new(pool);
    let ctx = Context::new();

    // Create multiple branches
    for i in 0..3 {
        let id = generate_test_id();
        let branch = BranchCreate {
            is_main: i == 0,
            name: format!("Branch {}", i),
            code: format!("BR{}", i),
            address: None,
            phone: None,
            npwp: None,
            image: None,
        };
        repo.create(&ctx, id, &branch)
            .await
            .expect("Failed to create branch");
    }

    let branches = repo
        .get_all(&ctx)
        .await
        .expect("Failed to get all branches");
    assert!(branches.len() >= 3);
}

#[tokio::test]
async fn test_update_branch_not_found() {
    let pool = init_sqlite_pool().await;
    let repo = SqliteBranchRepository::new(pool);
    let ctx = Context::new();

    let update_data = BranchUpdate {
        name: Some("Non-existent".to_string()),
        ..Default::default()
    };

    let result = repo.update(&ctx, 9999, &update_data).await;
    assert!(matches!(
        result,
        Err(sultan_core::domain::Error::NotFound(_))
    ));
}

#[tokio::test]
async fn test_create_branch_with_all_fields() {
    let pool = init_sqlite_pool().await;
    let repo = SqliteBranchRepository::new(pool);
    let ctx = Context::new();

    let id = generate_test_id();
    let branch = BranchCreate {
        is_main: false,
        name: "Complete Branch".to_string(),
        code: "COMP".to_string(),
        address: Some("456 Complete Ave".to_string()),
        phone: Some("555-9999".to_string()),
        npwp: Some("12345678901234".to_string()),
        image: Some("branch.png".to_string()),
    };

    repo.create(&ctx, id, &branch)
        .await
        .expect("Failed to create branch");

    let fetched = repo
        .get_by_id(&ctx, id)
        .await
        .expect("Failed to get branch")
        .expect("Branch not found");

    assert_eq!(fetched.name, "Complete Branch");
    assert_eq!(fetched.code, "COMP");
    assert_eq!(fetched.address, Some("456 Complete Ave".to_string()));
    assert_eq!(fetched.phone, Some("555-9999".to_string()));
    assert_eq!(fetched.npwp, Some("12345678901234".to_string()));
    assert_eq!(fetched.image, Some("branch.png".to_string()));
}

#[tokio::test]
async fn test_update_address_scenarios() {
    let pool = init_sqlite_pool().await;
    let repo = SqliteBranchRepository::new(pool);
    let ctx = Context::new();

    let id = generate_test_id();
    let branch = BranchCreate {
        is_main: false,
        name: "Address Test Branch".to_string(),
        code: "ADDR".to_string(),
        address: Some("123 Initial St".to_string()),
        phone: Some("555-1111".to_string()),
        npwp: None,
        image: None,
    };

    // Create the branch
    repo.create(&ctx, id, &branch)
        .await
        .expect("Failed to create branch");

    // Scenario 1: Update address with valid string value
    let update_with_value = BranchUpdate {
        address: Update::Set("456 Updated Ave".to_string()),
        ..Default::default()
    };
    repo.update(&ctx, id, &update_with_value)
        .await
        .expect("Failed to update address with value");

    let fetched1 = repo
        .get_by_id(&ctx, id)
        .await
        .expect("Failed to get branch")
        .expect("Branch not found");

    assert_eq!(
        fetched1.address,
        Some("456 Updated Ave".to_string()),
        "Address should be updated to new value"
    );
    assert_eq!(
        fetched1.phone,
        Some("555-1111".to_string()),
        "Phone should remain unchanged"
    );

    // Scenario 2: No update (Unchanged) -> keep the old value as it is
    let update_no_change = BranchUpdate {
        name: Some("Name Changed".to_string()), // Change name to prove update happened
        address: Update::Unchanged,             // Don't touch address
        ..Default::default()
    };
    repo.update(&ctx, id, &update_no_change)
        .await
        .expect("Failed to update without address change");

    let fetched2 = repo
        .get_by_id(&ctx, id)
        .await
        .expect("Failed to get branch")
        .expect("Branch not found");

    assert_eq!(fetched2.name, "Name Changed", "Name should be updated");
    assert_eq!(
        fetched2.address,
        Some("456 Updated Ave".to_string()),
        "Address should remain unchanged when update field is None"
    );

    // Scenario 3: Update address to nil/NULL value (Clear)
    let update_to_nil = BranchUpdate {
        address: Update::Clear, // Set address to NULL
        ..Default::default()
    };
    repo.update(&ctx, id, &update_to_nil)
        .await
        .expect("Failed to update address to nil");

    let fetched3 = repo
        .get_by_id(&ctx, id)
        .await
        .expect("Failed to get branch")
        .expect("Branch not found");

    assert_eq!(fetched3.address, None, "Address should be set to NULL/None");
    assert_eq!(
        fetched3.name, "Name Changed",
        "Name should remain unchanged from previous update"
    );
    assert_eq!(
        fetched3.phone,
        Some("555-1111".to_string()),
        "Phone should still remain unchanged"
    );
}
