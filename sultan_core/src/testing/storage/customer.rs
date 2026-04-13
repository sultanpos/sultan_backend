use crate::{
    domain::{
        error::Error::NotFound,
        model::{
            Update,
            customer::{CustomerCreate, CustomerFilter, CustomerQuery, CustomerUpdate},
            product::SortDirection,
        },
    },
    storage::{CustomerRepository, RepoCtx},
};
use sea_orm::DatabaseConnection;
use serde_json::json;

pub fn default_query() -> CustomerQuery {
    CustomerQuery {
        filter: CustomerFilter {
            number: None,
            name: None,
            phone: None,
            email: None,
            level: None,
        },
        sort_field: crate::domain::model::customer::CustomerSortField::Id,
        sort_direction: SortDirection::Asc,
        cursor: None,
        limit: 100,
    }
}

pub async fn customer_test_all<C, F, Fut>(repo: &C, ctx_factory: F)
where
    C: CustomerRepository,
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = RepoCtx<DatabaseConnection>>,
{
    customer_test_repo_integration(&ctx_factory().await, repo).await;
    customer_test_create_with_all_fields(&ctx_factory().await, repo).await;
    customer_test_create_minimal_fields(&ctx_factory().await, repo).await;
    customer_test_partial_update(&ctx_factory().await, repo).await;
    customer_test_update_address_scenarios(&ctx_factory().await, repo).await;
    customer_test_update_metadata(&ctx_factory().await, repo).await;
    customer_test_update_email_scenarios(&ctx_factory().await, repo).await;
    customer_test_update_level(&ctx_factory().await, repo).await;
    customer_test_update_non_existent(&ctx_factory().await, repo).await;
    customer_test_delete_non_existent(&ctx_factory().await, repo).await;
    customer_test_get_deleted(&ctx_factory().await, repo).await;
    customer_test_deleted_not_in_get_all(&ctx_factory().await, repo).await;
    customer_test_get_by_number_success(&ctx_factory().await, repo).await;
    customer_test_get_by_number_not_found(&ctx_factory().await, repo).await;
    customer_test_get_by_number_deleted(&ctx_factory().await, repo).await;
    customer_test_get_by_number_case_sensitive(&ctx_factory().await, repo).await;
    customer_test_get_by_id_not_found(&ctx_factory().await, repo).await;
    customer_test_get_all(&ctx_factory().await, repo).await;
    customer_test_filter_by_name(&ctx_factory().await, repo).await;
    customer_test_filter_by_number(&ctx_factory().await, repo).await;
    customer_test_filter_by_email(&ctx_factory().await, repo).await;
    customer_test_filter_by_phone(&ctx_factory().await, repo).await;
    customer_test_filter_by_level(&ctx_factory().await, repo).await;
    customer_test_filter_multiple_criteria(&ctx_factory().await, repo).await;
    customer_test_cursor_pagination_asc(&ctx_factory().await, repo).await;
    customer_test_cursor_pagination_desc(&ctx_factory().await, repo).await;
    customer_test_cursor_pagination_no_next(&ctx_factory().await, repo).await;
}

// =============================================================================
// Basic CRUD Tests
// =============================================================================

pub async fn customer_test_repo_integration<C: CustomerRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id = super::generate_test_id().await;
    let customer = CustomerCreate {
        number: "CUST001".to_string(),
        name: "Main Customer".to_string(),
        address: Some("123 Main St".to_string()),
        email: Some("main@customer.com".to_string()),
        phone: Some("555-1234".to_string()),
        level: 1,
        metadata: None,
    };

    // Test Create
    repo.create(ctx, id, &customer)
        .await
        .expect("Failed to create customer");

    // Test Get By ID
    let fetched_customer = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get customer")
        .expect("Customer not found");
    assert_eq!(fetched_customer.number, customer.number);
    assert_eq!(fetched_customer.name, customer.name);
    assert_eq!(fetched_customer.level, customer.level);

    // Test Update
    let update_data = CustomerUpdate {
        name: Some("Updated Customer".to_string()),
        ..Default::default()
    };
    repo.update(ctx, id, &update_data)
        .await
        .expect("Failed to update customer");

    let fetched_updated = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get updated customer")
        .expect("Updated customer not found");
    assert_eq!(fetched_updated.name, "Updated Customer");

    // Test Get All
    let page = repo
        .get_all(ctx, &default_query())
        .await
        .expect("Failed to get all customers");
    assert!(page.items.iter().any(|c| c.id == id));

    // Test Delete
    repo.delete(ctx, id)
        .await
        .expect("Failed to delete customer");
    let deleted_customer = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get deleted customer");
    assert!(deleted_customer.is_none());
}

