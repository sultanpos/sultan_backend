use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use sultan_core::domain::error::Error;

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        tracing::error!(error = ?self, "Request failed");

        match self {
            Error::Database(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
            }
            Error::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()
            }
            Error::NotFound(msg) => (StatusCode::NOT_FOUND, msg).into_response(),
            Error::Internal(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response()
            }
        }
    }
}
