use async_trait::async_trait;
use sultan_core::application::PurchaseOrderServiceTrait;
use sultan_core::domain::model::purchase_order::PurchaseOrderUpdate;
use sultan_core::domain::{
    DomainResult, Error, context::Context, model::purchase_order::PurchaseOrderCreate,
};

pub struct MockPurchaseOrderService {
    pub should_succeed: bool,
    pub id: i64,
}

impl MockPurchaseOrderService {
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
}

#[async_trait]
impl PurchaseOrderServiceTrait for MockPurchaseOrderService {
    async fn create(&self, _ctx: &Context, _data: &PurchaseOrderCreate) -> DomainResult<i64> {
        if !self.should_succeed {
            return Err(Error::Internal(
                "Failed to create purchase order".to_string(),
            ));
        }
        Ok(self.id)
    }
    async fn update(
        &self,
        _ctx: &Context,
        _branch_id: i64,
        _id: i64,
        _data: &PurchaseOrderUpdate,
    ) -> DomainResult<()> {
        Ok(())
    }
}
