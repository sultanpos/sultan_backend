use super::i64_to_string;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sultan_core::domain::model::Update;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct BranchCreateRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Name must be between 1 and 100 characters"
    ))]
    #[schema(example = "Sultan Branch")]
    pub name: String,
    #[validate(length(
        min = 1,
        max = 50,
        message = "Code must be between 1 and 50 characters"
    ))]
    #[schema(example = "SULTAN")]
    pub code: String,
    #[schema(example = false)]
    #[serde(default)]
    pub is_main: bool,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub npwp: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BranchCreateResponse {
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub id: i64,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct BranchUpdateRequest {
    #[schema(example = "Sultan Branch")]
    pub name: Option<String>,
    #[schema(example = "SULTAN")]
    pub code: Option<String>,
    pub is_main: Option<bool>,
    #[schema(value_type = Option<String>)]
    pub address: Update<String>,
    #[schema(value_type = Option<String>)]
    pub phone: Update<String>,
    #[schema(value_type = Option<String>)]
    pub npwp: Update<String>,
    #[schema(value_type = Option<String>)]
    pub image: Update<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BranchResponse {
    #[schema(example = "1234567890", value_type = String)]
    #[serde(serialize_with = "i64_to_string")]
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub is_main: bool,
    pub name: String,
    pub code: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub npwp: Option<String>,
    pub image: Option<String>,
}

impl From<sultan_core::domain::model::branch::Branch> for BranchResponse {
    fn from(branch: sultan_core::domain::model::branch::Branch) -> Self {
        Self {
            id: branch.id,
            created_at: branch.created_at,
            updated_at: branch.updated_at,
            is_main: branch.is_main,
            name: branch.name,
            code: branch.code,
            address: branch.address,
            phone: branch.phone,
            npwp: branch.npwp,
            image: branch.image,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BranchListResponse {
    pub branches: Vec<BranchResponse>,
}
