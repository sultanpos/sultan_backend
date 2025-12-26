use axum::{
    Json,
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;
use sultan_core::domain::BranchContext;

use crate::web::AppState;

/// Middleware to verify JWT Bearer token
pub async fn verify_jwt(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract Authorization header
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => {
            return Ok((
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Missing or invalid authorization header"})),
            )
                .into_response());
        }
    };

    // Verify token
    match state.jwt_manager.validate_token(token) {
        Ok(claims) => {
            let ctx = BranchContext::new();
            req.extensions_mut()
                .insert(ctx.with_user_id(claims.user_id));
            Ok(next.run(req).await)
        }
        Err(_) => Ok((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid or expired token"})),
        )
            .into_response()),
    }
}

/// Middleware to verify JWT Bearer token
pub async fn context_middleware(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let ctx = BranchContext::new();
    req.extensions_mut().insert(ctx);
    Ok(next.run(req).await)
}
