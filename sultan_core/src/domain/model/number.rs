use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a number sequence in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumberSequence {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub prefix: String,
    pub branch_id: Option<i64>,
    pub year: i32,
    pub month: Option<i32>,
    pub last_number: i32,
}

/// Parameters for generating a new number
#[derive(Debug, Clone)]
pub struct NumberGenerateParams {
    pub prefix: String,
    pub branch_id: Option<i64>,
    pub year: i32,
    pub month: Option<i32>,
}
