use super::{i64_to_string, option_string_to_i64, update_string_to_i64};
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
    #[schema(example = "1234567890", value_type = String)]
    #[serde(default, deserialize_with = "option_string_to_i64")]
    pub parent_id: Option<i64>,
}

/// Response after creating a category
#[derive(Debug, Serialize, ToSchema)]
pub struct CategoryCreateResponse {
    /// Category ID
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
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
    #[schema(example = "Electronic devices and accessories", value_type = Option<String>)]
    pub description: Update<String>,

    /// Parent category ID (optional, for subcategories)
    #[schema(example = "1234567890", value_type = Option<String>)]
    #[serde(default, deserialize_with = "update_string_to_i64")]
    pub parent_id: Update<i64>,
}

/// Child category response (simplified, no recursion)
#[derive(Debug, Serialize, ToSchema)]
pub struct CategoryChildResponse {
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub id: i64,
    /// Category name
    #[schema(example = "Laptops")]
    pub name: String,

    /// Category description
    #[schema(example = "Laptops and Notebooks")]
    pub description: Option<String>,
}

/// Category response with hierarchical structure
#[derive(Debug, Serialize, ToSchema)]
pub struct CategoryResponse {
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub id: i64,
    /// Category name
    #[schema(example = "Electronics")]
    pub name: String,

    /// Category description
    #[schema(example = "Electronic devices and accessories")]
    pub description: Option<String>,

    /// Child categories (one level deep)
    pub children: Option<Vec<CategoryChildResponse>>,
}
