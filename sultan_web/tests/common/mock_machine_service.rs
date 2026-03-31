use async_trait::async_trait;
use chrono::Utc;
use sultan_core::application::MachineServiceTrait;
use sultan_core::domain::{
    DomainResult, Error,
    context::Context,
    model::machine::{
        Machine, MachineCreate, MachineCursor, MachineFilter, MachinePage, MachineQuery,
        MachineSortField, MachineUpdate,
    },
    model::product::SortDirection,
};

pub struct MockMachineService {
    pub should_succeed: bool,
    pub id: i64,
}

impl MockMachineService {
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

fn sample_machine(id: i64) -> Machine {
    Machine {
        id,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
        is_deleted: false,
        branch_id: 1,
        key: format!("POS-0{}", id),
        name: format!("Counter {}", id),
        description: Some("Main counter".to_string()),
        metadata: None,
    }
}

#[async_trait]
impl MachineServiceTrait for MockMachineService {
    async fn create(&self, _ctx: &Context, _machine: &MachineCreate) -> DomainResult<i64> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to create machine".to_string()));
        }
        Ok(self.id)
    }

    async fn update(&self, _ctx: &Context, id: i64, _machine: &MachineUpdate) -> DomainResult<()> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to update machine".to_string()));
        }
        if id != 1 {
            return Err(Error::NotFound(format!("Machine with id {} not found", id)));
        }
        Ok(())
    }

    async fn delete(&self, _ctx: &Context, id: i64) -> DomainResult<()> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to delete machine".to_string()));
        }
        if id != 1 {
            return Err(Error::NotFound(format!("Machine with id {} not found", id)));
        }
        Ok(())
    }

    async fn get_by_id(&self, _ctx: &Context, id: i64) -> DomainResult<Option<Machine>> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to get machine".to_string()));
        }
        if id == 1 {
            Ok(Some(sample_machine(1)))
        } else {
            Ok(None)
        }
    }

    async fn get_all(&self, _ctx: &Context, query: &MachineQuery) -> DomainResult<MachinePage> {
        if !self.should_succeed {
            return Err(Error::Internal("Failed to list machines".to_string()));
        }

        // Return empty if name filter is "empty"
        if let Some(name) = &query.filter.name
            && name == "empty"
        {
            return Ok(MachinePage {
                items: vec![],
                next_cursor: None,
            });
        }

        Ok(MachinePage {
            items: vec![sample_machine(1), sample_machine(2)],
            next_cursor: None,
        })
    }
}

/// Default mock query for tests
#[allow(dead_code)]
pub fn default_machine_query() -> MachineQuery {
    MachineQuery {
        filter: MachineFilter {
            branch_id: None,
            name: None,
        },
        sort_field: MachineSortField::CreatedAt,
        sort_direction: SortDirection::Desc,
        cursor: None,
        limit: 20,
    }
}

/// Sample cursor for pagination tests
#[allow(dead_code)]
pub fn sample_cursor() -> MachineCursor {
    MachineCursor {
        field_value: "Counter 1".to_string(),
        id: 1,
    }
}