pub async fn customer_test_create_with_all_fields<C: CustomerRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id = super::generate_test_id().await;
    let metadata = json!({
        "membership": "gold",
        "points": 1500,
        "preferences": ["email", "sms"]
    });

    let customer = CustomerCreate {
        number: "CUST999".to_string(),
        name: "Complete Customer".to_string(),
        address: Some("456 Complete Ave".to_string()),
        email: Some("complete@customer.com".to_string()),
        phone: Some("555-9999".to_string()),
        level: 3,
        metadata: Some(metadata.clone()),
    };

    repo.create(ctx, id, &customer)
        .await
        .expect("Failed to create customer");

    let fetched = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get customer")
        .expect("Customer not found");

    assert_eq!(fetched.number, "CUST999");
    assert_eq!(fetched.name, "Complete Customer");
    assert_eq!(fetched.address, Some("456 Complete Ave".to_string()));
    assert_eq!(fetched.email, Some("complete@customer.com".to_string()));
    assert_eq!(fetched.phone, Some("555-9999".to_string()));
    assert_eq!(fetched.level, 3);
    assert!(fetched.metadata.is_some());
    assert_eq!(fetched.metadata.unwrap()["membership"], "gold");
}

pub async fn customer_test_create_minimal_fields<C: CustomerRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id = super::generate_test_id().await;
    let customer = CustomerCreate {
        number: "MIN001".to_string(),
        name: "Minimal Customer".to_string(),
        address: None,
        email: None,
        phone: None,
        level: 0,
        metadata: None,
    };

    repo.create(ctx, id, &customer)
        .await
        .expect("Failed to create customer");

    let fetched = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get customer")
        .expect("Customer not found");

    assert_eq!(fetched.number, "MIN001");
    assert_eq!(fetched.name, "Minimal Customer");
    assert_eq!(fetched.address, None);
    assert_eq!(fetched.email, None);
    assert_eq!(fetched.phone, None);
    assert_eq!(fetched.level, 0);
    assert_eq!(fetched.metadata, None);
}

// =============================================================================
// Update Tests
// =============================================================================

pub async fn customer_test_partial_update<C: CustomerRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id = super::generate_test_id().await;
    let customer = CustomerCreate {
        number: "ORIG001".to_string(),
        name: "Original Customer".to_string(),
        address: Some("456 Elm St".to_string()),
        email: Some("original@customer.com".to_string()),
        phone: Some("555-5678".to_string()),
        level: 2,
        metadata: None,
    };

    // Create the customer
    repo.create(ctx, id, &customer)
        .await
        .expect("Failed to create customer");

    // Partial update: only update name
    let partial_update = CustomerUpdate {
        name: Some("Updated Name".to_string()),
        ..Default::default()
    };
    repo.update(ctx, id, &partial_update)
        .await
        .expect("Failed to update customer");

    let fetched = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get customer")
        .expect("Customer not found");

    // Name should be updated
    assert_eq!(fetched.name, "Updated Name");
    // Other fields should remain unchanged
    assert_eq!(fetched.number, "ORIG001");
    assert_eq!(fetched.email, Some("original@customer.com".to_string()));
    assert_eq!(fetched.address, Some("456 Elm St".to_string()));
    assert_eq!(fetched.phone, Some("555-5678".to_string()));
    assert_eq!(fetched.level, 2);
}

