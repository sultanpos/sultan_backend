use async_trait::async_trait;

use crate::domain::{
    Context, DomainResult,
    model::sell_price::{
        SellDiscount, SellDiscountCreate, SellDiscountUpdate, SellPrice, SellPriceCreate,
        SellPriceUpdate,
    },
};

#[async_trait]
pub trait SellPriceRepository<Tx>: Send + Sync {
    async fn create(&self, ctx: &Context, id: i64, price: &SellPriceCreate) -> DomainResult<()>;
    async fn create_tx(
        &self,
        ctx: &Context,
        id: i64,
        price: &SellPriceCreate,
        tx: &mut Tx,
    ) -> DomainResult<()>;
    async fn update(&self, ctx: &Context, id: i64, price: &SellPriceUpdate) -> DomainResult<()>;
    async fn update_tx(
        &self,
        ctx: &Context,
        id: i64,
        price: &SellPriceUpdate,
        tx: &mut Tx,
    ) -> DomainResult<()>;
    async fn delete(&self, ctx: &Context, id: i64) -> DomainResult<()>;
    async fn delete_tx(&self, ctx: &Context, id: i64, tx: &mut Tx) -> DomainResult<()>;
    async fn get_all_by_product_variant_id(
        &self,
        ctx: &Context,
        id: i64,
    ) -> DomainResult<Vec<SellPrice>>;
    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<SellPrice>>;

    async fn create_discount(
        &self,
        ctx: &Context,
        id: i64,
        price: &SellDiscountCreate,
    ) -> DomainResult<()>;
    async fn create_discount_tx(
        &self,
        ctx: &Context,
        id: i64,
        price: &SellDiscountCreate,
        tx: &mut Tx,
    ) -> DomainResult<()>;
    async fn update_discount(
        &self,
        ctx: &Context,
        id: i64,
        price: &SellDiscountUpdate,
    ) -> DomainResult<()>;
    async fn update_discount_tx(
        &self,
        ctx: &Context,
        id: i64,
        price: &SellDiscountUpdate,
        tx: &mut Tx,
    ) -> DomainResult<()>;
    async fn delete_discount(&self, ctx: &Context, id: i64) -> DomainResult<()>;
    async fn delete_discount_by_sell_price_id_tx(
        &self,
        ctx: &Context,
        id: i64,
        tx: &mut Tx,
    ) -> DomainResult<()>;
    async fn get_all_discount_by_price_id(
        &self,
        ctx: &Context,
        id: i64,
    ) -> DomainResult<Vec<SellDiscount>>;
    async fn get_discount_by_id(
        &self,
        ctx: &Context,
        id: i64,
    ) -> DomainResult<Option<SellDiscount>>;
}
