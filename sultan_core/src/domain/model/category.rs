use chrono::Utc;

#[derive(Debug, Clone)]
pub struct Category {
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub deleted_at: Option<chrono::DateTime<Utc>>,
    pub is_deleted: bool,
    pub name: String,
    pub description: Option<String>,
    pub children: Option<Vec<Category>>,
}

#[derive(Debug, Clone)]
pub struct CategoryCreate {
    pub parent_id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CategoryUpdate {
    pub parent_id: super::Update<i64>,
    pub name: Option<String>,
    pub description: super::Update<String>,
}
