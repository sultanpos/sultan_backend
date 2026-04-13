use crate::{
    domain::{
        error::Error::NotFound,
        model::{
            Update,
            product::SortDirection,
            supplier::{
                SupplierCreate, SupplierFilter, SupplierQuery, SupplierSortField, SupplierUpdate,
            },
        },
    },
    storage::{RepoCtx, SupplierRepository},
};
use sea_orm::DatabaseConnection;
use serde_json::json;

pub fn default_query() -> SupplierQuery {
    SupplierQuery {
        filter: SupplierFilter::default(),
        sort_field: SupplierSortField::Id,
        sort_direction: SortDirection::Asc,
        cursor: None,
        limit: 100,
    }
}

/// Run all supplier repository tests using a context factory that creates fresh contexts.
pub async fn supplier_test_all<S, F, Fut>(repo: &S, ctx_factory: F)
where
    S: SupplierRepository,
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = RepoCtx<DatabaseConnection>>,
{
    supplier_test_repo_integration(&ctx_factory().await, repo).await;
    supplier_test_create_with_all_fields(&ctx_factory().await, repo).await;
    supplier_test_create_minimal_fields(&ctx_factory().await, repo).await;
    supplier_test_partial_update(&ctx_factory().await, repo).await;
    supplier_test_update_address_scenarios(&ctx_factory().await, repo).await;
    supplier_test_update_metadata(&ctx_factory().await, repo).await;
    supplier_test_update_email_scenarios(&ctx_factory().await, repo).await;
    supplier_test_update_non_existent(&ctx_factory().await, repo).await;
    supplier_test_delete_non_existent(&ctx_factory().await, repo).await;
    supplier_test_get_deleted(&ctx_factory().await, repo).await;
    supplier_test_deleted_not_in_get_all(&ctx_factory().await, repo).await;
    supplier_test_get_by_id_not_found(&ctx_factory().await, repo).await;
    supplier_test_get_all(&ctx_factory().await, repo).await;
    supplier_test_filter_by_name(&ctx_factory().await, repo).await;
    supplier_test_filter_by_code(&ctx_factory().await, repo).await;
    supplier_test_filter_by_email(&ctx_factory().await, repo).await;
    supplier_test_filter_by_phone(&ctx_factory().await, repo).await;
    supplier_test_filter_by_npwp(&ctx_factory().await, repo).await;
    supplier_test_filter_multiple_criteria(&ctx_factory().await, repo).await;
    supplier_test_cursor_pagination_asc(&ctx_factory().await, repo).await;
    supplier_test_cursor_pagination_desc(&ctx_factory().await, repo).await;
    supplier_test_cursor_pagination_no_next(&ctx_factory().await, repo).await;
}

// =============================================================================
// Basic CRUD Tests
// =============================================================================

pub async fn supplier_test_repo_integration<S: SupplierRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    let id = super::generate_test_id().await;
    let supplier = SupplierCreate {
        name: "Main Supplier".to_string(),
        code: Some("MAIN".to_string()),
        address: Some("123 Main St".to_string()),
        phone: Some("555-1234".to_string()),
        npwp: None,
        npwp_name: None,
        email: None,
        metadata: None,
    };

    // Test Create
    repo.create(ctx, id, &supplier)
        .await
        .expect("Failed to create supplier");

    // Test Get By ID
    let fetched_supplier = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get supplier")
        .expect("Supplier not found");
    assert_eq!(fetched_supplier.name, supplier.name);
    assert_eq!(fetched_supplier.code, supplier.code);

    // Test Update
    let update_data = SupplierUpdate {
        name: Some("Updated Supplier".to_string()),
        ..Default::default()
    };
    repo.update(ctx, id, &update_data)
        .await
        .expect("Failed to update supplier");

    let fetched_updated = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get updated supplier")
        .expect("Updated supplier not found");
    assert_eq!(fetched_updated.name, "Updated Supplier");

    // Test Get All
    let suppliers = repo
        .get_all(ctx, &default_query())
        .await
        .expect("Failed to get all suppliers");
    assert!(suppliers.items.iter().any(|s| s.id == id));

    // Test Delete
    repo.delete(ctx, id)
        .await
        .expect("Failed to delete supplier");
    let deleted_supplier = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get deleted supplier");
    assert!(deleted_supplier.is_none());
}

