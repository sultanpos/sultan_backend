use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};

use crate::{
    domain::model::{
        product::SortDirection,
        purchase_order::{
            PaymentStatus, PurchaseOrderCreate, PurchaseOrderFilter, PurchaseOrderItemCreate,
            PurchaseOrderItemUpdate, PurchaseOrderQuery, PurchaseOrderSortField,
            PurchaseOrderStatus, PurchaseOrderUpdate, PurchasePaymentChannel,
            PurchasePaymentCreate, PurchasePaymentUpdate,
        },
    },
    storage::{PurchaseOrderRepository, RepoCtx},
};

// ── FK helpers ────────────────────────────────────────────────────────────────

async fn create_test_branch(ctx: &RepoCtx<DatabaseConnection>) -> i64 {
    let id = super::generate_test_id().await;
    let now = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.fZ")
        .to_string();
    ctx.db
        .execute_raw(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            "INSERT INTO branches (id, created_at, updated_at, is_deleted, is_main, name, code) \
             VALUES (?, ?, ?, 0, 1, 'Test Branch', 'TB')",
            [id.into(), now.clone().into(), now.into()],
        ))
        .await
        .expect("create_test_branch failed");
    id
}

async fn create_test_supplier(ctx: &RepoCtx<DatabaseConnection>) -> i64 {
    let id = super::generate_test_id().await;
    let now = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.fZ")
        .to_string();
    ctx.db
        .execute_raw(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            "INSERT INTO suppliers (id, created_at, updated_at, is_deleted, name) \
             VALUES (?, ?, ?, 0, 'Test Supplier')",
            [id.into(), now.clone().into(), now.into()],
        ))
        .await
        .expect("create_test_supplier failed");
    id
}

async fn create_test_product_variant(ctx: &RepoCtx<DatabaseConnection>) -> i64 {
    let product_id = super::generate_test_id().await;
    let variant_id = super::generate_test_id().await;
    let now = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.fZ")
        .to_string();
    ctx.db
        .execute_raw(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            "INSERT INTO products (id, created_at, updated_at, is_deleted, name, product_type) \
             VALUES (?, ?, ?, 0, 'Test Product', 'product')",
            [product_id.into(), now.clone().into(), now.clone().into()],
        ))
        .await
        .expect("create product failed");
    ctx.db
        .execute_raw(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            "INSERT INTO product_variants (id, created_at, updated_at, is_deleted, product_id) \
             VALUES (?, ?, ?, 0, ?)",
            [
                variant_id.into(),
                now.clone().into(),
                now.into(),
                product_id.into(),
            ],
        ))
        .await
        .expect("create variant failed");
    variant_id
}

fn default_query(filter: PurchaseOrderFilter) -> PurchaseOrderQuery {
    PurchaseOrderQuery {
        filter,
        sort_field: PurchaseOrderSortField::CreatedAt,
        sort_direction: SortDirection::Desc,
        cursor: None,
        limit: 20,
    }
}

fn one_item(variant_id: i64) -> PurchaseOrderItemCreate {
    PurchaseOrderItemCreate {
        product_variant_id: variant_id,
        product_name: "Test Product".to_string(),
        variant_name: None,
        barcode: None,
        quantity: 2,
        unit_cost: 10_000,
        discount_amount: 0,
    }
}

// ── Test suite ────────────────────────────────────────────────────────────────

