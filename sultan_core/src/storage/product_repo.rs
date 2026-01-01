use async_trait::async_trait;

use crate::domain::{
    Context, DomainResult,
    model::{
        category::Category,
        product::{
            Product, ProductCreate, ProductUpdate, ProductVariant, ProductVariantCreate,
            ProductVariantUpdate,
        },
    },
};

#[async_trait]
pub trait ProductRepository<Tx>: Send + Sync {
    async fn create_product(
        &self,
        ctx: &Context,
        id: i64,
        product: &ProductCreate,
        tx: &mut Tx,
    ) -> DomainResult<()>;
    async fn update_product(
        &self,
        ctx: &Context,
        id: i64,
        product: &ProductUpdate,
        tx: &mut Tx,
    ) -> DomainResult<()>;
    async fn delete_product(&self, ctx: &Context, id: i64, tx: &mut Tx) -> DomainResult<()>;
    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<Product>>;

    async fn create_variant(
        &self,
        ctx: &Context,
        id: i64,
        variant: &ProductVariantCreate,
        tx: &mut Tx,
    ) -> DomainResult<()>;
    async fn update_variant(
        &self,
        ctx: &Context,
        id: i64,
        variant: &ProductVariantUpdate,
    ) -> DomainResult<()>;
    async fn delete_variant(&self, ctx: &Context, id: i64, tx: &mut Tx) -> DomainResult<()>;
    async fn delete_variants_by_product_id(
        &self,
        ctx: &Context,
        product_id: i64,
        tx: &mut Tx,
    ) -> DomainResult<()>;
    async fn get_variant_by_barcode(
        &self,
        ctx: &Context,
        barcode: &str,
    ) -> DomainResult<Option<ProductVariant>>;
    async fn get_variant_by_id(
        &self,
        ctx: &Context,
        id: i64,
    ) -> DomainResult<Option<ProductVariant>>;
    async fn get_variant_by_product_id(
        &self,
        ctx: &Context,
        product_id: i64,
    ) -> DomainResult<Vec<ProductVariant>>;

    async fn get_product_category(&self, ctx: &Context, product_id: i64) -> DomainResult<Vec<i64>>;
}
