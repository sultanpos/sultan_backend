use async_trait::async_trait;
use sultan_core::application::PaymentChannelServiceTrait;
use sultan_core::domain::{
    DomainResult, Error,
    context::Context,
    model::payment_channel::{
        PaymentChannel, PaymentChannelCreate, PaymentChannelFilter, PaymentChannelPriorityUpdate,
        PaymentChannelUpdate,
    },
};

pub struct MockPaymentChannelService {
    pub should_succeed: bool,
    pub id: i64,
}

impl MockPaymentChannelService {
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

fn mock_channel(id: i64) -> PaymentChannel {
    PaymentChannel {
        id,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        deleted_at: None,
        is_deleted: false,
        branch_id: None,
        name: "Cash".to_string(),
        priority: 1,
        metadata: None,
    }
}

#[async_trait]
impl PaymentChannelServiceTrait for MockPaymentChannelService {
    async fn create(&self, _ctx: &Context, _data: &PaymentChannelCreate) -> DomainResult<i64> {
        if !self.should_succeed {
            return Err(Error::Internal(
                "Failed to create payment channel".to_string(),
            ));
        }
        Ok(self.id)
    }

    async fn get_by_id(&self, _ctx: &Context, id: i64) -> DomainResult<Option<PaymentChannel>> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to get payment channel".to_string()));
        }
        if id != self.id {
            return Ok(None);
        }
        Ok(Some(mock_channel(id)))
    }

    async fn get_all(
        &self,
        _ctx: &Context,
        _filter: &PaymentChannelFilter,
    ) -> DomainResult<Vec<PaymentChannel>> {
        if !self.should_succeed {
            return Err(Error::Internal(
                "Failed to list payment channels".to_string(),
            ));
        }
        Ok(vec![mock_channel(self.id)])
    }

    async fn update(
        &self,
        _ctx: &Context,
        id: i64,
        _data: &PaymentChannelUpdate,
    ) -> DomainResult<()> {
        if !self.should_succeed {
            return Err(Error::Internal(
                "Failed to update payment channel".to_string(),
            ));
        }
        if id != self.id {
            return Err(Error::NotFound(format!(
                "Payment channel with id {} not found",
                id
            )));
        }
        Ok(())
    }

    async fn delete(&self, _ctx: &Context, id: i64) -> DomainResult<()> {
        if !self.should_succeed {
            return Err(Error::Internal(
                "Failed to delete payment channel".to_string(),
            ));
        }
        if id != self.id {
            return Err(Error::NotFound(format!(
                "Payment channel with id {} not found",
                id
            )));
        }
        Ok(())
    }

    async fn update_priorities(
        &self,
        _ctx: &Context,
        _updates: &[PaymentChannelPriorityUpdate],
    ) -> DomainResult<()> {
        if !self.should_succeed {
            return Err(Error::Internal(
                "Failed to update payment channel priorities".to_string(),
            ));
        }
        Ok(())
    }
}