pub async fn purchase_order_test_all<C, F, Fut>(repo: &C, ctx_factory: F)
where
    C: PurchaseOrderRepository,
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = RepoCtx<DatabaseConnection>>,
{
    purchase_order_test_create_and_get_by_id(&ctx_factory().await, repo).await;
    purchase_order_test_update(&ctx_factory().await, repo).await;
    purchase_order_test_add_payment_updates_paid_amount(&ctx_factory().await, repo).await;
    purchase_order_test_add_payment_full_status_paid(&ctx_factory().await, repo).await;
    purchase_order_test_add_payment_not_found(&ctx_factory().await, repo).await;
    purchase_order_test_soft_delete(&ctx_factory().await, repo).await;
    purchase_order_test_delete_not_found(&ctx_factory().await, repo).await;
    purchase_order_test_get_by_id_not_found(&ctx_factory().await, repo).await;
    purchase_order_test_update_not_found(&ctx_factory().await, repo).await;
    purchase_order_test_get_all_filter_by_status(&ctx_factory().await, repo).await;
    purchase_order_test_get_all_filter_by_supplier(&ctx_factory().await, repo).await;
    purchase_order_test_get_all_filter_by_number(&ctx_factory().await, repo).await;
    purchase_order_test_get_all_filter_by_reference_number(&ctx_factory().await, repo).await;
    purchase_order_test_cursor_pagination(&ctx_factory().await, repo).await;
    purchase_order_test_add_item(&ctx_factory().await, repo).await;
    purchase_order_test_update_item(&ctx_factory().await, repo).await;
    purchase_order_test_delete_item(&ctx_factory().await, repo).await;
    purchase_order_test_get_items(&ctx_factory().await, repo).await;
    purchase_order_test_get_payments(&ctx_factory().await, repo).await;
    purchase_order_test_update_payment(&ctx_factory().await, repo).await;
    purchase_order_test_delete_payment(&ctx_factory().await, repo).await;
}

// ── CRUD ──────────────────────────────────────────────────────────────────────

pub async fn purchase_order_test_create_and_get_by_id<C: PurchaseOrderRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let supplier_id = create_test_supplier(ctx).await;
    let variant_id = create_test_product_variant(ctx).await;

    let id = super::generate_test_id().await;
    let item_id = super::generate_test_id().await;

    let create = PurchaseOrderCreate {
        branch_id,
        supplier_id: Some(supplier_id),
        number: "PO-0001".to_string(),
        reference_number: Some("INV-SUPP-001".to_string()),
        order_date: None,
        expected_date: None,
        payment_due_date: None,
        discount_amount: 0,
        notes: Some("Test order".to_string()),
        metadata: None,
    };
    let create_item = PurchaseOrderItemCreate {
        product_variant_id: variant_id,
        product_name: "Test Product".to_string(),
        variant_name: None,
        barcode: Some("123456".to_string()),
        quantity: 10,
        unit_cost: 5_000,
        discount_amount: 0,
    };

    repo.create(ctx, id, &create, &[(item_id, create_item)])
        .await
        .expect("create failed");

    let fetched = repo
        .get_by_id(ctx, id)
        .await
        .expect("get_by_id failed")
        .expect("order not found");

    assert_eq!(fetched.id, id);
    assert_eq!(fetched.branch_id, branch_id);
    assert_eq!(fetched.supplier_id, Some(supplier_id));
    assert_eq!(fetched.number, "PO-0001");
    assert_eq!(fetched.reference_number, Some("INV-SUPP-001".to_string()));
    assert_eq!(fetched.status, PurchaseOrderStatus::Draft);
    assert_eq!(fetched.subtotal, 50_000);
    assert_eq!(fetched.total_amount, 50_000);
    assert_eq!(fetched.paid_amount, 0);
    assert_eq!(fetched.payment_status, PaymentStatus::Unpaid);
    assert!(!fetched.is_deleted);
    assert_eq!(fetched.items.len(), 1);
    assert_eq!(fetched.items[0].id, item_id);
    assert_eq!(fetched.items[0].quantity, 10);
    assert_eq!(fetched.items[0].total_cost, 50_000);
    assert!(fetched.payments.is_empty());
}