pub async fn customer_test_update_address_scenarios<C: CustomerRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id = super::generate_test_id().await;
    let customer = CustomerCreate {
        number: "ADDR001".to_string(),
        name: "Address Test Customer".to_string(),
        address: Some("123 Initial St".to_string()),
        email: Some("address@test.com".to_string()),
        phone: Some("555-1111".to_string()),
        level: 1,
        metadata: None,
    };

    // Create the customer
    repo.create(ctx, id, &customer)
        .await
        .expect("Failed to create customer");

    // Scenario 1: Update address with valid string value
    let update_with_value = CustomerUpdate {
        address: Update::Set("456 Updated Ave".to_string()),
        ..Default::default()
    };
    repo.update(ctx, id, &update_with_value)
        .await
        .expect("Failed to update address with value");

    let fetched1 = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get customer")
        .expect("Customer not found");

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
    let update_no_change = CustomerUpdate {
        name: Some("Name Changed".to_string()), // Change name to prove update happened
        address: Update::Unchanged,             // Don't touch address
        ..Default::default()
    };
    repo.update(ctx, id, &update_no_change)
        .await
        .expect("Failed to update without address change");

    let fetched2 = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get customer")
        .expect("Customer not found");

    assert_eq!(fetched2.name, "Name Changed", "Name should be updated");
    assert_eq!(
        fetched2.address,
        Some("456 Updated Ave".to_string()),
        "Address should remain unchanged when update field is Unchanged"
    );

    // Scenario 3: Update address to nil/NULL value (Clear)
    let update_to_nil = CustomerUpdate {
        address: Update::Clear, // Set address to NULL
        ..Default::default()
    };
    repo.update(ctx, id, &update_to_nil)
        .await
        .expect("Failed to update address to nil");

    let fetched3 = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get customer")
        .expect("Customer not found");

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

pub async fn customer_test_update_metadata<C: CustomerRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id = super::generate_test_id().await;
    let initial_metadata = json!({"version": 1});

    let customer = CustomerCreate {
        number: "META001".to_string(),
        name: "Metadata Test Customer".to_string(),
        address: None,
        email: None,
        phone: None,
        level: 0,
        metadata: Some(initial_metadata),
    };

    repo.create(ctx, id, &customer)
        .await
        .expect("Failed to create customer");

    // Update metadata
    let new_metadata = json!({"version": 2, "extra": "data"});
    let update_data = CustomerUpdate {
        metadata: Update::Set(new_metadata.clone()),
        ..Default::default()
    };
    repo.update(ctx, id, &update_data)
        .await
        .expect("Failed to update metadata");

    let fetched = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get customer")
        .expect("Customer not found");

    assert!(fetched.metadata.is_some());
    let meta = fetched.metadata.unwrap();
    assert_eq!(meta["version"], 2);
    assert_eq!(meta["extra"], "data");

    // Clear metadata
    let clear_update = CustomerUpdate {
        metadata: Update::Clear,
        ..Default::default()
    };
    repo.update(ctx, id, &clear_update)
        .await
        .expect("Failed to clear metadata");

    let fetched2 = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get customer")
        .expect("Customer not found");

    assert_eq!(fetched2.metadata, None);
}

