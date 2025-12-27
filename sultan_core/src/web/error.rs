use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

use crate::domain::Error;

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        tracing::error!(error = ?self, "Request failed");

        match self {
            Error::ValidationError(msg) => {
                (StatusCode::BAD_REQUEST, Json(json!({"error": msg}))).into_response()
            }
            Error::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database error"})),
            )
                .into_response(),
            Error::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid credentials"})),
            )
                .into_response(),
            Error::NotFound(msg) => {
                (StatusCode::NOT_FOUND, Json(json!({"error": msg}))).into_response()
            }
            Error::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Internal error"})),
            )
                .into_response(),
            Error::Unauthorized(msg) => {
                (StatusCode::UNAUTHORIZED, Json(json!({"error": msg}))).into_response()
            }
            Error::Forbidden(msg) => {
                (StatusCode::FORBIDDEN, Json(json!({"error": msg}))).into_response()
            }
            Error::Cancelled(msg) => {
                (StatusCode::REQUEST_TIMEOUT, Json(json!({"error": msg}))).into_response()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;

    async fn response_to_json(response: Response) -> serde_json::Value {
        let body = response.into_body();
        let bytes = body.collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn test_validation_error_response() {
        let error = Error::ValidationError("Invalid input".to_string());
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = response_to_json(response).await;
        assert_eq!(json["error"], "Invalid input");
    }

    #[tokio::test]
    async fn test_database_error_response() {
        let error = Error::Database("Connection failed".to_string());
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let json = response_to_json(response).await;
        assert_eq!(json["error"], "Database error");
    }

    #[tokio::test]
    async fn test_invalid_credentials_response() {
        let error = Error::InvalidCredentials;
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let json = response_to_json(response).await;
        assert_eq!(json["error"], "Invalid credentials");
    }

    #[tokio::test]
    async fn test_not_found_response() {
        let error = Error::NotFound("User not found".to_string());
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let json = response_to_json(response).await;
        assert_eq!(json["error"], "User not found");
    }

    #[tokio::test]
    async fn test_internal_error_response() {
        let error = Error::Internal("Something went wrong".to_string());
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let json = response_to_json(response).await;
        assert_eq!(json["error"], "Internal error");
    }

    #[tokio::test]
    async fn test_unauthorized_response() {
        let error = Error::Unauthorized("Not authenticated".to_string());
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let json = response_to_json(response).await;
        assert_eq!(json["error"], "Not authenticated");
    }

    #[tokio::test]
    async fn test_forbidden_response() {
        let error = Error::Forbidden("Access denied".to_string());
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        let json = response_to_json(response).await;
        assert_eq!(json["error"], "Access denied");
    }

    #[tokio::test]
    async fn test_cancelled_response() {
        let error = Error::Cancelled("Operation cancelled".to_string());
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);

        let json = response_to_json(response).await;
        assert_eq!(json["error"], "Operation cancelled");
    }
}
