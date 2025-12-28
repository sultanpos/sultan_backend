use async_trait::async_trait;
use sultan_core::application::SupplierServiceTrait;
use sultan_core::domain::{
    DomainResult, Error,
    context::Context,
    model::{
        pagination::PaginationOptions,
        supplier::{Supplier, SupplierCreate, SupplierFilter, SupplierUpdate},
    },
};

pub struct MockSupplierService {
    pub should_succeed: bool,
    pub id: i64,
}

impl MockSupplierService {
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
impl SupplierServiceTrait for MockSupplierService {
    async fn create(&self, _ctx: &Context, _supplier: &SupplierCreate) -> DomainResult<i64> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to create supplier".to_string()));
        }
        Ok(self.id)
    }

    async fn update(
        &self,
        _ctx: &Context,
        id: i64,
        _supplier: &SupplierUpdate,
    ) -> DomainResult<()> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to update supplier".to_string()));
        }
        // Return NotFound if id is not 1
        if id != 1 {
            return Err(Error::NotFound(format!(
                "Supplier with id {} not found",
                id
            )));
        }
        Ok(())
    }

    async fn delete(&self, _ctx: &Context, id: i64) -> DomainResult<()> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to delete supplier".to_string()));
        }
        // Return NotFound if id is not 1
        if id != 1 {
            return Err(Error::NotFound(format!(
                "Supplier with id {} not found",
                id
            )));
        }
        Ok(())
    }

    async fn get_by_id(&self, _ctx: &Context, id: i64) -> DomainResult<Option<Supplier>> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to get supplier".to_string()));
        }

        // Return mock supplier if id is 1
        if id == 1 {
            Ok(Some(Supplier {
                id: 1,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                deleted_at: None,
                is_deleted: false,
                name: "PT. Test Supplier".to_string(),
                code: Some("SUP001".to_string()),
                email: Some("supplier@test.com".to_string()),
                address: Some("Jakarta".to_string()),
                phone: Some("08123456789".to_string()),
                npwp: Some("12.345.678.9-012.000".to_string()),
                npwp_name: Some("PT Test Supplier".to_string()),
                metadata: None,
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_all(
        &self,
        _ctx: &Context,
        filter: &SupplierFilter,
        _pagination: &PaginationOptions,
    ) -> DomainResult<Vec<Supplier>> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to get suppliers".to_string()));
        }

        // Return empty if name filter is "empty"
        if let Some(name) = &filter.name
            && name == "empty"
        {
            return Ok(vec![]);
        }

        // Return filtered results if filter is provided
        let mut suppliers = vec![
            Supplier {
                id: 1,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                deleted_at: None,
                is_deleted: false,
                name: "PT. Test Supplier".to_string(),
                code: Some("SUP001".to_string()),
                email: Some("supplier@test.com".to_string()),
                address: Some("Jakarta".to_string()),
                phone: Some("08123456789".to_string()),
                npwp: Some("12.345.678.9-012.000".to_string()),
                npwp_name: Some("PT Test Supplier".to_string()),
                metadata: None,
            },
            Supplier {
                id: 2,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                deleted_at: None,
                is_deleted: false,
                name: "CV. Another Supplier".to_string(),
                code: Some("SUP002".to_string()),
                email: Some("another@test.com".to_string()),
                address: Some("Bandung".to_string()),
                phone: Some("08198765432".to_string()),
                npwp: None,
                npwp_name: None,
                metadata: None,
            },
        ];

        // Apply filters
        if let Some(name) = &filter.name {
            suppliers.retain(|s| s.name.contains(name));
        }
        if let Some(code) = &filter.code {
            suppliers.retain(|s| s.code.as_ref().is_some_and(|c| c.contains(code)));
        }
        if let Some(phone) = &filter.phone {
            suppliers.retain(|s| s.phone.as_ref().is_some_and(|p| p.contains(phone)));
        }
        if let Some(email) = &filter.email {
            suppliers.retain(|s| s.email.as_ref().is_some_and(|e| e.contains(email)));
        }
        if let Some(npwp) = &filter.npwp {
            suppliers.retain(|s| s.npwp.as_ref().is_some_and(|n| n.contains(npwp)));
        }

        Ok(suppliers)
    }
}