pub async fn customer_test_update_email_scenarios<C: CustomerRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id = super::generate_test_id().await;
    let customer = CustomerCreate {
        number: "EMAIL001".to_string(),
        name: "Email Test Customer".to_string(),
        address: None,
        email: Some("initial@test.com".to_string()),
        phone: None,
        level: 0,
        metadata: None,
    };

    repo.create(ctx, id, &customer)
        .await
        .expect("Failed to create customer");

    // Scenario 1: Update email with Set
    let update_with_value = CustomerUpdate {
        email: Update::Set("updated@test.com".to_string()),
        ..Default::default()
    };
    repo.update(ctx, id, &update_with_value)
        .await
        .expect("Failed to update email");

    let fetched1 = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get customer")
        .expect("Customer not found");

    assert_eq!(fetched1.email, Some("updated@test.com".to_string()));

    // Scenario 2: Unchanged email
    let update_unchanged = CustomerUpdate {
        name: Some("Name Changed".to_string()),
        email: Update::Unchanged,
        ..Default::default()
    };
    repo.update(ctx, id, &update_unchanged)
        .await
        .expect("Failed to update");

    let fetched2 = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get customer")
        .expect("Customer not found");

    assert_eq!(fetched2.name, "Name Changed");
    assert_eq!(fetched2.email, Some("updated@test.com".to_string()));

    // Scenario 3: Clear email
    let update_clear = CustomerUpdate {
        email: Update::Clear,
        ..Default::default()
    };
    repo.update(ctx, id, &update_clear)
        .await
        .expect("Failed to clear email");

    let fetched3 = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get customer")
        .expect("Customer not found");

    assert_eq!(fetched3.email, None);
}

pub async fn customer_test_update_level<C: CustomerRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id = super::generate_test_id().await;
    let customer = CustomerCreate {
        number: "LVL001".to_string(),
        name: "Level Test Customer".to_string(),
        address: None,
        email: None,
        phone: None,
        level: 1,
        metadata: None,
    };

    repo.create(ctx, id, &customer)
        .await
        .expect("Failed to create customer");

    // Update level
    let update_data = CustomerUpdate {
        level: Some(5),
        ..Default::default()
    };
    repo.update(ctx, id, &update_data)
        .await
        .expect("Failed to update level");

    let fetched = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get customer")
        .expect("Customer not found");

    assert_eq!(fetched.level, 5);
}

pub async fn customer_test_update_non_existent<C: CustomerRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let update_data = CustomerUpdate {
        name: Some("Non-existent".to_string()),
        ..Default::default()
    };

    let result = repo.update(ctx, 999999, &update_data).await;
    assert!(matches!(result, Err(NotFound(_))));
}

// =============================================================================
// Delete Tests
// =============================================================================

pub async fn customer_test_delete_non_existent<C: CustomerRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let result = repo.delete(ctx, 999999).await;
    assert!(matches!(result, Err(NotFound(_))));
}

pub async fn customer_test_get_deleted<C: CustomerRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id = super::generate_test_id().await;
    let customer = CustomerCreate {
        number: "DEL001".to_string(),
        name: "To Delete".to_string(),
        address: None,
        email: None,
        phone: None,
        level: 0,
        metadata: None,
    };

    repo.create(ctx, id, &customer)
        .await
        .expect("Failed to create customer");
    repo.delete(ctx, id)
        .await
        .expect("Failed to delete customer");

    let result = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get customer");
    assert!(result.is_none());
}

pub async fn customer_test_deleted_not_in_get_all<C: CustomerRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id = super::generate_test_id().await;
    let customer = CustomerCreate {
        number: "WBD001".to_string(),
        name: "Will Be Deleted".to_string(),
        address: None,
        email: Some("deleted@test.com".to_string()),
        phone: None,
        level: 0,
        metadata: None,
    };

    repo.create(ctx, id, &customer)
        .await
        .expect("Failed to create customer");

    // Verify it appears in get_all
    let page_before = repo
        .get_all(ctx, &default_query())
        .await
        .expect("Failed to get all customers");
    assert!(page_before.items.iter().any(|c| c.id == id));

    // Delete it
    repo.delete(ctx, id)
        .await
        .expect("Failed to delete customer");

    // Verify it no longer appears in get_all
    let page_after = repo
        .get_all(ctx, &default_query())
        .await
        .expect("Failed to get all customers");
    assert!(!page_after.items.iter().any(|c| c.id == id));
}

// =============================================================================
// Get Tests
// =============================================================================

