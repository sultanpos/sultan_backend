use chrono::Utc;

#[derive(Debug, Clone)]
pub struct Token {
    pub id: i64,
    pub expired_at: chrono::DateTime<Utc>,
    pub user_id: i64,
    pub token: String,
}
