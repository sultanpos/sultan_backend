use crate::{
    domain::{
        Context, Error,
        model::{
            Update,
            sell_price::{
                SellDiscountCreate, SellDiscountUpdate, SellPriceCreate, SellPriceUpdate,
            },
        },
    },
    storage::{
        sell_price_repo::SellPriceRepository,
        sqlite::{SqliteProductRepository, SqliteSellPriceRepository},
        transaction::TransactionManager,
    },
};
use serde_json::json;

pub async fn create_sqlite_branch_repo()
-> (Context, SqliteProductRepository, SqliteSellPriceRepository) {
    let pool = super::init_sqlite_pool().await;
    (
        Context::new(),
        SqliteProductRepository::new(pool.clone()),
        SqliteSellPriceRepository::new(pool.clone()),
    )
}

pub struct SellPriceTestData<'a, T: TransactionManager + 'a> {
    pub ctx: Context,
    pub product_id: i64,
    pub unit_id: i64,
    pub variant_id: Vec<i64>,
    pub tx_manager: Box<T>,
    pub sell_price_repo: Box<dyn SellPriceRepository<T::Transaction<'a>>>,
}

pub async fn sell_price_test_repo_integration<'a, T: TransactionManager + 'a>(
    test_data: &SellPriceTestData<'a, T>,
) {
    let id = super::generate_test_id().await;
    let sell_price = SellPriceCreate {
        branch_id: None,
        product_variant_id: test_data.variant_id[0],
        price: 1000,
        quantity: 10,
        uom_id: test_data.unit_id,
        metadata: Some(json!({"key": "value"})),
    };

    test_data
        .sell_price_repo
        .create(&test_data.ctx, id, &sell_price)
        .await
        .expect("failed to insert sell price");

    let fetched_price = test_data
        .sell_price_repo
        .get_by_id(&test_data.ctx, id)
        .await
        .expect("failed to get sell price")
        .expect("sell price not found");
    assert_eq!(fetched_price.id, id);
    assert_eq!(fetched_price.price, 1000);
    assert_eq!(fetched_price.quantity, 10);
    assert_eq!(fetched_price.product_variant_id, test_data.variant_id[0]);
    assert_eq!(fetched_price.uom_id, test_data.unit_id);
    assert_eq!(fetched_price.metadata, Some(json!({"key": "value"})));

    let update_result = test_data
        .sell_price_repo
        .update(
            &test_data.ctx,
            1,
            &SellPriceUpdate {
                price: Some(1200),
                quantity: None,
                uom_id: None,
                metadata: Update::Unchanged,
            },
        )
        .await;
    assert!(matches!(update_result, Err(Error::NotFound(_))));

    test_data
        .sell_price_repo
        .update(
            &test_data.ctx,
            id,
            &SellPriceUpdate {
                price: Some(1200),
                quantity: None,
                uom_id: None,
                metadata: Update::Unchanged,
            },
        )
        .await
        .expect("unable to update");
}

pub async fn sell_price_test_delete<'a, T: TransactionManager + 'a>(
    test_data: &SellPriceTestData<'a, T>,
) {
    let id = super::generate_test_id().await;
    let sell_price = SellPriceCreate {
        branch_id: None,
        product_variant_id: test_data.variant_id[0],
        price: 1500,
        quantity: 5,
        uom_id: test_data.unit_id,
        metadata: Some(json!({"test": "delete"})),
    };

    test_data
        .sell_price_repo
        .create(&test_data.ctx, id, &sell_price)
        .await
        .expect("Failed to create sell price");

    test_data
        .sell_price_repo
        .delete(&test_data.ctx, id)
        .await
        .expect("Failed to delete sell price");

    let result = test_data
        .sell_price_repo
        .get_by_id(&test_data.ctx, id)
        .await
        .expect("Failed to get deleted sell price");

    assert!(result.is_none());
}

pub async fn sell_price_test_delete_non_existent<'a, T: TransactionManager + 'a>(
    test_data: &SellPriceTestData<'a, T>,
) {
    let result = test_data
        .sell_price_repo
        .delete(&test_data.ctx, 999999)
        .await;
    assert!(matches!(result, Err(Error::NotFound(_))));
}

