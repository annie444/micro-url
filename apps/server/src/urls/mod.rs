pub mod routes;
pub mod structs;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::state::ServerState;

pub const URL_TAG: &str = "urls";
pub const URL_PREFIX: &str = "/api/url";

pub fn url_router(state: ServerState) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(routes::new_url))
        .routes(routes!(routes::delete_url))
        .routes(routes!(routes::update_url))
        .routes(routes!(routes::url_info))
        .routes(routes!(routes::get_url))
        .with_state(state)
}