pub async fn supplier_test_create_with_all_fields<S: SupplierRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    let id = super::generate_test_id().await;
    let metadata = json!({
        "contact_person": "John Doe",
        "rating": 4.5,
        "tags": ["reliable", "fast"]
    });

    let supplier = SupplierCreate {
        name: "Complete Supplier".to_string(),
        code: Some("COMP".to_string()),
        email: Some("complete@supplier.com".to_string()),
        address: Some("456 Complete Ave".to_string()),
        phone: Some("555-9999".to_string()),
        npwp: Some("12345678901234".to_string()),
        npwp_name: Some("PT Complete Supplier".to_string()),
        metadata: Some(metadata.clone()),
    };

    repo.create(ctx, id, &supplier)
        .await
        .expect("Failed to create supplier");

    let fetched = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get supplier")
        .expect("Supplier not found");

    assert_eq!(fetched.name, "Complete Supplier");
    assert_eq!(fetched.code, Some("COMP".to_string()));
    assert_eq!(fetched.email, Some("complete@supplier.com".to_string()));
    assert_eq!(fetched.address, Some("456 Complete Ave".to_string()));
    assert_eq!(fetched.phone, Some("555-9999".to_string()));
    assert_eq!(fetched.npwp, Some("12345678901234".to_string()));
    assert_eq!(fetched.npwp_name, Some("PT Complete Supplier".to_string()));
    assert!(fetched.metadata.is_some());
    assert_eq!(fetched.metadata.unwrap()["contact_person"], "John Doe");
}

pub async fn supplier_test_create_minimal_fields<S: SupplierRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    let id = super::generate_test_id().await;
    let supplier = SupplierCreate {
        name: "Minimal Supplier".to_string(),
        code: None,
        email: None,
        address: None,
        phone: None,
        npwp: None,
        npwp_name: None,
        metadata: None,
    };

    repo.create(ctx, id, &supplier)
        .await
        .expect("Failed to create supplier");

    let fetched = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get supplier")
        .expect("Supplier not found");

    assert_eq!(fetched.name, "Minimal Supplier");
    assert_eq!(fetched.code, None);
    assert_eq!(fetched.email, None);
    assert_eq!(fetched.address, None);
    assert_eq!(fetched.phone, None);
    assert_eq!(fetched.npwp, None);
    assert_eq!(fetched.npwp_name, None);
    assert_eq!(fetched.metadata, None);
}

// =============================================================================
// Update Tests
// =============================================================================

pub async fn supplier_test_partial_update<S: SupplierRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    let id = super::generate_test_id().await;
    let supplier = SupplierCreate {
        name: "Original Supplier".to_string(),
        code: Some("ORIG".to_string()),
        email: Some("original@supplier.com".to_string()),
        address: Some("456 Elm St".to_string()),
        phone: Some("555-5678".to_string()),
        npwp: Some("98765432109876".to_string()),
        npwp_name: Some("PT Original".to_string()),
        metadata: None,
    };

    // Create the supplier
    repo.create(ctx, id, &supplier)
        .await
        .expect("Failed to create supplier");

    // Partial update: only update name
    let partial_update = SupplierUpdate {
        name: Some("Updated Name".to_string()),
        ..Default::default()
    };
    repo.update(ctx, id, &partial_update)
        .await
        .expect("Failed to update supplier");

    let fetched = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get supplier")
        .expect("Supplier not found");

    // Name should be updated
    assert_eq!(fetched.name, "Updated Name");
    // Other fields should remain unchanged
    assert_eq!(fetched.code, Some("ORIG".to_string()));
    assert_eq!(fetched.email, Some("original@supplier.com".to_string()));
    assert_eq!(fetched.address, Some("456 Elm St".to_string()));
    assert_eq!(fetched.phone, Some("555-5678".to_string()));
    assert_eq!(fetched.npwp, Some("98765432109876".to_string()));
    assert_eq!(fetched.npwp_name, Some("PT Original".to_string()));
}