pub async fn sell_price_test_get_by_id_not_found<'a, T: TransactionManager + 'a>(
    test_data: &SellPriceTestData<'a, T>,
) {
    let result = test_data
        .sell_price_repo
        .get_by_id(&test_data.ctx, 999999)
        .await
        .expect("Failed to execute get_by_id");

    assert!(result.is_none());
}

pub async fn sell_price_test_get_all_by_variant_empty<'a, T: TransactionManager + 'a>(
    test_data: &SellPriceTestData<'a, T>,
) {
    let prices = test_data
        .sell_price_repo
        .get_all_by_product_variant_id(&test_data.ctx, test_data.variant_id[0])
        .await
        .expect("Failed to get prices");

    assert!(prices.is_empty());
}

pub async fn sell_price_test_get_all_by_variant_multiple<'a, T: TransactionManager + 'a>(
    test_data: &SellPriceTestData<'a, T>,
) {
    let variant_id = test_data.variant_id[0];

    // Create multiple prices for the same variant with different branch_ids to avoid unique constraint
    for i in 1..=3 {
        let id = super::generate_test_id().await;
        let branch_id = super::generate_test_id().await; // Use unique branch_id for each
        let price = SellPriceCreate {
            branch_id: Some(branch_id),
            product_variant_id: variant_id,
            price: 1000 + (i * 100),
            quantity: 10 + i,
            uom_id: test_data.unit_id,
            metadata: Some(json!({"index": i})),
        };

        test_data
            .sell_price_repo
            .create(&test_data.ctx, id, &price)
            .await
            .expect("Failed to create sell price");
    }

    let prices = test_data
        .sell_price_repo
        .get_all_by_product_variant_id(&test_data.ctx, variant_id)
        .await
        .expect("Failed to get prices");

    assert_eq!(prices.len(), 3);
}

pub async fn sell_price_test_update_price_only<'a, T: TransactionManager + 'a>(
    test_data: &SellPriceTestData<'a, T>,
) {
    let id = super::generate_test_id().await;
    let sell_price = SellPriceCreate {
        branch_id: None,
        product_variant_id: test_data.variant_id[0],
        price: 1000,
        quantity: 10,
        uom_id: test_data.unit_id,
        metadata: Some(json!({"key": "original"})),
    };

    test_data
        .sell_price_repo
        .create(&test_data.ctx, id, &sell_price)
        .await
        .expect("Failed to create sell price");

    test_data
        .sell_price_repo
        .update(
            &test_data.ctx,
            id,
            &SellPriceUpdate {
                price: Some(1500),
                quantity: None,
                uom_id: None,
                metadata: Update::Unchanged,
            },
        )
        .await
        .expect("Failed to update price");

    let updated = test_data
        .sell_price_repo
        .get_by_id(&test_data.ctx, id)
        .await
        .expect("Failed to get updated price")
        .expect("Price not found");

    assert_eq!(updated.price, 1500);
    assert_eq!(updated.quantity, 10);
    assert_eq!(updated.metadata, Some(json!({"key": "original"})));
}

pub async fn sell_price_test_update_quantity_only<'a, T: TransactionManager + 'a>(
    test_data: &SellPriceTestData<'a, T>,
) {
    let id = super::generate_test_id().await;
    let sell_price = SellPriceCreate {
        branch_id: None,
        product_variant_id: test_data.variant_id[0],
        price: 2000,
        quantity: 5,
        uom_id: test_data.unit_id,
        metadata: None,
    };

    test_data
        .sell_price_repo
        .create(&test_data.ctx, id, &sell_price)
        .await
        .expect("Failed to create sell price");

    test_data
        .sell_price_repo
        .update(
            &test_data.ctx,
            id,
            &SellPriceUpdate {
                price: None,
                quantity: Some(15),
                uom_id: None,
                metadata: Update::Unchanged,
            },
        )
        .await
        .expect("Failed to update quantity");

    let updated = test_data
        .sell_price_repo
        .get_by_id(&test_data.ctx, id)
        .await
        .expect("Failed to get updated price")
        .expect("Price not found");

    assert_eq!(updated.quantity, 15);
    assert_eq!(updated.price, 2000);
}