pub async fn purchase_order_test_update<C: PurchaseOrderRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let variant_id = create_test_product_variant(ctx).await;
    let id = super::generate_test_id().await;

    let item_id = super::generate_test_id().await;
    repo.create(
        ctx,
        id,
        &PurchaseOrderCreate {
            branch_id,
            supplier_id: None,
            number: "PO-UPD".to_string(),
            reference_number: None,
            order_date: None,
            expected_date: None,
            payment_due_date: None,
            discount_amount: 0,
            notes: None,
            metadata: None,
        },
        &[(item_id, one_item(variant_id))],
    )
    .await
    .expect("create failed");

    repo.update(
        ctx,
        id,
        &PurchaseOrderUpdate {
            status: Some(PurchaseOrderStatus::Ordered),
            notes: crate::domain::model::Update::Set("Sent to supplier".to_string()),
            order_date: crate::domain::model::Update::Set("2026-04-02T10:00:00.000Z".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("update failed");

    let updated = repo
        .get_by_id(ctx, id)
        .await
        .unwrap()
        .expect("not found after update");

    assert_eq!(updated.status, PurchaseOrderStatus::Ordered);
    assert_eq!(updated.notes, Some("Sent to supplier".to_string()));
    assert!(updated.order_date.is_some());
}

// ── Payment ───────────────────────────────────────────────────────────────────

pub async fn purchase_order_test_add_payment_updates_paid_amount<C: PurchaseOrderRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let variant_id = create_test_product_variant(ctx).await;
    let id = super::generate_test_id().await;
    let payment_id = super::generate_test_id().await;

    let item_id = super::generate_test_id().await;
    repo.create(
        ctx,
        id,
        &PurchaseOrderCreate {
            branch_id,
            supplier_id: None,
            number: "PO-PAY1".to_string(),
            reference_number: None,
            order_date: None,
            expected_date: None,
            payment_due_date: None,
            discount_amount: 0,
            notes: None,
            metadata: None,
        },
        &[(item_id, one_item(variant_id))], // total = 20_000
    )
    .await
    .expect("create failed");

    repo.add_payment(
        ctx,
        id,
        payment_id,
        &PurchasePaymentCreate {
            amount: 10_000,
            channel: PurchasePaymentChannel::Cash,
            paid_at: "2026-04-02T10:00:00.000Z".to_string(),
            reference: None,
            notes: None,
        },
    )
    .await
    .expect("add_payment failed");

    let order = repo.get_by_id(ctx, id).await.unwrap().unwrap();
    assert_eq!(order.paid_amount, 10_000);
    assert_eq!(order.payment_status, PaymentStatus::Partial);
    assert_eq!(order.payments.len(), 1);
    assert_eq!(order.payments[0].id, payment_id);
    assert_eq!(order.payments[0].amount, 10_000);
    assert_eq!(order.payments[0].channel, PurchasePaymentChannel::Cash);
}

pub async fn purchase_order_test_add_payment_full_status_paid<C: PurchaseOrderRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let variant_id = create_test_product_variant(ctx).await;
    let id = super::generate_test_id().await;
    let payment_id = super::generate_test_id().await;

    let item_id = super::generate_test_id().await;
    repo.create(
        ctx,
        id,
        &PurchaseOrderCreate {
            branch_id,
            supplier_id: None,
            number: "PO-PAY2".to_string(),
            reference_number: None,
            order_date: None,
            expected_date: None,
            payment_due_date: None,
            discount_amount: 0,
            notes: None,
            metadata: None,
        },
        &[(item_id, one_item(variant_id))], // total = 20_000
    )
    .await
    .expect("create failed");

    repo.add_payment(
        ctx,
        id,
        payment_id,
        &PurchasePaymentCreate {
            amount: 20_000,
            channel: PurchasePaymentChannel::BankTransfer,
            paid_at: "2026-04-02T11:00:00.000Z".to_string(),
            reference: Some("TRF-001".to_string()),
            notes: None,
        },
    )
    .await
    .expect("add_payment failed");

    let order = repo.get_by_id(ctx, id).await.unwrap().unwrap();
    assert_eq!(order.payment_status, PaymentStatus::Paid);
    assert_eq!(order.paid_amount, 20_000);
}

pub async fn purchase_order_test_add_payment_not_found<C: PurchaseOrderRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let result = repo
        .add_payment(
            ctx,
            999_999_999,
            super::generate_test_id().await,
            &PurchasePaymentCreate {
                amount: 1_000,
                channel: PurchasePaymentChannel::Cash,
                paid_at: "2026-04-02T10:00:00.000Z".to_string(),
                reference: None,
                notes: None,
            },
        )
        .await;
    assert!(
        matches!(result, Err(crate::domain::error::Error::NotFound(_))),
        "Expected NotFound when adding payment to non-existent order"
    );
}

// ── Delete ────────────────────────────────────────────────────────────────────