pub async fn supplier_test_update_address_scenarios<S: SupplierRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    let id = super::generate_test_id().await;
    let supplier = SupplierCreate {
        name: "Address Test Supplier".to_string(),
        code: Some("ADDR".to_string()),
        email: Some("address@test.com".to_string()),
        address: Some("123 Initial St".to_string()),
        phone: Some("555-1111".to_string()),
        npwp: None,
        npwp_name: None,
        metadata: None,
    };

    // Create the supplier
    repo.create(ctx, id, &supplier)
        .await
        .expect("Failed to create supplier");

    // Scenario 1: Update address with valid string value
    let update_with_value = SupplierUpdate {
        address: Update::Set("456 Updated Ave".to_string()),
        ..Default::default()
    };
    repo.update(ctx, id, &update_with_value)
        .await
        .expect("Failed to update address with value");

    let fetched1 = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get supplier")
        .expect("Supplier not found");

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
    let update_no_change = SupplierUpdate {
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
        .expect("Failed to get supplier")
        .expect("Supplier not found");

    assert_eq!(fetched2.name, "Name Changed", "Name should be updated");
    assert_eq!(
        fetched2.address,
        Some("456 Updated Ave".to_string()),
        "Address should remain unchanged when update field is Unchanged"
    );

    // Scenario 3: Update address to nil/NULL value (Clear)
    let update_to_nil = SupplierUpdate {
        address: Update::Clear, // Set address to NULL
        ..Default::default()
    };
    repo.update(ctx, id, &update_to_nil)
        .await
        .expect("Failed to update address to nil");

    let fetched3 = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get supplier")
        .expect("Supplier not found");

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

pub async fn supplier_test_update_metadata<S: SupplierRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    let id = super::generate_test_id().await;
    let initial_metadata = json!({"version": 1});

    let supplier = SupplierCreate {
        name: "Metadata Test Supplier".to_string(),
        code: None,
        email: None,
        address: None,
        phone: None,
        npwp: None,
        npwp_name: None,
        metadata: Some(initial_metadata),
    };

    repo.create(ctx, id, &supplier)
        .await
        .expect("Failed to create supplier");

    // Update metadata
    let new_metadata = json!({"version": 2, "extra": "data"});
    let update_data = SupplierUpdate {
        metadata: Update::Set(new_metadata.clone()),
        ..Default::default()
    };
    repo.update(ctx, id, &update_data)
        .await
        .expect("Failed to update metadata");

    let fetched = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get supplier")
        .expect("Supplier not found");

    assert!(fetched.metadata.is_some());
    let meta = fetched.metadata.unwrap();
    assert_eq!(meta["version"], 2);
    assert_eq!(meta["extra"], "data");

    // Clear metadata
    let clear_update = SupplierUpdate {
        metadata: Update::Clear,
        ..Default::default()
    };
    repo.update(ctx, id, &clear_update)
        .await
        .expect("Failed to clear metadata");

    let fetched2 = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get supplier")
        .expect("Supplier not found");

    assert_eq!(fetched2.metadata, None);
}

pub async fn supplier_test_update_email_scenarios<S: SupplierRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    let id = super::generate_test_id().await;
    let supplier = SupplierCreate {
        name: "Email Test Supplier".to_string(),
        code: None,
        email: Some("initial@test.com".to_string()),
        address: None,
        phone: None,
        npwp: None,
        npwp_name: None,
        metadata: None,
    };

    repo.create(ctx, id, &supplier)
        .await
        .expect("Failed to create supplier");

    // Scenario 1: Update email with Set
    let update_with_value = SupplierUpdate {
        email: Update::Set("updated@test.com".to_string()),
        ..Default::default()
    };
    repo.update(ctx, id, &update_with_value)
        .await
        .expect("Failed to update email");

    let fetched1 = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get supplier")
        .expect("Supplier not found");

    assert_eq!(fetched1.email, Some("updated@test.com".to_string()));

    // Scenario 2: Unchanged email
    let update_unchanged = SupplierUpdate {
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
        .expect("Failed to get supplier")
        .expect("Supplier not found");

    assert_eq!(fetched2.name, "Name Changed");
    assert_eq!(fetched2.email, Some("updated@test.com".to_string()));

    // Scenario 3: Clear email
    let update_clear = SupplierUpdate {
        email: Update::Clear,
        ..Default::default()
    };
    repo.update(ctx, id, &update_clear)
        .await
        .expect("Failed to clear email");

    let fetched3 = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get supplier")
        .expect("Supplier not found");

    assert_eq!(fetched3.email, None);
}

