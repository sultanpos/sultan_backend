use async_trait::async_trait;
use sultan_core::application::PurchaseOrderServiceTrait;
use sultan_core::domain::model::purchase_order::{PurchaseOrder, PurchaseOrderUpdate};
use sultan_core::domain::{
    DomainResult, Error, context::Context, model::purchase_order::PurchaseOrderCreate,
};

pub struct MockPurchaseOrderService {
    pub should_succeed: bool,
    pub id: i64,
    pub purchase_order: Option<PurchaseOrder>,
}

impl MockPurchaseOrderService {
    pub fn new_success() -> Self {
        Self {
            should_succeed: true,
            id: 1,
            purchase_order: None,
        }
    }

    #[allow(dead_code)]
    pub fn new_success_with_purchase_order(purchase_order: PurchaseOrder) -> Self {
        Self {
            should_succeed: true,
            id: purchase_order.id,
            purchase_order: Some(purchase_order),
        }
    }

    #[allow(dead_code)]
    pub fn new_failure() -> Self {
        Self {
            should_succeed: false,
            id: 1,
            purchase_order: None,
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
        if !self.should_succeed {
            return Err(Error::Internal(
                "Failed to update purchase order".to_string(),
            ));
        }
        Ok(())
    }
    async fn delete(&self, _ctx: &Context, _branch_id: i64, _id: i64) -> DomainResult<()> {
        if !self.should_succeed {
            return Err(Error::Internal(
                "Failed to delete purchase order".to_string(),
            ));
        }
        Ok(())
    }
    async fn get(
        &self,
        _ctx: &Context,
        _branch_id: i64,
        _id: i64,
    ) -> DomainResult<Option<PurchaseOrder>> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to get purchase order".to_string()));
        }
        Ok(self.purchase_order.clone())
    }
}