pub async fn purchase_order_test_soft_delete<C: PurchaseOrderRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let variant_id = create_test_product_variant(ctx).await;
    let id = super::generate_test_id().await;

    let item_id = super::generate_test_id().await;
    repo.create(
        ctx,
        id,
        &PurchaseOrderCreate {
            branch_id,
            supplier_id: None,
            number: "PO-DEL".to_string(),
            reference_number: None,
            order_date: None,
            expected_date: None,
            payment_due_date: None,
            discount_amount: 0,
            notes: None,
            metadata: None,
        },
        &[(item_id, one_item(variant_id))],
    )
    .await
    .expect("create failed");

    repo.delete(ctx, id).await.expect("delete failed");

    let result = repo.get_by_id(ctx, id).await.unwrap();
    assert!(result.is_none(), "Deleted order must not be returned");
}

pub async fn purchase_order_test_delete_not_found<C: PurchaseOrderRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let result = repo.delete(ctx, 999_999_999).await;
    assert!(
        matches!(result, Err(crate::domain::error::Error::NotFound(_))),
        "Expected NotFound for delete on non-existent order"
    );
}

pub async fn purchase_order_test_get_by_id_not_found<C: PurchaseOrderRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let result = repo.get_by_id(ctx, 999_999_999).await.unwrap();
    assert!(result.is_none());
}

pub async fn purchase_order_test_update_not_found<C: PurchaseOrderRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let result = repo
        .update(
            ctx,
            999_999_999,
            &PurchaseOrderUpdate {
                status: Some(PurchaseOrderStatus::Ordered),
                ..Default::default()
            },
        )
        .await;
    assert!(
        matches!(result, Err(crate::domain::error::Error::NotFound(_))),
        "Expected NotFound for update on non-existent order"
    );
}

// ── Filters ───────────────────────────────────────────────────────────────────

pub async fn purchase_order_test_get_all_filter_by_status<C: PurchaseOrderRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let variant_id = create_test_product_variant(ctx).await;
    let id_draft = super::generate_test_id().await;
    let id_ordered = super::generate_test_id().await;

    for (id, number) in [(id_draft, "PO-FS1"), (id_ordered, "PO-FS2")] {
        let item_id = super::generate_test_id().await;
        repo.create(
            ctx,
            id,
            &PurchaseOrderCreate {
                branch_id,
                supplier_id: None,
                number: number.to_string(),
                reference_number: None,
                order_date: None,
                expected_date: None,
                payment_due_date: None,
                discount_amount: 0,
                notes: None,
                metadata: None,
            },
            &[(item_id, one_item(variant_id))],
        )
        .await
        .expect("create failed");
    }

    repo.update(
        ctx,
        id_ordered,
        &PurchaseOrderUpdate {
            status: Some(PurchaseOrderStatus::Ordered),
            ..Default::default()
        },
    )
    .await
    .expect("update failed");

    let page = repo
        .get_all(
            ctx,
            &default_query(PurchaseOrderFilter {
                status: Some(PurchaseOrderStatus::Draft),
                ..Default::default()
            }),
        )
        .await
        .expect("get_all failed");

    assert!(
        page.items
            .iter()
            .all(|o| o.status == PurchaseOrderStatus::Draft)
    );
    assert!(page.items.iter().any(|o| o.id == id_draft));
    assert!(!page.items.iter().any(|o| o.id == id_ordered));
}

pub async fn purchase_order_test_get_all_filter_by_supplier<C: PurchaseOrderRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let supplier_a = create_test_supplier(ctx).await;
    let supplier_b = create_test_supplier(ctx).await;
    let variant_id = create_test_product_variant(ctx).await;
    let id_a = super::generate_test_id().await;
    let id_b = super::generate_test_id().await;

    for (id, supplier_id, number) in [
        (id_a, Some(supplier_a), "PO-SA"),
        (id_b, Some(supplier_b), "PO-SB"),
    ] {
        let item_id = super::generate_test_id().await;
        repo.create(
            ctx,
            id,
            &PurchaseOrderCreate {
                branch_id,
                supplier_id,
                number: number.to_string(),
                reference_number: None,
                order_date: None,
                expected_date: None,
                payment_due_date: None,
                discount_amount: 0,
                notes: None,
                metadata: None,
            },
            &[(item_id, one_item(variant_id))],
        )
        .await
        .expect("create failed");
    }

    let page = repo
        .get_all(
            ctx,
            &default_query(PurchaseOrderFilter {
                supplier_id: Some(supplier_a),
                ..Default::default()
            }),
        )
        .await
        .expect("get_all failed");

    assert!(page.items.iter().all(|o| o.supplier_id == Some(supplier_a)));
    assert!(page.items.iter().any(|o| o.id == id_a));
    assert!(!page.items.iter().any(|o| o.id == id_b));
}