pub async fn supplier_test_update_non_existent<S: SupplierRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    let update_data = SupplierUpdate {
        name: Some("Non-existent".to_string()),
        ..Default::default()
    };

    let result = repo.update(ctx, 999999, &update_data).await;
    assert!(matches!(result, Err(NotFound(_))));
}

// =============================================================================
// Delete Tests
// =============================================================================

pub async fn supplier_test_delete_non_existent<S: SupplierRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    let result = repo.delete(ctx, 999999).await;
    assert!(matches!(result, Err(NotFound(_))));
}

pub async fn supplier_test_get_deleted<S: SupplierRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    let id = super::generate_test_id().await;
    let supplier = SupplierCreate {
        name: "To Delete".to_string(),
        code: Some("DEL".to_string()),
        email: None,
        address: None,
        phone: None,
        npwp: None,
        npwp_name: None,
        metadata: None,
    };

    repo.create(ctx, id, &supplier)
        .await
        .expect("Failed to create supplier");
    repo.delete(ctx, id)
        .await
        .expect("Failed to delete supplier");

    let result = repo
        .get_by_id(ctx, id)
        .await
        .expect("Failed to get supplier");
    assert!(result.is_none());
}

pub async fn supplier_test_deleted_not_in_get_all<S: SupplierRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    let id = super::generate_test_id().await;
    let supplier = SupplierCreate {
        name: "Will Be Deleted".to_string(),
        code: Some("WBD".to_string()),
        email: Some("address@test.com".to_string()),
        address: None,
        phone: None,
        npwp: None,
        npwp_name: None,
        metadata: None,
    };

    repo.create(ctx, id, &supplier)
        .await
        .expect("Failed to create supplier");

    // Verify it appears in get_all
    let suppliers_before = repo
        .get_all(ctx, &default_query())
        .await
        .expect("Failed to get all suppliers");
    assert!(suppliers_before.items.iter().any(|s| s.id == id));

    // Delete it
    repo.delete(ctx, id)
        .await
        .expect("Failed to delete supplier");

    // Verify it no longer appears in get_all
    let suppliers_after = repo
        .get_all(ctx, &default_query())
        .await
        .expect("Failed to get all suppliers");
    assert!(!suppliers_after.items.iter().any(|s| s.id == id));
}

// =============================================================================
// Get Tests
// =============================================================================

pub async fn supplier_test_get_by_id_not_found<S: SupplierRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    let result = repo
        .get_by_id(ctx, 999999)
        .await
        .expect("Failed to get supplier");
    assert!(result.is_none());
}

pub async fn supplier_test_get_all<S: SupplierRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    // Create multiple suppliers
    let mut created_ids = Vec::new();
    for i in 0..3 {
        let id = super::generate_test_id().await;
        created_ids.push(id);
        let supplier = SupplierCreate {
            name: format!("Supplier {}", i),
            code: Some(format!("SUP{}", i)),
            email: None,
            address: None,
            phone: None,
            npwp: None,
            npwp_name: None,
            metadata: None,
        };
        repo.create(ctx, id, &supplier)
            .await
            .expect("Failed to create supplier");
    }

    let suppliers = repo
        .get_all(ctx, &default_query())
        .await
        .expect("Failed to get all suppliers");

    // Verify all created suppliers are returned
    for id in created_ids {
        assert!(suppliers.items.iter().any(|s| s.id == id));
    }
}

// =============================================================================
// Filter Tests
// =============================================================================

