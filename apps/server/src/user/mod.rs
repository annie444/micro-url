pub mod local;
pub mod oidc;
pub mod routes;
pub mod structs;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::state::ServerState;

pub const OIDC_TAG: &str = "oidc-users";
pub const USER_TAG: &str = "user";
pub const LOCAL_TAG: &str = "local-users";
pub const OIDC_PREFIX: &str = "/api/user/oidc";
pub const USER_PREFIX: &str = "/api/user";
pub const LOCAL_PREFIX: &str = "/api/user/local";

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
}