pub async fn purchase_order_test_get_all_filter_by_number<C: PurchaseOrderRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let variant_id = create_test_product_variant(ctx).await;
    let id = super::generate_test_id().await;

    let item_id = super::generate_test_id().await;
    repo.create(
        ctx,
        id,
        &PurchaseOrderCreate {
            branch_id,
            supplier_id: None,
            number: "PO-UNIQUE-9999".to_string(),
            reference_number: None,
            order_date: None,
            expected_date: None,
            payment_due_date: None,
            discount_amount: 0,
            notes: None,
            metadata: None,
        },
        &[(item_id, one_item(variant_id))],
    )
    .await
    .expect("create failed");

    let page = repo
        .get_all(
            ctx,
            &default_query(PurchaseOrderFilter {
                number: Some("UNIQUE-9999".to_string()),
                ..Default::default()
            }),
        )
        .await
        .expect("get_all failed");

    assert!(page.items.iter().any(|o| o.id == id));
    assert!(page.items.iter().all(|o| o.number.contains("UNIQUE-9999")));
}

pub async fn purchase_order_test_get_all_filter_by_reference_number<C: PurchaseOrderRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let variant_id = create_test_product_variant(ctx).await;
    let id = super::generate_test_id().await;

    let item_id = super::generate_test_id().await;
    repo.create(
        ctx,
        id,
        &PurchaseOrderCreate {
            branch_id,
            supplier_id: None,
            number: "PO-REF-TEST".to_string(),
            reference_number: Some("SUPP-INV-XYZ".to_string()),
            order_date: None,
            expected_date: None,
            payment_due_date: None,
            discount_amount: 0,
            notes: None,
            metadata: None,
        },
        &[(item_id, one_item(variant_id))],
    )
    .await
    .expect("create failed");

    let page = repo
        .get_all(
            ctx,
            &default_query(PurchaseOrderFilter {
                reference_number: Some("INV-XYZ".to_string()),
                ..Default::default()
            }),
        )
        .await
        .expect("get_all failed");

    assert!(page.items.iter().any(|o| o.id == id));
}

// ── Pagination ────────────────────────────────────────────────────────────────

pub async fn purchase_order_test_cursor_pagination<C: PurchaseOrderRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let variant_id = create_test_product_variant(ctx).await;

    for i in 0..5u32 {
        let id = super::generate_test_id().await;
        let item_id = super::generate_test_id().await;
        repo.create(
            ctx,
            id,
            &PurchaseOrderCreate {
                branch_id,
                supplier_id: None,
                number: format!("PO-PAG-{i:04}"),
                reference_number: None,
                order_date: None,
                expected_date: None,
                payment_due_date: None,
                discount_amount: 0,
                notes: None,
                metadata: None,
            },
            &[(item_id, one_item(variant_id))],
        )
        .await
        .expect("create failed");
    }

    let page1 = repo
        .get_all(
            ctx,
            &PurchaseOrderQuery {
                filter: PurchaseOrderFilter::default(),
                sort_field: PurchaseOrderSortField::CreatedAt,
                sort_direction: SortDirection::Asc,
                cursor: None,
                limit: 3,
            },
        )
        .await
        .expect("page1 failed");

    assert_eq!(page1.items.len(), 3);
    assert!(page1.next_cursor.is_some());

    let page2 = repo
        .get_all(
            ctx,
            &PurchaseOrderQuery {
                filter: PurchaseOrderFilter::default(),
                sort_field: PurchaseOrderSortField::CreatedAt,
                sort_direction: SortDirection::Asc,
                cursor: page1.next_cursor,
                limit: 3,
            },
        )
        .await
        .expect("page2 failed");

    assert!(!page2.items.is_empty());

    // No overlap between pages
    let ids1: std::collections::HashSet<i64> = page1.items.iter().map(|o| o.id).collect();
    for o in &page2.items {
        assert!(!ids1.contains(&o.id), "Duplicate item across pages");
    }
}

