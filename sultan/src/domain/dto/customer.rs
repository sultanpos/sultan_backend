use serde::{Deserialize, Serialize};
use serde_json::Value;
use sultan_core::domain::model::Update;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CustomerCreateRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Name must be between 1 and 100 characters"
    ))]
    #[schema(example = "CV. Sultan Pos")]
    pub name: String,
    pub number: Option<String>,
    pub address: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub level: i32,
    pub metadata: Option<Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CustomerCreateResponse {
    #[schema(example = 1)]
    pub id: i64,
}
