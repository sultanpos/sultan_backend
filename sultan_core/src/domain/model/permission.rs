pub mod resource {
    pub const SUPER_ADMIN: i32 = 1;
    pub const ADMIN: i32 = 2;
    pub const BRANCH: i32 = 3;
    pub const USER: i32 = 4;
    pub const CATEGORY: i32 = 5;
    pub const SUPPLIER: i32 = 6;
    pub const CUSTOMER: i32 = 7;
    pub const PRODUCT: i32 = 8;
}

pub mod action {
    pub const CREATE: i32 = 1;
    pub const READ: i32 = 2;
    pub const UPDATE: i32 = 4;
    pub const DELETE: i32 = 8;
    // additional actions can be defined here
}

#[derive(Debug, Clone)]
pub struct Permission {
    pub user_id: i64,
    pub branch_id: Option<i64>,
    pub resource: i32,
    pub action: i32,
}
