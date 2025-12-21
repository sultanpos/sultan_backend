use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// Request to create a new category
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateCategoryRequest {
    /// Category name
    #[validate(length(
        min = 1,
        max = 100,
        message = "Name must be between 1 and 100 characters"
    ))]
    #[schema(example = "Electronics")]
    pub name: String,

    /// Category description (optional)
    #[validate(length(max = 500, message = "Description must not exceed 500 characters"))]
    #[schema(example = "Electronic devices and accessories")]
    pub description: Option<String>,

    /// Parent category ID (optional, for subcategories)
    #[schema(example = 1)]
    pub parent_id: Option<i64>,
}

/// Response after creating a category
#[derive(Debug, Serialize, ToSchema)]
pub struct CategoryResponse {
    /// Category ID
    #[schema(example = 1)]
    pub id: i64,

    /// Category name
    #[schema(example = "Electronics")]
    pub name: String,

    /// Category description
    #[schema(example = "Electronic devices and accessories")]
    pub description: Option<String>,

    /// Parent category ID
    #[schema(example = 1)]
    pub parent_id: Option<i64>,

    /// Creation timestamp
    #[schema(example = "2025-12-21T10:30:00Z")]
    pub created_at: String,
}
