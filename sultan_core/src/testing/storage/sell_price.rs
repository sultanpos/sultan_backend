use crate::{
    domain::{
        Context, Error,
        model::{
            Update,
            sell_price::{SellPriceCreate, SellPriceUpdate},
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
}
