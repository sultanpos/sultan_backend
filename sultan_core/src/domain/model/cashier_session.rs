use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Status of a cashier session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    #[default]
    Open,
    Closed,
}

impl SessionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            SessionStatus::Open => "open",
            SessionStatus::Closed => "closed",
        }
    }
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for SessionStatus {
    type Err = crate::domain::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "open" => Ok(SessionStatus::Open),
            "closed" => Ok(SessionStatus::Closed),
            other => Err(crate::domain::Error::ValidationError(format!(
                "Invalid session status '{}'. Must be one of: open, closed",
                other
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CashierSession {
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub deleted_at: Option<chrono::DateTime<Utc>>,
    pub is_deleted: bool,
    pub branch_id: i64,
    pub user_id: i64,
    pub opened_at: chrono::DateTime<Utc>,
    pub closed_at: Option<chrono::DateTime<Utc>>,
    pub status: SessionStatus,
    pub opening_cash: i64,
    pub closing_cash: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CashierSessionCreate {
    pub branch_id: i64,
    pub user_id: i64,
    pub opening_cash: i64,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CashierSessionClose {
    pub closing_cash: i64,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct CashierSessionFilter {
    pub branch_id: Option<i64>,
    pub user_id: Option<i64>,
    pub status: Option<SessionStatus>,
}
