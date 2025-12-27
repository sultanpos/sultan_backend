use chrono::Utc;

use super::Update;

#[derive(Debug, Clone)]
pub struct Branch {
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub deleted_at: Option<chrono::DateTime<Utc>>,
    pub is_deleted: bool,
    pub is_main: bool,
    pub name: String,
    pub code: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub npwp: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BranchCreate {
    pub is_main: bool,
    pub name: String,
    pub code: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub npwp: Option<String>,
    pub image: Option<String>,
}

/// Represents a partial update for a Branch.
///
/// Each field uses `Update<T>` to distinguish between:
/// - `Update::Unchanged` - Don't modify the field
/// - `Update::Set(value)` - Set the field to a new value
/// - `Update::Clear` - Set the field to NULL (only for nullable fields)
///
/// # Example
/// ```ignore
/// let update = BranchUpdate {
///     name: Update::Set("New Name".to_string()),
///     address: Update::Clear,  // Set address to NULL
///     ..Default::default()     // All other fields unchanged
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct BranchUpdate {
    pub is_main: Option<bool>,
    pub name: Option<String>,
    pub code: Option<String>,
    pub address: Update<String>,
    pub phone: Update<String>,
    pub npwp: Update<String>,
    pub image: Update<String>,
}