// ── Item management ───────────────────────────────────────────────────────────

pub async fn purchase_order_test_add_item<C: PurchaseOrderRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let variant_id = create_test_product_variant(ctx).await;
    let order_id = super::generate_test_id().await;
    let item_id1 = super::generate_test_id().await;
    let item_id2 = super::generate_test_id().await;

    let create = PurchaseOrderCreate {
        branch_id,
        supplier_id: None,
        number: "PO-ADDITEM".to_string(),
        reference_number: None,
        order_date: None,
        expected_date: None,
        payment_due_date: None,
        discount_amount: 0,
        notes: None,
        metadata: None,
    };
    repo.create(
        ctx,
        order_id,
        &create,
        &[(
            item_id1,
            PurchaseOrderItemCreate {
                product_variant_id: variant_id,
                product_name: "Product A".to_string(),
                variant_name: None,
                barcode: None,
                quantity: 5,
                unit_cost: 1_000,
                discount_amount: 0,
            },
        )],
    )
    .await
    .expect("create failed");

    let order_before = repo.get_by_id(ctx, order_id).await.unwrap().unwrap();
    assert_eq!(order_before.subtotal, 5_000);
    assert_eq!(order_before.items.len(), 1);

    let new_item = PurchaseOrderItemCreate {
        product_variant_id: variant_id,
        product_name: "Product B".to_string(),
        variant_name: None,
        barcode: None,
        quantity: 3,
        unit_cost: 2_000,
        discount_amount: 0,
    };
    repo.add_item(ctx, order_id, item_id2, &new_item)
        .await
        .expect("add_item failed");

    let order_after = repo.get_by_id(ctx, order_id).await.unwrap().unwrap();
    assert_eq!(order_after.subtotal, 11_000); // 5_000 + 6_000
    assert_eq!(order_after.total_amount, 11_000);
    assert_eq!(order_after.items.len(), 2);
    assert!(order_after.items.iter().any(|i| i.id == item_id2));
}

