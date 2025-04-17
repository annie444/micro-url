use axum::Router;
use entity::{short_link, user as entity_user};
use serde::Serialize;
use utoipa::{
    Modify, OpenApi, openapi,
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

use crate::{state::ServerState, urls, user, utils};

#[derive(Debug, Serialize)]
pub struct SecurityDef;

impl Modify for SecurityDef {
    fn modify(&self, openapi: &mut openapi::OpenApi) {
        if let Some(schema) = openapi.components.as_mut() {
            schema.add_security_scheme(
                "session_id",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::with_description(
                    "sid".to_string(),
                    "Session ID cookie".to_string(),
                ))),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityDef),
    components(
        schemas(
            entity_user::Model,
            short_link::Model,
            user::structs::NewUserRequest,
            user::structs::LoginRequest,
            user::structs::UserLink,
            user::structs::UserLinksAndViews,
            user::structs::OidcName,
            urls::structs::NewUrlRequest,
            utils::BasicError,
            utils::BasicResponse
        ),
    ),
    tags(
        (name = urls::URL_TAG, description = "URL API routes"),
        (name = user::USER_TAG, description = "User API routes"),
        (name = user::OIDC_TAG, description = "OIDC users API routes"),
        (name = user::LOCAL_TAG, description = "Local users API routes"),
    )
)]
pub struct ApiDoc;

/// Get health of the API.
#[utoipa::path(
    method(get, head),
    path = "/api/health",
    responses(
        (status = OK, description = "Success", body = str, content_type = "text/plain")
    )
)]
async fn health() -> &'static str {
    "ok"
}

pub fn router(state: ServerState) -> Router {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health))
        .merge(urls::url_router(state.clone()))
        .merge(user::user_router(state.clone()))
        .merge(user::oidc_router(state.clone()))
        .merge(user::local_router(state.clone()))
        .split_for_parts();

    router.merge(SwaggerUi::new("/api/ui/swagger").url("/api/doc/openapi.json", api))
}
