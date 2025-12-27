#[derive(Debug, Clone)]
pub struct PaginationOrder {
    pub field: String,
    pub direction: String,
}

#[derive(Debug, Clone)]
pub struct PaginationOptions {
    pub page: u32,
    pub page_size: u32,
    pub order: Option<PaginationOrder>,
}

impl PaginationOptions {
    pub fn new(page: u32, page_size: u32, order: Option<PaginationOrder>) -> Self {
        Self {
            page,
            page_size,
            order,
        }
    }

    pub fn offset(&self) -> u32 {
        (self.page - 1) * self.page_size
    }

    pub fn limit(&self) -> u32 {
        self.page_size
    }
}