pub async fn customer_test_get_by_number_success<C: CustomerRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id = super::generate_test_id().await;
    let customer = CustomerCreate {
        number: "CUST-NUM-001".to_string(),
        name: "Customer By Number".to_string(),
        address: Some("123 Number St".to_string()),
        email: Some("number@customer.com".to_string()),
        phone: Some("555-0001".to_string()),
        level: 1,
        metadata: None,
    };

    repo.create(ctx, id, &customer)
        .await
        .expect("Failed to create customer");

    let result = repo
        .get_by_number(ctx, "CUST-NUM-001")
        .await
        .expect("Failed to get customer by number");

    assert!(result.is_some());
    let retrieved = result.unwrap();
    assert_eq!(retrieved.id, id);
    assert_eq!(retrieved.number, "CUST-NUM-001");
    assert_eq!(retrieved.name, "Customer By Number");
    assert_eq!(retrieved.email, Some("number@customer.com".to_string()));
}

pub async fn customer_test_get_by_number_not_found<C: CustomerRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let result = repo
        .get_by_number(ctx, "NONEXISTENT-NUMBER")
        .await
        .expect("Failed to get customer by number");
    assert!(result.is_none());
}

pub async fn customer_test_get_by_number_deleted<C: CustomerRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id = super::generate_test_id().await;
    let customer = CustomerCreate {
        number: "CUST-DEL-001".to_string(),
        name: "Deleted Customer".to_string(),
        address: None,
        email: None,
        phone: None,
        level: 1,
        metadata: None,
    };

    repo.create(ctx, id, &customer)
        .await
        .expect("Failed to create customer");

    repo.delete(ctx, id)
        .await
        .expect("Failed to delete customer");

    let result = repo
        .get_by_number(ctx, "CUST-DEL-001")
        .await
        .expect("Failed to get customer by number");
    assert!(result.is_none(), "Deleted customer should not be returned");
}

pub async fn customer_test_get_by_number_case_sensitive<C: CustomerRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id = super::generate_test_id().await;
    let customer = CustomerCreate {
        number: "CustNum123".to_string(),
        name: "Case Test Customer".to_string(),
        address: None,
        email: None,
        phone: None,
        level: 1,
        metadata: None,
    };

    repo.create(ctx, id, &customer)
        .await
        .expect("Failed to create customer");

    // Exact match should work
    let result = repo
        .get_by_number(ctx, "CustNum123")
        .await
        .expect("Failed to get customer by number");
    assert!(result.is_some());

    // Different case should not match (SQLite is case-sensitive for text by default)
    let result_lower = repo
        .get_by_number(ctx, "custnum123")
        .await
        .expect("Failed to get customer by number");
    assert!(
        result_lower.is_none(),
        "Should be case-sensitive and not match"
    );
}

pub async fn customer_test_get_by_id_not_found<C: CustomerRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let result = repo
        .get_by_id(ctx, 999999)
        .await
        .expect("Failed to get customer");
    assert!(result.is_none());
}

pub async fn customer_test_get_all<C: CustomerRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    // Create multiple customers
    let mut created_ids = Vec::new();
    for i in 0..3 {
        let id = super::generate_test_id().await;
        created_ids.push(id);
        let customer = CustomerCreate {
            number: format!("CUST{:03}", i),
            name: format!("Customer {}", i),
            address: None,
            email: None,
            phone: None,
            level: i,
            metadata: None,
        };
        repo.create(ctx, id, &customer)
            .await
            .expect("Failed to create customer");
    }

    let page = repo
        .get_all(ctx, &default_query())
        .await
        .expect("Failed to get all customers");

    // Verify all created customers are returned
    for id in created_ids {
        assert!(page.items.iter().any(|c| c.id == id));
    }
}

// =============================================================================
// Filter Tests
// =============================================================================

