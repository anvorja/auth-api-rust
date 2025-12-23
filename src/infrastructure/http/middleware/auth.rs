use axum::{
    extract::{State, Request},
    middleware::Next,
    response::Response,
    http::{StatusCode, header},
};
use crate::app::AppState;

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    let token = match token {
        Some(t) => t,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    match state.auth_usecase.jwt_service().verify_token(token) {
        Ok(token_data) => {
            req.extensions_mut().insert(token_data.claims);
            Ok(next.run(req).await)
        },
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}
