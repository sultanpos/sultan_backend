use chrono::Utc;
use serde_json::Value;

use super::Update;

#[derive(Debug, Clone)]
pub struct Customer {
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub deleted_at: Option<chrono::DateTime<Utc>>,
    pub is_deleted: bool,
    pub number: String,
    pub name: String,
    pub address: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub level: i32,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct CustomerCreate {
    pub number: String,
    pub name: String,
    pub address: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub level: i32,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Default)]
pub struct CustomerUpdate {
    pub number: Option<String>,
    pub name: Option<String>,
    pub address: Update<String>,
    pub email: Update<String>,
    pub phone: Update<String>,
    pub level: Option<i32>,
    pub metadata: Update<Value>,
}

#[derive(Debug, Clone, Default)]
pub struct CustomerFilter {
    pub number: Option<String>,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub level: Option<i32>,
}
