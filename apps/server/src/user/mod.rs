pub mod local;
pub mod oidc;
pub mod routes;
pub mod structs;

use std::sync::LazyLock;

use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::state::ServerState;

pub const OIDC_TAG: &'static str = "oidc-users";
pub const USER_TAG: &'static str = "user";
pub const LOCAL_TAG: &'static str = "local-users";
pub const OIDC_PREFIX: &'static str = "/api/user/oidc";
pub const USER_PREFIX: &'static str = "/api/user";
pub const LOCAL_PREFIX: &'static str = "/api/user/local";

pub static SECURITY: LazyLock<SecurityScheme> = LazyLock::new(|| {
    SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::with_description(
        "sid".to_string(),
        "Session ID cookie".to_string(),
    )))
});

pub fn oidc_router(state: ServerState) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(oidc::get_oidc_provider))
        .routes(routes!(oidc::oidc_callback))
        .routes(routes!(oidc::oidc_login))
        .with_state(state)
}

pub fn local_router(state: ServerState) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(local::add_local_user))
        .routes(routes!(local::local_login))
        .with_state(state)
}

pub fn user_router(state: ServerState) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(routes::get_user))
        .routes(routes!(routes::logout))
        .routes(routes!(routes::get_user_urls))
        .with_state(state)
        .with_security_scheme(SecurityScheme::ApiKey(ApiKey::Cookie(
            ApiKeyValue::with_description("sid".to_string(), "Session ID cookie".to_string()),
        )))
}