pub async fn supplier_test_filter_by_name<S: SupplierRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    let id1 = super::generate_test_id().await;
    let id2 = super::generate_test_id().await;

    repo.create(
        ctx,
        id1,
        &SupplierCreate {
            name: "Alpha Supplier".to_string(),
            code: None,
            email: None,
            address: None,
            phone: None,
            npwp: None,
            npwp_name: None,
            metadata: None,
        },
    )
    .await
    .expect("Failed to create supplier");

    repo.create(
        ctx,
        id2,
        &SupplierCreate {
            name: "Beta Vendor".to_string(),
            code: None,
            email: None,
            address: None,
            phone: None,
            npwp: None,
            npwp_name: None,
            metadata: None,
        },
    )
    .await
    .expect("Failed to create supplier");

    let query = SupplierQuery {
        filter: SupplierFilter {
            name: Some("Alpha".to_string()),
            ..SupplierFilter::default()
        },
        ..default_query()
    };

    let suppliers = repo
        .get_all(ctx, &query)
        .await
        .expect("Failed to get suppliers");

    assert!(suppliers.items.iter().any(|s| s.id == id1));
    assert!(!suppliers.items.iter().any(|s| s.id == id2));
}

pub async fn supplier_test_filter_by_code<S: SupplierRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    let id1 = super::generate_test_id().await;
    let id2 = super::generate_test_id().await;

    repo.create(
        ctx,
        id1,
        &SupplierCreate {
            name: "Supplier One".to_string(),
            code: Some("ABC123".to_string()),
            email: None,
            address: None,
            phone: None,
            npwp: None,
            npwp_name: None,
            metadata: None,
        },
    )
    .await
    .expect("Failed to create supplier");

    repo.create(
        ctx,
        id2,
        &SupplierCreate {
            name: "Supplier Two".to_string(),
            code: Some("XYZ789".to_string()),
            email: None,
            address: None,
            phone: None,
            npwp: None,
            npwp_name: None,
            metadata: None,
        },
    )
    .await
    .expect("Failed to create supplier");

    let query = SupplierQuery {
        filter: SupplierFilter {
            code: Some("ABC".to_string()),
            ..SupplierFilter::default()
        },
        ..default_query()
    };

    let suppliers = repo
        .get_all(ctx, &query)
        .await
        .expect("Failed to get suppliers");

    assert!(suppliers.items.iter().any(|s| s.id == id1));
    assert!(!suppliers.items.iter().any(|s| s.id == id2));
}

pub async fn supplier_test_filter_by_email<S: SupplierRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    let id1 = super::generate_test_id().await;
    let id2 = super::generate_test_id().await;

    repo.create(
        ctx,
        id1,
        &SupplierCreate {
            name: "Supplier Email1".to_string(),
            code: None,
            email: Some("alpha@company.com".to_string()),
            address: None,
            phone: None,
            npwp: None,
            npwp_name: None,
            metadata: None,
        },
    )
    .await
    .expect("Failed to create supplier");

    repo.create(
        ctx,
        id2,
        &SupplierCreate {
            name: "Supplier Email2".to_string(),
            code: None,
            email: Some("beta@vendor.com".to_string()),
            address: None,
            phone: None,
            npwp: None,
            npwp_name: None,
            metadata: None,
        },
    )
    .await
    .expect("Failed to create supplier");

    let query = SupplierQuery {
        filter: SupplierFilter {
            email: Some("alpha".to_string()),
            ..SupplierFilter::default()
        },
        ..default_query()
    };

    let suppliers = repo
        .get_all(ctx, &query)
        .await
        .expect("Failed to get suppliers");

    assert!(suppliers.items.iter().any(|s| s.id == id1));
    assert!(!suppliers.items.iter().any(|s| s.id == id2));
}