pub async fn customer_test_filter_by_name<C: CustomerRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id1 = super::generate_test_id().await;
    let id2 = super::generate_test_id().await;

    repo.create(
        ctx,
        id1,
        &CustomerCreate {
            number: "ALPHA001".to_string(),
            name: "Alpha Customer".to_string(),
            address: None,
            email: None,
            phone: None,
            level: 0,
            metadata: None,
        },
    )
    .await
    .expect("Failed to create customer");

    repo.create(
        ctx,
        id2,
        &CustomerCreate {
            number: "BETA001".to_string(),
            name: "Beta Client".to_string(),
            address: None,
            email: None,
            phone: None,
            level: 0,
            metadata: None,
        },
    )
    .await
    .expect("Failed to create customer");

    let query = CustomerQuery {
        filter: CustomerFilter {
            name: Some("Alpha".to_string()),
            number: None,
            email: None,
            phone: None,
            level: None,
        },
        ..default_query()
    };

    let page = repo
        .get_all(ctx, &query)
        .await
        .expect("Failed to get customers");

    assert!(page.items.iter().any(|c| c.id == id1));
    assert!(!page.items.iter().any(|c| c.id == id2));
}

pub async fn customer_test_filter_by_number<C: CustomerRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id1 = super::generate_test_id().await;
    let id2 = super::generate_test_id().await;

    repo.create(
        ctx,
        id1,
        &CustomerCreate {
            number: "ABC123".to_string(),
            name: "Customer One".to_string(),
            address: None,
            email: None,
            phone: None,
            level: 0,
            metadata: None,
        },
    )
    .await
    .expect("Failed to create customer");

    repo.create(
        ctx,
        id2,
        &CustomerCreate {
            number: "XYZ789".to_string(),
            name: "Customer Two".to_string(),
            address: None,
            email: None,
            phone: None,
            level: 0,
            metadata: None,
        },
    )
    .await
    .expect("Failed to create customer");

    let query = CustomerQuery {
        filter: CustomerFilter {
            name: None,
            number: Some("ABC".to_string()),
            email: None,
            phone: None,
            level: None,
        },
        ..default_query()
    };

    let page = repo
        .get_all(ctx, &query)
        .await
        .expect("Failed to get customers");

    assert!(page.items.iter().any(|c| c.id == id1));
    assert!(!page.items.iter().any(|c| c.id == id2));
}

pub async fn customer_test_filter_by_email<C: CustomerRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id1 = super::generate_test_id().await;
    let id2 = super::generate_test_id().await;

    repo.create(
        ctx,
        id1,
        &CustomerCreate {
            number: "E001".to_string(),
            name: "Customer Email1".to_string(),
            address: None,
            email: Some("alpha@company.com".to_string()),
            phone: None,
            level: 0,
            metadata: None,
        },
    )
    .await
    .expect("Failed to create customer");

    repo.create(
        ctx,
        id2,
        &CustomerCreate {
            number: "E002".to_string(),
            name: "Customer Email2".to_string(),
            address: None,
            email: Some("beta@vendor.com".to_string()),
            phone: None,
            level: 0,
            metadata: None,
        },
    )
    .await
    .expect("Failed to create customer");

    let query = CustomerQuery {
        filter: CustomerFilter {
            name: None,
            number: None,
            email: Some("alpha".to_string()),
            phone: None,
            level: None,
        },
        ..default_query()
    };

    let page = repo
        .get_all(ctx, &query)
        .await
        .expect("Failed to get customers");

    assert!(page.items.iter().any(|c| c.id == id1));
    assert!(!page.items.iter().any(|c| c.id == id2));
}

