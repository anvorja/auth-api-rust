use axum::{
    extract::State,
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::CookieJar;
use validator::Validate;
use crate::{
    app::AppState,
    presentation::dto::{RegisterUserRequest, LoginUserRequest, AuthResponse, UserResponse},
};

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    tag = "Auth",
    request_body = RegisterUserRequest,
    responses(
        (status = 201, description = "User registered successfully", body = UserResponse),
        (status = 400, description = "Validation error or User already exists"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterUserRequest>,
) -> impl IntoResponse {
    if let Err(e) = payload.validate() {
        return (StatusCode::BAD_REQUEST, format!("Validation error: {}", e)).into_response();
    }

    match state.auth_usecase.register_user(
        payload.name,
        payload.last_name,
        payload.email,
        payload.password,
    ).await {
        Ok(user) => {
             let response = UserResponse {
                id: user.id.to_string(),
                name: user.name,
                last_name: user.last_name,
                email: user.email,
            };
            (StatusCode::CREATED, Json(response)).into_response()
        },
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "Auth",
    request_body = LoginUserRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<LoginUserRequest>,
) -> impl IntoResponse {
    if let Err(e) = payload.validate() {
        return (StatusCode::BAD_REQUEST, format!("Validation error: {}", e)).into_response();
    }

    match state.auth_usecase.login_user(payload.email, payload.password).await {
        Ok((access_token, refresh_token)) => {
            let cookie = Cookie::build(("refresh_token", refresh_token))
                .http_only(true)
                .path("/api/v1/auth/refresh")
                .same_site(SameSite::Strict)
                // .secure(true) // TODO: Enable in production if SSL is enabled
                .build();

            (StatusCode::OK, jar.add(cookie), Json(AuthResponse { token: access_token })).into_response()
        },
        Err(_) => (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    tag = "Auth",
    responses(
        (status = 200, description = "Token refreshed", body = AuthResponse),
        (status = 401, description = "Invalid or missing refresh token")
    )
)]
pub async fn refresh(
    State(state): State<AppState>,
    jar: CookieJar,
) -> impl IntoResponse {
    let refresh_token = match jar.get("refresh_token") {
        Some(cookie) => cookie.value(),
        None => return (StatusCode::UNAUTHORIZED, "Missing refresh token").into_response(),
    };

    match state.auth_usecase.refresh_access_token(refresh_token) {
        Ok(new_token) => (StatusCode::OK, Json(AuthResponse { token: new_token })).into_response(),
        Err(_) => (StatusCode::UNAUTHORIZED, "Invalid refresh token").into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    tag = "Auth",
    responses(
        (status = 200, description = "Logged out successfully")
    )
)]
pub async fn logout(jar: CookieJar) -> impl IntoResponse {
    let cookie = Cookie::build("refresh_token")
        .path("/api/v1/auth/refresh")
        .removal()
        .build();

    (StatusCode::OK, jar.add(cookie), "Logged out").into_response()
}
