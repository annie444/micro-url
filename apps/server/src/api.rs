use axum::Router;
use entity::{short_link, user as entity_user};
use serde::Serialize;
use utoipa::{
    openapi,
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

use crate::{state::ServerState, structs, urls, user};

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
            user::structs::UserProfile,
            user::structs::UserLinks,
            user::structs::OidcName,
            structs::BasicError, 
            structs::BasicResponse
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

    let router = router.merge(SwaggerUi::new("/swagger-ui").url("/apidoc/openapi.json", api));
    router
}