pub async fn customer_test_filter_by_phone<C: CustomerRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id1 = super::generate_test_id().await;
    let id2 = super::generate_test_id().await;

    repo.create(
        ctx,
        id1,
        &CustomerCreate {
            number: "P001".to_string(),
            name: "Customer A".to_string(),
            address: None,
            email: None,
            phone: Some("555-1234".to_string()),
            level: 0,
            metadata: None,
        },
    )
    .await
    .expect("Failed to create customer");

    repo.create(
        ctx,
        id2,
        &CustomerCreate {
            number: "P002".to_string(),
            name: "Customer B".to_string(),
            address: None,
            email: None,
            phone: Some("666-5678".to_string()),
            level: 0,
            metadata: None,
        },
    )
    .await
    .expect("Failed to create customer");

    let query = CustomerQuery {
        filter: CustomerFilter {
            name: None,
            number: None,
            email: None,
            phone: Some("555".to_string()),
            level: None,
        },
        ..default_query()
    };

    let page = repo
        .get_all(ctx, &query)
        .await
        .expect("Failed to get customers");

    assert!(page.items.iter().any(|c| c.id == id1));
    assert!(!page.items.iter().any(|c| c.id == id2));
}

pub async fn customer_test_filter_by_level<C: CustomerRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id1 = super::generate_test_id().await;
    let id2 = super::generate_test_id().await;

    repo.create(
        ctx,
        id1,
        &CustomerCreate {
            number: "LVL1".to_string(),
            name: "Customer X".to_string(),
            address: None,
            email: None,
            phone: None,
            level: 1,
            metadata: None,
        },
    )
    .await
    .expect("Failed to create customer");

    repo.create(
        ctx,
        id2,
        &CustomerCreate {
            number: "LVL2".to_string(),
            name: "Customer Y".to_string(),
            address: None,
            email: None,
            phone: None,
            level: 3,
            metadata: None,
        },
    )
    .await
    .expect("Failed to create customer");

    let query = CustomerQuery {
        filter: CustomerFilter {
            name: None,
            number: None,
            email: None,
            phone: None,
            level: Some(1),
        },
        ..default_query()
    };

    let page = repo
        .get_all(ctx, &query)
        .await
        .expect("Failed to get customers");

    assert!(page.items.iter().any(|c| c.id == id1));
    assert!(!page.items.iter().any(|c| c.id == id2));
}

pub async fn customer_test_filter_multiple_criteria<C: CustomerRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let id1 = super::generate_test_id().await;
    let id2 = super::generate_test_id().await;
    let id3 = super::generate_test_id().await;

    repo.create(
        ctx,
        id1,
        &CustomerCreate {
            number: "ALP001".to_string(),
            name: "Alpha Corp".to_string(),
            address: None,
            email: Some("alpha@corp.com".to_string()),
            phone: None,
            level: 1,
            metadata: None,
        },
    )
    .await
    .unwrap();

    repo.create(
        ctx,
        id2,
        &CustomerCreate {
            number: "BET002".to_string(),
            name: "Alpha Inc".to_string(),
            address: None,
            email: Some("alpha@corp.com".to_string()),
            phone: None,
            level: 2,
            metadata: None,
        },
    )
    .await
    .unwrap();

    repo.create(
        ctx,
        id3,
        &CustomerCreate {
            number: "ALP003".to_string(),
            name: "Beta Corp".to_string(),
            address: None,
            email: Some("beta@corp.com".to_string()),
            phone: None,
            level: 1,
            metadata: None,
        },
    )
    .await
    .unwrap();

    // Filter by name, number, and level
    let query = CustomerQuery {
        filter: CustomerFilter {
            name: Some("Alpha".to_string()),
            number: Some("ALP".to_string()),
            email: None,
            phone: None,
            level: Some(1),
        },
        ..default_query()
    };

    let page = repo
        .get_all(ctx, &query)
        .await
        .expect("Failed to get customers");

    // Only id1 matches all criteria
    assert!(page.items.iter().any(|c| c.id == id1));
    assert!(!page.items.iter().any(|c| c.id == id2)); // Has Alpha name but BET number and level 2
    assert!(!page.items.iter().any(|c| c.id == id3)); // Has ALP number and level 1 but Beta name
}

// =============================================================================
// Cursor Pagination Tests
// =============================================================================

