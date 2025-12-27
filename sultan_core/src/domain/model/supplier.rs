use chrono::Utc;
use serde_json::Value;

use super::Update;

#[derive(Debug, Clone)]
pub struct Supplier {
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub deleted_at: Option<chrono::DateTime<Utc>>,
    pub is_deleted: bool,
    pub name: String,
    pub code: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub npwp: Option<String>,
    pub npwp_name: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct SupplierCreate {
    pub name: String,
    pub code: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub npwp: Option<String>,
    pub npwp_name: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Default)]
pub struct SupplierUpdate {
    pub name: Option<String>,
    pub code: Update<String>,
    pub email: Update<String>,
    pub address: Update<String>,
    pub phone: Update<String>,
    pub npwp: Update<String>,
    pub npwp_name: Update<String>,
    pub metadata: Update<Value>,
}

#[derive(Debug, Clone)]
pub struct SupplierFilter {
    pub name: Option<String>,
    pub code: Option<String>,
    pub phone: Option<String>,
    pub npwp: Option<String>,
    pub email: Option<String>,
}