pub async fn sell_price_test_update_metadata<'a, T: TransactionManager + 'a>(
    test_data: &SellPriceTestData<'a, T>,
) {
    let id = super::generate_test_id().await;
    let sell_price = SellPriceCreate {
        branch_id: None,
        product_variant_id: test_data.variant_id[0],
        price: 3000,
        quantity: 8,
        uom_id: test_data.unit_id,
        metadata: Some(json!({"old": "data"})),
    };

    test_data
        .sell_price_repo
        .create(&test_data.ctx, id, &sell_price)
        .await
        .expect("Failed to create sell price");

    test_data
        .sell_price_repo
        .update(
            &test_data.ctx,
            id,
            &SellPriceUpdate {
                price: None,
                quantity: None,
                uom_id: None,
                metadata: Update::Set(json!({"new": "value"})),
            },
        )
        .await
        .expect("Failed to update metadata");

    let updated = test_data
        .sell_price_repo
        .get_by_id(&test_data.ctx, id)
        .await
        .expect("Failed to get updated price")
        .expect("Price not found");

    assert_eq!(updated.metadata, Some(json!({"new": "value"})));
}

pub async fn sell_price_test_different_variants<'a, T: TransactionManager + 'a>(
    test_data: &SellPriceTestData<'a, T>,
) {
    let id1 = super::generate_test_id().await;
    let id2 = super::generate_test_id().await;

    let price1 = SellPriceCreate {
        branch_id: None,
        product_variant_id: test_data.variant_id[0],
        price: 1000,
        quantity: 10,
        uom_id: test_data.unit_id,
        metadata: None,
    };

    let price2 = SellPriceCreate {
        branch_id: None,
        product_variant_id: test_data.variant_id[1],
        price: 2000,
        quantity: 20,
        uom_id: test_data.unit_id,
        metadata: None,
    };

    test_data
        .sell_price_repo
        .create(&test_data.ctx, id1, &price1)
        .await
        .expect("Failed to create price 1");

    test_data
        .sell_price_repo
        .create(&test_data.ctx, id2, &price2)
        .await
        .expect("Failed to create price 2");

    let prices1 = test_data
        .sell_price_repo
        .get_all_by_product_variant_id(&test_data.ctx, test_data.variant_id[0])
        .await
        .expect("Failed to get prices for variant 1");

    let prices2 = test_data
        .sell_price_repo
        .get_all_by_product_variant_id(&test_data.ctx, test_data.variant_id[1])
        .await
        .expect("Failed to get prices for variant 2");

    assert_eq!(prices1.len(), 1);
    assert_eq!(prices1[0].price, 1000);
    assert_eq!(prices2.len(), 1);
    assert_eq!(prices2[0].price, 2000);
}

// =============================================================================
// Discount Tests
// =============================================================================

pub async fn sell_price_test_create_discount<'a, T: TransactionManager + 'a>(
    test_data: &SellPriceTestData<'a, T>,
) {
    let price_id = super::generate_test_id().await;
    let sell_price = SellPriceCreate {
        branch_id: None,
        product_variant_id: test_data.variant_id[0],
        price: 10000,
        quantity: 10,
        uom_id: test_data.unit_id,
        metadata: None,
    };

    test_data
        .sell_price_repo
        .create(&test_data.ctx, price_id, &sell_price)
        .await
        .expect("Failed to create sell price");

    let discount_id = super::generate_test_id().await;
    let discount = SellDiscountCreate {
        price_id: price_id,
        quantity: 5,
        discount_formula: "price * 0.9".to_string(),
        customer_level: None,
        metadata: Some(json!({"type": "bulk"})),
    };

    test_data
        .sell_price_repo
        .create_discount(&test_data.ctx, discount_id, &discount)
        .await
        .expect("Failed to create discount");

    let fetched = test_data
        .sell_price_repo
        .get_discount_by_id(&test_data.ctx, discount_id)
        .await
        .expect("Failed to get discount")
        .expect("Discount not found");

    assert_eq!(fetched.price_id, price_id);
    assert_eq!(fetched.quantity, 5);
    assert_eq!(fetched.discount_formula, Some("price * 0.9".to_string()));
}