pub async fn purchase_order_test_update_item<C: PurchaseOrderRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let variant_id = create_test_product_variant(ctx).await;
    let order_id = super::generate_test_id().await;
    let item_id = super::generate_test_id().await;

    repo.create(
        ctx,
        order_id,
        &PurchaseOrderCreate {
            branch_id,
            supplier_id: None,
            number: "PO-UPDITEM".to_string(),
            reference_number: None,
            order_date: None,
            expected_date: None,
            payment_due_date: None,
            discount_amount: 0,
            notes: None,
            metadata: None,
        },
        &[(
            item_id,
            PurchaseOrderItemCreate {
                product_variant_id: variant_id,
                product_name: "Old Name".to_string(),
                variant_name: None,
                barcode: None,
                quantity: 2,
                unit_cost: 5_000,
                discount_amount: 0,
            },
        )],
    )
    .await
    .expect("create failed");

    repo.update_item(
        ctx,
        item_id,
        &PurchaseOrderItemUpdate {
            quantity: Some(4),
            unit_cost: Some(3_000),
            product_name: Some("New Name".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("update_item failed");

    let order = repo.get_by_id(ctx, order_id).await.unwrap().unwrap();
    assert_eq!(order.items[0].quantity, 4);
    assert_eq!(order.items[0].unit_cost, 3_000);
    assert_eq!(order.items[0].total_cost, 12_000);
    assert_eq!(order.items[0].product_name, "New Name");
    assert_eq!(order.subtotal, 12_000);
    assert_eq!(order.total_amount, 12_000);
}

pub async fn purchase_order_test_delete_item<C: PurchaseOrderRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let variant_id = create_test_product_variant(ctx).await;
    let order_id = super::generate_test_id().await;
    let item_id1 = super::generate_test_id().await;
    let item_id2 = super::generate_test_id().await;

    repo.create(
        ctx,
        order_id,
        &PurchaseOrderCreate {
            branch_id,
            supplier_id: None,
            number: "PO-DELITEM".to_string(),
            reference_number: None,
            order_date: None,
            expected_date: None,
            payment_due_date: None,
            discount_amount: 0,
            notes: None,
            metadata: None,
        },
        &[
            (
                item_id1,
                PurchaseOrderItemCreate {
                    product_variant_id: variant_id,
                    product_name: "Item 1".to_string(),
                    variant_name: None,
                    barcode: None,
                    quantity: 2,
                    unit_cost: 5_000,
                    discount_amount: 0,
                },
            ),
            (
                item_id2,
                PurchaseOrderItemCreate {
                    product_variant_id: variant_id,
                    product_name: "Item 2".to_string(),
                    variant_name: None,
                    barcode: None,
                    quantity: 1,
                    unit_cost: 3_000,
                    discount_amount: 0,
                },
            ),
        ],
    )
    .await
    .expect("create failed");

    let order_before = repo.get_by_id(ctx, order_id).await.unwrap().unwrap();
    assert_eq!(order_before.subtotal, 13_000);
    assert_eq!(order_before.items.len(), 2);

    repo.delete_item(ctx, item_id1)
        .await
        .expect("delete_item failed");

    let order_after = repo.get_by_id(ctx, order_id).await.unwrap().unwrap();
    assert_eq!(order_after.subtotal, 3_000);
    assert_eq!(order_after.total_amount, 3_000);
    assert_eq!(order_after.items.len(), 1);
    assert!(!order_after.items.iter().any(|i| i.id == item_id1));

    // delete_item not found
    let result = repo.delete_item(ctx, 999_999_999).await;
    assert!(
        matches!(result, Err(crate::domain::error::Error::NotFound(_))),
        "Expected NotFound for delete_item on non-existent item"
    );
}

pub async fn purchase_order_test_get_items<C: PurchaseOrderRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let variant_id = create_test_product_variant(ctx).await;
    let order_id = super::generate_test_id().await;
    let item_id = super::generate_test_id().await;

    repo.create(
        ctx,
        order_id,
        &PurchaseOrderCreate {
            branch_id,
            supplier_id: None,
            number: "PO-GETITEMS".to_string(),
            reference_number: None,
            order_date: None,
            expected_date: None,
            payment_due_date: None,
            discount_amount: 0,
            notes: None,
            metadata: None,
        },
        &[(item_id, one_item(variant_id))],
    )
    .await
    .expect("create failed");

    let items = repo
        .get_items(ctx, order_id)
        .await
        .expect("get_items failed");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, item_id);
    assert_eq!(items[0].purchase_order_id, order_id);
}

// ── Payment management ────────────────────────────────────────────────────────

pub async fn purchase_order_test_get_payments<C: PurchaseOrderRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let variant_id = create_test_product_variant(ctx).await;
    let order_id = super::generate_test_id().await;
    let item_id = super::generate_test_id().await;
    let payment_id = super::generate_test_id().await;

    repo.create(
        ctx,
        order_id,
        &PurchaseOrderCreate {
            branch_id,
            supplier_id: None,
            number: "PO-GETPAY".to_string(),
            reference_number: None,
            order_date: None,
            expected_date: None,
            payment_due_date: None,
            discount_amount: 0,
            notes: None,
            metadata: None,
        },
        &[(item_id, one_item(variant_id))],
    )
    .await
    .expect("create failed");

    repo.add_payment(
        ctx,
        order_id,
        payment_id,
        &PurchasePaymentCreate {
            amount: 5_000,
            channel: PurchasePaymentChannel::Cash,
            paid_at: "2026-04-02T10:00:00.000Z".to_string(),
            reference: None,
            notes: None,
        },
    )
    .await
    .expect("add_payment failed");

    let payments = repo
        .get_payments(ctx, order_id)
        .await
        .expect("get_payments failed");
    assert_eq!(payments.len(), 1);
    assert_eq!(payments[0].id, payment_id);
    assert_eq!(payments[0].amount, 5_000);
    assert_eq!(payments[0].purchase_order_id, order_id);
}