pub async fn supplier_test_filter_by_phone<S: SupplierRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    let id1 = super::generate_test_id().await;
    let id2 = super::generate_test_id().await;

    repo.create(
        ctx,
        id1,
        &SupplierCreate {
            name: "Supplier A".to_string(),
            code: None,
            email: None,
            address: None,
            phone: Some("555-1234".to_string()),
            npwp: None,
            npwp_name: None,
            metadata: None,
        },
    )
    .await
    .expect("Failed to create supplier");

    repo.create(
        ctx,
        id2,
        &SupplierCreate {
            name: "Supplier B".to_string(),
            code: None,
            email: None,
            address: None,
            phone: Some("666-5678".to_string()),
            npwp: None,
            npwp_name: None,
            metadata: None,
        },
    )
    .await
    .expect("Failed to create supplier");

    let query = SupplierQuery {
        filter: SupplierFilter {
            phone: Some("555".to_string()),
            ..SupplierFilter::default()
        },
        ..default_query()
    };

    let suppliers = repo
        .get_all(ctx, &query)
        .await
        .expect("Failed to get suppliers");

    assert!(suppliers.items.iter().any(|s| s.id == id1));
    assert!(!suppliers.items.iter().any(|s| s.id == id2));
}

pub async fn supplier_test_filter_by_npwp<S: SupplierRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    let id1 = super::generate_test_id().await;
    let id2 = super::generate_test_id().await;

    repo.create(
        ctx,
        id1,
        &SupplierCreate {
            name: "Supplier X".to_string(),
            code: None,
            email: None,
            address: None,
            phone: None,
            npwp: Some("12345678901234".to_string()),
            npwp_name: None,
            metadata: None,
        },
    )
    .await
    .expect("Failed to create supplier");

    repo.create(
        ctx,
        id2,
        &SupplierCreate {
            name: "Supplier Y".to_string(),
            code: None,
            email: None,
            address: None,
            phone: None,
            npwp: Some("98765432109876".to_string()),
            npwp_name: None,
            metadata: None,
        },
    )
    .await
    .expect("Failed to create supplier");

    let query = SupplierQuery {
        filter: SupplierFilter {
            npwp: Some("1234".to_string()),
            ..SupplierFilter::default()
        },
        ..default_query()
    };

    let suppliers = repo
        .get_all(ctx, &query)
        .await
        .expect("Failed to get suppliers");

    assert!(suppliers.items.iter().any(|s| s.id == id1));
    assert!(!suppliers.items.iter().any(|s| s.id == id2));
}

pub async fn supplier_test_filter_multiple_criteria<S: SupplierRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    let id1 = super::generate_test_id().await;
    let id2 = super::generate_test_id().await;
    let id3 = super::generate_test_id().await;

    repo.create(
        ctx,
        id1,
        &SupplierCreate {
            name: "Alpha Corp".to_string(),
            code: Some("ALP001".to_string()),
            email: Some("alpha@corp.com".to_string()),
            address: None,
            phone: None,
            npwp: None,
            npwp_name: None,
            metadata: None,
        },
    )
    .await
    .unwrap();

    repo.create(
        ctx,
        id2,
        &SupplierCreate {
            name: "Alpha Inc".to_string(),
            code: Some("BET002".to_string()),
            email: Some("alpha@corp.com".to_string()),
            address: None,
            phone: None,
            npwp: None,
            npwp_name: None,
            metadata: None,
        },
    )
    .await
    .unwrap();

    repo.create(
        ctx,
        id3,
        &SupplierCreate {
            name: "Beta Corp".to_string(),
            code: Some("ALP003".to_string()),
            email: Some("beta@corp.com".to_string()),
            address: None,
            phone: None,
            npwp: None,
            npwp_name: None,
            metadata: None,
        },
    )
    .await
    .unwrap();

    // Filter by both name and code
    let query = SupplierQuery {
        filter: SupplierFilter {
            name: Some("Alpha".to_string()),
            code: Some("ALP".to_string()),
            ..SupplierFilter::default()
        },
        ..default_query()
    };

    let suppliers = repo
        .get_all(ctx, &query)
        .await
        .expect("Failed to get suppliers");

    // Only id1 matches both criteria
    assert!(suppliers.items.iter().any(|s| s.id == id1));
    assert!(!suppliers.items.iter().any(|s| s.id == id2)); // Has Alpha name but BET code
    assert!(!suppliers.items.iter().any(|s| s.id == id3)); // Has ALP code but Beta name
}

