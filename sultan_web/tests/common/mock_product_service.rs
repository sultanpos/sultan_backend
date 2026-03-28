use async_trait::async_trait;
use sultan_core::application::ProductServiceTrait;
use sultan_core::domain::{
    DomainResult, Error,
    context::Context,
    model::product::{
        CursorPage, Product, ProductFullCreate, ProductQuery, ProductUpdate, ProductVariant,
        ProductVariantCreate, ProductVariantUpdate,
    },
};

pub struct MockProductService {
    pub should_succeed: bool,
    pub id: i64,
}

impl MockProductService {
    pub fn new_success() -> Self {
        Self {
            should_succeed: true,
            id: 1,
        }
    }

    #[allow(dead_code)]
    pub fn new_failure() -> Self {
        Self {
            should_succeed: false,
            id: 1,
        }
    }

    fn create_mock_product(&self) -> Product {
        Product {
            id: self.id,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
            is_deleted: false,
            name: "Test Product".to_string(),
            description: Some("Test Description".to_string()),
            product_type: "goods".to_string(),
            main_image: None,
            sellable: true,
            buyable: true,
            editable_price: false,
            metadata: None,
            variant_count: 0,
            categories: vec![],
            variants: vec![],
        }
    }

    fn create_mock_variant(&self) -> ProductVariant {
        ProductVariant {
            id: self.id,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
            is_deleted: false,
            barcode: Some("1234567890".to_string()),
            name: Some("Test Variant".to_string()),
            metadata: None,
            sell_prices: vec![],
        }
    }
}

#[async_trait]
impl ProductServiceTrait for MockProductService {
    async fn create_product(
        &self,
        _ctx: &Context,
        _product: &ProductFullCreate,
    ) -> DomainResult<i64> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to create product".to_string()));
        }
        Ok(self.id)
    }

    async fn update_product(
        &self,
        _ctx: &Context,
        id: i64,
        _product: &ProductUpdate,
    ) -> DomainResult<()> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to update product".to_string()));
        }
        if id != self.id {
            return Err(Error::NotFound(format!("Product with id {} not found", id)));
        }
        Ok(())
    }

    async fn delete_product(&self, _ctx: &Context, id: i64) -> DomainResult<()> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to delete product".to_string()));
        }
        if id != self.id {
            return Err(Error::NotFound(format!("Product with id {} not found", id)));
        }
        Ok(())
    }

    async fn get_by_id(&self, _ctx: &Context, id: i64) -> DomainResult<Option<Product>> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to get product".to_string()));
        }
        if id == self.id {
            Ok(Some(self.create_mock_product()))
        } else {
            Ok(None)
        }
    }

    async fn get_all_products(
        &self,
        _ctx: &Context,
        _query: &ProductQuery,
    ) -> DomainResult<CursorPage<Product>> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to get products".to_string()));
        }
        Ok(CursorPage {
            items: vec![self.create_mock_product()],
            next_cursor: None,
        })
    }

    async fn create_variant(
        &self,
        _ctx: &Context,
        _variant: &ProductVariantCreate,
    ) -> DomainResult<i64> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to create variant".to_string()));
        }
        Ok(self.id)
    }

    async fn update_variant(
        &self,
        _ctx: &Context,
        id: i64,
        _variant: &ProductVariantUpdate,
    ) -> DomainResult<()> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to update variant".to_string()));
        }
        if id != self.id {
            return Err(Error::NotFound(format!("Variant with id {} not found", id)));
        }
        Ok(())
    }

    async fn delete_variant(&self, _ctx: &Context, id: i64) -> DomainResult<()> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to delete variant".to_string()));
        }
        if id != self.id {
            return Err(Error::NotFound(format!("Variant with id {} not found", id)));
        }
        Ok(())
    }

    async fn delete_variants_by_product_id(
        &self,
        _ctx: &Context,
        product_id: i64,
    ) -> DomainResult<()> {
        if !self.should_succeed {
            return Err(Error::Internal(
                "Failed to delete variants by product id".to_string(),
            ));
        }
        if product_id != self.id {
            return Err(Error::NotFound(format!(
                "Product with id {} not found",
                product_id
            )));
        }
        Ok(())
    }

    async fn get_variant_by_barcode(
        &self,
        _ctx: &Context,
        barcode: &str,
    ) -> DomainResult<Option<ProductVariant>> {
        if !self.should_succeed {
            return Err(Error::Internal(
                "Failed to get variant by barcode".to_string(),
            ));
        }
        if barcode == "1234567890" {
            Ok(Some(self.create_mock_variant()))
        } else {
            Ok(None)
        }
    }

    async fn get_variant_by_id(
        &self,
        _ctx: &Context,
        id: i64,
    ) -> DomainResult<Option<ProductVariant>> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to get variant by id".to_string()));
        }
        if id == self.id {
            Ok(Some(self.create_mock_variant()))
        } else {
            Ok(None)
        }
    }

    async fn get_variant_by_product_id(
        &self,
        _ctx: &Context,
        product_id: i64,
    ) -> DomainResult<Vec<ProductVariant>> {
        if !self.should_succeed {
            return Err(Error::Internal(
                "Failed to get variants by product id".to_string(),
            ));
        }
        if product_id == self.id {
            Ok(vec![self.create_mock_variant()])
        } else {
            Ok(vec![])
        }
    }
}