pub async fn purchase_order_test_update_payment<C: PurchaseOrderRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let variant_id = create_test_product_variant(ctx).await;
    let order_id = super::generate_test_id().await;
    let item_id = super::generate_test_id().await;
    let payment_id = super::generate_test_id().await;

    repo.create(
        ctx,
        order_id,
        &PurchaseOrderCreate {
            branch_id,
            supplier_id: None,
            number: "PO-UPDPAY".to_string(),
            reference_number: None,
            order_date: None,
            expected_date: None,
            payment_due_date: None,
            discount_amount: 0,
            notes: None,
            metadata: None,
        },
        &[(item_id, one_item(variant_id))], // subtotal = 20_000
    )
    .await
    .expect("create failed");

    repo.add_payment(
        ctx,
        order_id,
        payment_id,
        &PurchasePaymentCreate {
            amount: 5_000,
            channel: PurchasePaymentChannel::Cash,
            paid_at: "2026-04-02T10:00:00.000Z".to_string(),
            reference: None,
            notes: Some("initial".to_string()),
        },
    )
    .await
    .expect("add_payment failed");

    // Update payment: change amount and channel
    repo.update_payment(
        ctx,
        payment_id,
        &PurchasePaymentUpdate {
            amount: Some(10_000),
            channel: Some(PurchasePaymentChannel::BankTransfer),
            notes: crate::domain::model::Update::Set("updated".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("update_payment failed");

    let payments = repo.get_payments(ctx, order_id).await.unwrap();
    assert_eq!(payments[0].amount, 10_000);
    assert_eq!(payments[0].channel, PurchasePaymentChannel::BankTransfer);
    assert_eq!(payments[0].notes, Some("updated".to_string()));

    let order = repo.get_by_id(ctx, order_id).await.unwrap().unwrap();
    assert_eq!(order.paid_amount, 10_000);
    assert_eq!(order.payment_status, PaymentStatus::Partial);

    // update_payment not found
    let result = repo
        .update_payment(ctx, 999_999_999, &PurchasePaymentUpdate::default())
        .await;
    assert!(
        matches!(result, Err(crate::domain::error::Error::NotFound(_))),
        "Expected NotFound for update_payment on non-existent payment"
    );
}

pub async fn purchase_order_test_delete_payment<C: PurchaseOrderRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &C,
) {
    let branch_id = create_test_branch(ctx).await;
    let variant_id = create_test_product_variant(ctx).await;
    let order_id = super::generate_test_id().await;
    let item_id = super::generate_test_id().await;
    let payment_id = super::generate_test_id().await;

    repo.create(
        ctx,
        order_id,
        &PurchaseOrderCreate {
            branch_id,
            supplier_id: None,
            number: "PO-DELPAY".to_string(),
            reference_number: None,
            order_date: None,
            expected_date: None,
            payment_due_date: None,
            discount_amount: 0,
            notes: None,
            metadata: None,
        },
        &[(item_id, one_item(variant_id))], // total = 20_000
    )
    .await
    .expect("create failed");

    repo.add_payment(
        ctx,
        order_id,
        payment_id,
        &PurchasePaymentCreate {
            amount: 20_000,
            channel: PurchasePaymentChannel::Cash,
            paid_at: "2026-04-02T10:00:00.000Z".to_string(),
            reference: None,
            notes: None,
        },
    )
    .await
    .expect("add_payment failed");

    let order_before = repo.get_by_id(ctx, order_id).await.unwrap().unwrap();
    assert_eq!(order_before.payment_status, PaymentStatus::Paid);
    assert_eq!(order_before.paid_amount, 20_000);

    repo.delete_payment(ctx, payment_id)
        .await
        .expect("delete_payment failed");

    let order_after = repo.get_by_id(ctx, order_id).await.unwrap().unwrap();
    assert_eq!(order_after.paid_amount, 0);
    assert_eq!(order_after.payment_status, PaymentStatus::Unpaid);
    let payments = repo.get_payments(ctx, order_id).await.unwrap();
    assert!(payments.is_empty());

    // delete_payment not found
    let result = repo.delete_payment(ctx, 999_999_999).await;
    assert!(
        matches!(result, Err(crate::domain::error::Error::NotFound(_))),
        "Expected NotFound for delete_payment on non-existent payment"
    );
}
