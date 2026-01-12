use async_trait::async_trait;

use crate::domain::{
    Context, DomainResult,
    model::{
        sell_price::{
            SellDiscount, SellDiscountCreate, SellDiscountUpdate, SellPrice, SellPriceCreate,
            SellPriceUpdate,
        },
        stock::{Stock, StockCreate, StockUpdate},
    },
};

#[async_trait]
pub trait StockRepository<Tx>: Send + Sync {
    async fn create_tx(
        &self,
        ctx: &Context,
        id: i64,
        price: &StockCreate,
        tx: &mut Tx,
    ) -> DomainResult<()>;
    async fn update_tx(
        &self,
        ctx: &Context,
        id: i64,
        price: &StockUpdate,
        tx: &mut Tx,
    ) -> DomainResult<()>;
    async fn delete_tx(&self, ctx: &Context, id: i64, tx: &mut Tx) -> DomainResult<()>;
    async fn get_all_by_product_variant_id(
        &self,
        ctx: &Context,
        id: i64,
    ) -> DomainResult<Vec<Stock>>;
    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<SellPrice>>;
}
