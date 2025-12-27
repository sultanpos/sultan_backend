use async_trait::async_trait;
use sultan_core::application::CustomerServiceTrait;
use sultan_core::domain::model::pagination::PaginationOptions;
use sultan_core::domain::{
    DomainResult, Error,
    context::Context,
    model::customer::{Customer, CustomerCreate, CustomerFilter, CustomerUpdate},
};

pub struct MockCustomerService {
    pub should_succeed: bool,
    pub id: i64,
    pub return_empty: bool,
}

impl MockCustomerService {
    pub fn new_success() -> Self {
        Self {
            should_succeed: true,
            id: 1,
            return_empty: false,
        }
    }

    #[allow(dead_code)]
    pub fn new_failure() -> Self {
        Self {
            should_succeed: false,
            id: 1,
            return_empty: false,
        }
    }

    #[allow(dead_code)]
    pub fn new_empty() -> Self {
        Self {
            should_succeed: true,
            id: 1,
            return_empty: true,
        }
    }
}

#[async_trait]
impl CustomerServiceTrait for MockCustomerService {
    async fn create(&self, _ctx: &Context, _customer: &CustomerCreate) -> DomainResult<i64> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to create customer".to_string()));
        }
        Ok(self.id)
    }

    async fn update(
        &self,
        _ctx: &Context,
        id: i64,
        _customer: &CustomerUpdate,
    ) -> DomainResult<()> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to update customer".to_string()));
        }
        if id != 1 {
            return Err(Error::NotFound(format!(
                "Customer with id {} not found",
                id
            )));
        }
        Ok(())
    }

    async fn delete(&self, _ctx: &Context, id: i64) -> DomainResult<()> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to delete customer".to_string()));
        }
        if id != 1 {
            return Err(Error::NotFound(format!(
                "Customer with id {} not found",
                id
            )));
        }
        Ok(())
    }

    async fn get_by_number(&self, _ctx: &Context, number: &str) -> DomainResult<Option<Customer>> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to get customer".to_string()));
        }
        if number == "CUST001" {
            Ok(Some(create_mock_customer(self.id, "CUST001", "John Doe")))
        } else {
            Ok(None)
        }
    }

    async fn get_by_id(&self, _ctx: &Context, id: i64) -> DomainResult<Option<Customer>> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to get customer".to_string()));
        }
        if id == 1 {
            Ok(Some(create_mock_customer(self.id, "CUST001", "John Doe")))
        } else {
            Ok(None)
        }
    }

    async fn get_all(
        &self,
        _ctx: &Context,
        _filter: &CustomerFilter,
        _pagination: &PaginationOptions,
    ) -> DomainResult<Vec<Customer>> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to get customers".to_string()));
        }
        if self.return_empty {
            return Ok(vec![]);
        }
        Ok(vec![
            create_mock_customer(1, "CUST001", "John Doe"),
            create_mock_customer(2, "CUST002", "Jane Smith"),
        ])
    }
}

fn create_mock_customer(id: i64, number: &str, name: &str) -> Customer {
    Customer {
        id,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        deleted_at: None,
        is_deleted: false,
        number: number.to_string(),
        name: name.to_string(),
        address: Some("123 Test St".to_string()),
        email: Some("test@customer.com".to_string()),
        phone: Some("555-1234".to_string()),
        level: 1,
        metadata: None,
    }
}