pub async fn customer_test_cursor_pagination_asc<C: CustomerRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    // Create 5 customers
    for i in 0..5 {
        let id = super::generate_test_id().await;
        let customer = CustomerCreate {
            number: format!("PAG{:03}", i),
            name: format!("Paginated Customer {}", i),
            address: None,
            email: None,
            phone: None,
            level: 0,
            metadata: None,
        };
        repo.create(ctx, id, &customer)
            .await
            .expect("Failed to create customer");
    }

    // Get first page (2 items), sorted by id asc
    let query_page1 = CustomerQuery {
        sort_field: crate::domain::model::customer::CustomerSortField::Id,
        sort_direction: SortDirection::Asc,
        cursor: None,
        limit: 2,
        ..default_query()
    };
    let page1 = repo
        .get_all(ctx, &query_page1)
        .await
        .expect("Failed to get page 1");
    assert_eq!(page1.items.len(), 2);
    assert!(page1.next_cursor.is_some());

    // Get second page using the cursor
    let query_page2 = CustomerQuery {
        sort_field: crate::domain::model::customer::CustomerSortField::Id,
        sort_direction: SortDirection::Asc,
        cursor: page1.next_cursor.clone(),
        limit: 2,
        ..default_query()
    };
    let page2 = repo
        .get_all(ctx, &query_page2)
        .await
        .expect("Failed to get page 2");
    assert_eq!(page2.items.len(), 2);

    // Verify pages don't overlap
    for c1 in &page1.items {
        assert!(!page2.items.iter().any(|c2| c2.id == c1.id));
    }
}

pub async fn customer_test_cursor_pagination_desc<C: CustomerRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    // Create 3 customers
    for i in 0..3 {
        let id = super::generate_test_id().await;
        let customer = CustomerCreate {
            number: format!("DESC{:03}", i),
            name: format!("Desc Customer {}", i),
            address: None,
            email: None,
            phone: None,
            level: 0,
            metadata: None,
        };
        repo.create(ctx, id, &customer)
            .await
            .expect("Failed to create customer");
    }

    // Get first page in descending order by name
    let query_page1 = CustomerQuery {
        sort_field: crate::domain::model::customer::CustomerSortField::Name,
        sort_direction: SortDirection::Desc,
        cursor: None,
        limit: 2,
        ..default_query()
    };
    let page1 = repo
        .get_all(ctx, &query_page1)
        .await
        .expect("Failed to get page 1 desc");
    assert_eq!(page1.items.len(), 2);
    assert!(page1.next_cursor.is_some());

    // Get second page using the cursor
    let query_page2 = CustomerQuery {
        sort_field: crate::domain::model::customer::CustomerSortField::Name,
        sort_direction: SortDirection::Desc,
        cursor: page1.next_cursor.clone(),
        limit: 2,
        ..default_query()
    };
    let page2 = repo
        .get_all(ctx, &query_page2)
        .await
        .expect("Failed to get page 2 desc");
    assert_eq!(page2.items.len(), 1);
    assert!(page2.next_cursor.is_none());

    // Verify pages don't overlap
    for c1 in &page1.items {
        assert!(!page2.items.iter().any(|c2| c2.id == c1.id));
    }
}

pub async fn customer_test_cursor_pagination_no_next<C: CustomerRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    // Create exactly 2 customers
    for i in 0..2 {
        let id = super::generate_test_id().await;
        let customer = CustomerCreate {
            number: format!("NONEXT{:03}", i),
            name: format!("No Next Customer {}", i),
            address: None,
            email: None,
            phone: None,
            level: 0,
            metadata: None,
        };
        repo.create(ctx, id, &customer)
            .await
            .expect("Failed to create customer");
    }

    // Get page of 5, only 2 exist — no next cursor
    let query = CustomerQuery {
        sort_field: crate::domain::model::customer::CustomerSortField::UpdatedAt,
        sort_direction: SortDirection::Asc,
        cursor: None,
        limit: 5,
        ..default_query()
    };
    let page = repo.get_all(ctx, &query).await.expect("Failed to get page");
    assert_eq!(page.items.len(), 2);
    assert!(page.next_cursor.is_none());
}