// =============================================================================
// Cursor Pagination Tests
// =============================================================================

pub async fn supplier_test_cursor_pagination_asc<S: SupplierRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    // Create 5 suppliers
    for i in 0..5 {
        let id = super::generate_test_id().await;
        let supplier = SupplierCreate {
            name: format!("Paginated Supplier {}", i),
            code: Some(format!("PAG{}", i)),
            email: None,
            address: None,
            phone: None,
            npwp: None,
            npwp_name: None,
            metadata: None,
        };
        repo.create(ctx, id, &supplier)
            .await
            .expect("Failed to create supplier");
    }

    // Get first page (2 items, ascending by id)
    let page1 = repo
        .get_all(
            ctx,
            &SupplierQuery {
                limit: 2,
                ..default_query()
            },
        )
        .await
        .expect("Failed to get page 1");
    assert_eq!(page1.items.len(), 2);
    assert!(
        page1.next_cursor.is_some(),
        "Expected a next cursor for page 1"
    );

    // Get second page using cursor
    let cursor = page1.next_cursor.unwrap();
    let page2 = repo
        .get_all(
            ctx,
            &SupplierQuery {
                limit: 2,
                cursor: Some(cursor),
                ..default_query()
            },
        )
        .await
        .expect("Failed to get page 2");
    assert_eq!(page2.items.len(), 2);

    // Verify pages don't overlap
    for s1 in &page1.items {
        assert!(!page2.items.iter().any(|s2| s2.id == s1.id));
    }
}

pub async fn supplier_test_cursor_pagination_desc<S: SupplierRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    // Create 4 suppliers
    for i in 0..4 {
        let id = super::generate_test_id().await;
        let supplier = SupplierCreate {
            name: format!("Desc Supplier {}", i),
            code: None,
            email: None,
            address: None,
            phone: None,
            npwp: None,
            npwp_name: None,
            metadata: None,
        };
        repo.create(ctx, id, &supplier)
            .await
            .expect("Failed to create supplier");
    }

    // Get first page descending
    let page1 = repo
        .get_all(
            ctx,
            &SupplierQuery {
                limit: 2,
                sort_direction: SortDirection::Desc,
                ..default_query()
            },
        )
        .await
        .expect("Failed to get page 1");
    assert_eq!(page1.items.len(), 2);
    assert!(page1.next_cursor.is_some());

    // IDs should be in descending order
    assert!(page1.items[0].id > page1.items[1].id);

    // Get second page using cursor
    let cursor = page1.next_cursor.unwrap();
    let page2 = repo
        .get_all(
            ctx,
            &SupplierQuery {
                limit: 2,
                sort_direction: SortDirection::Desc,
                cursor: Some(cursor),
                ..default_query()
            },
        )
        .await
        .expect("Failed to get page 2");
    assert_eq!(page2.items.len(), 2);

    // All items in page2 should have lower IDs than items in page1
    for s1 in &page1.items {
        for s2 in &page2.items {
            assert!(
                s1.id > s2.id,
                "Page 1 IDs should be > page 2 IDs in desc order"
            );
        }
    }
}

pub async fn supplier_test_cursor_pagination_no_next<S: SupplierRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &S,
) {
    // Create exactly 3 suppliers
    for i in 0..3 {
        let id = super::generate_test_id().await;
        let supplier = SupplierCreate {
            name: format!("NoNext Supplier {}", i),
            code: None,
            email: None,
            address: None,
            phone: None,
            npwp: None,
            npwp_name: None,
            metadata: None,
        };
        repo.create(ctx, id, &supplier)
            .await
            .expect("Failed to create supplier");
    }

    // Fetch all 3 at once — no next cursor expected
    let page = repo
        .get_all(
            ctx,
            &SupplierQuery {
                limit: 10,
                ..default_query()
            },
        )
        .await
        .expect("Failed to get page");
    assert!(
        page.next_cursor.is_none(),
        "Expected no next cursor when all items fit in one page"
    );
    assert!(page.items.len() >= 3);
}