pub async fn sell_price_test_update_discount<'a, T: TransactionManager + 'a>(
    test_data: &SellPriceTestData<'a, T>,
) {
    let price_id = super::generate_test_id().await;
    let sell_price = SellPriceCreate {
        branch_id: None,
        product_variant_id: test_data.variant_id[0],
        price: 5000,
        quantity: 10,
        uom_id: test_data.unit_id,
        metadata: None,
    };

    test_data
        .sell_price_repo
        .create(&test_data.ctx, price_id, &sell_price)
        .await
        .expect("Failed to create sell price");

    let discount_id = super::generate_test_id().await;
    let discount = SellDiscountCreate {
        price_id: price_id,
        quantity: 10,
        discount_formula: "price * 0.95".to_string(),
        customer_level: None,
        metadata: None,
    };

    test_data
        .sell_price_repo
        .create_discount(&test_data.ctx, discount_id, &discount)
        .await
        .expect("Failed to create discount");

    let update = SellDiscountUpdate {
        quantity: Some(20),
        discount_formula: Some("price * 0.85".to_string()),
        customer_level: Update::Unchanged,
        metadata: Update::Set(json!({"updated": true})),
    };

    test_data
        .sell_price_repo
        .update_discount(&test_data.ctx, discount_id, &update)
        .await
        .expect("Failed to update discount");

    let updated = test_data
        .sell_price_repo
        .get_discount_by_id(&test_data.ctx, discount_id)
        .await
        .expect("Failed to get updated discount")
        .expect("Discount not found");

    assert_eq!(updated.quantity, 20);
    assert_eq!(updated.discount_formula, Some("price * 0.85".to_string()));
    assert_eq!(updated.metadata, Some(json!({"updated": true})));
}

pub async fn sell_price_test_delete_discount<'a, T: TransactionManager + 'a>(
    test_data: &SellPriceTestData<'a, T>,
) {
    let price_id = super::generate_test_id().await;
    let sell_price = SellPriceCreate {
        branch_id: None,
        product_variant_id: test_data.variant_id[0],
        price: 8000,
        quantity: 10,
        uom_id: test_data.unit_id,
        metadata: None,
    };

    test_data
        .sell_price_repo
        .create(&test_data.ctx, price_id, &sell_price)
        .await
        .expect("Failed to create sell price");

    let discount_id = super::generate_test_id().await;
    let discount = SellDiscountCreate {
        price_id: price_id,
        quantity: 15,
        discount_formula: "price * 0.8".to_string(),
        customer_level: None,
        metadata: None,
    };

    test_data
        .sell_price_repo
        .create_discount(&test_data.ctx, discount_id, &discount)
        .await
        .expect("Failed to create discount");

    test_data
        .sell_price_repo
        .delete_discount(&test_data.ctx, discount_id)
        .await
        .expect("Failed to delete discount");

    let result = test_data
        .sell_price_repo
        .get_discount_by_id(&test_data.ctx, discount_id)
        .await
        .expect("Failed to get deleted discount");

    assert!(result.is_none());
}

pub async fn sell_price_test_get_all_discounts<'a, T: TransactionManager + 'a>(
    test_data: &SellPriceTestData<'a, T>,
) {
    let price_id = super::generate_test_id().await;
    let sell_price = SellPriceCreate {
        branch_id: None,
        product_variant_id: test_data.variant_id[0],
        price: 12000,
        quantity: 10,
        uom_id: test_data.unit_id,
        metadata: None,
    };

    test_data
        .sell_price_repo
        .create(&test_data.ctx, price_id, &sell_price)
        .await
        .expect("Failed to create sell price");

    for i in 0..3 {
        let discount_id = super::generate_test_id().await;
        let discount = SellDiscountCreate {
            price_id: price_id,
            quantity: 10 + (i * 10),
            discount_formula: format!("price * {}", 0.95 - (i as f64 * 0.05)),
            customer_level: None,
            metadata: Some(json!({"tier": i})),
        };

        test_data
            .sell_price_repo
            .create_discount(&test_data.ctx, discount_id, &discount)
            .await
            .expect("Failed to create discount");
    }

    let discounts = test_data
        .sell_price_repo
        .get_all_discount_by_price_id(&test_data.ctx, price_id)
        .await
        .expect("Failed to get discounts");

    assert_eq!(discounts.len(), 3);
}

pub async fn sell_price_test_get_discount_not_found<'a, T: TransactionManager + 'a>(
    test_data: &SellPriceTestData<'a, T>,
) {
    let result = test_data
        .sell_price_repo
        .get_discount_by_id(&test_data.ctx, 999999)
        .await
        .expect("Failed to execute get_discount_by_id");

    assert!(result.is_none());
}
