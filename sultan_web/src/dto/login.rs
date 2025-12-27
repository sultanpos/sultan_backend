use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(length(min = 1, message = "Username cannot be empty"))]
    #[schema(example = "admin")]
    pub username: String,

    #[validate(length(min = 1, message = "Password cannot be empty"))]
    #[schema(example = "password123")]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LoginResponse {
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub access_token: String,

    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub refresh_token: String,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct RefreshTokenRequest {
    #[validate(length(min = 1, message = "Refresh token cannot be empty"))]
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub refresh_token: String,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct LogoutRequest {
    #[validate(length(min = 1, message = "Refresh token cannot be empty"))]
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub refresh_token: String,
}
