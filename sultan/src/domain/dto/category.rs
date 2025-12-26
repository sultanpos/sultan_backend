use serde::{Deserialize, Serialize};
use sultan_core::domain::model::Update;
use utoipa::ToSchema;
use validator::Validate;

/// Request to create a new category
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CategoryCreateRequest {
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
pub struct CategoryCreateResponse {
    /// Category ID
    #[schema(example = 1)]
    pub id: i64,
}

/// Request to update a new category
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CategoryUpdateRequest {
    /// Category name
    #[validate(length(
        min = 1,
        max = 100,
        message = "Name must be between 1 and 100 characters"
    ))]
    #[schema(example = "Electronics")]
    pub name: String,

    /// Category description (optional)
    #[schema(example = "Electronic devices and accessories")]
    pub description: Update<String>,

    /// Parent category ID (optional, for subcategories)
    #[schema(example = 1)]
    pub parent_id: Update<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CategoryResponse {
    pub name: String,
    pub description: Option<String>,
    pub children: Option<Vec<CategoryResponse>>,
}
