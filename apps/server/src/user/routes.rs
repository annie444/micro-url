use axum::{debug_handler, extract::State};
use axum_extra::extract::cookie::PrivateCookieJar;
use entity::{sessions, short_link, user, views};
use sea_orm::{entity::*, query::*};
use tracing::instrument;

use super::structs::{LogoutResponse, UserLinksResponse, UserProfile, UserProfileResponse};
use crate::state::ServerState;

// /auth/logout
#[instrument]
#[debug_handler]
#[utoipa::path(
    get,
    path = "/logout",
    context_path = super::USER_PREFIX,
    responses(LogoutResponse),
    tags = [super::LOCAL_TAG, super::OIDC_TAG],
    security(("session_id" = []))
)]
pub async fn logout(
    jar: PrivateCookieJar,
    State(state): State<ServerState>,
) -> Result<LogoutResponse, LogoutResponse> {
    let Some(cookie) = jar.get("sid") else {
        return Err(LogoutResponse::UserNotLoggedIn(
            "User not logged in".to_string().into(),
        ));
    };

    let session = sessions::Entity::find()
        .filter(sessions::Column::SessionId.eq(cookie.value()))
        .one(&state.conn)
        .await?;

    let session = match session {
        Some(session) => session,
        None => {
            return Err(LogoutResponse::SessionNotFound(
                "User session not found".to_string().into(),
            ));
        }
    };

    session.delete(&state.conn).await?;

    Ok(LogoutResponse::UserLoggedOut(
        "User logged out".to_string().into(),
        jar.remove("sid"),
    ))
}

// /api/user
#[instrument]
#[debug_handler]
#[utoipa::path(
    get,
    path = "",
    context_path = super::USER_PREFIX,
    responses(UserProfileResponse),
    tag = super::USER_TAG,
    security(("session_id" = []))
)]
pub async fn get_user(
    jar: PrivateCookieJar,
    State(state): State<ServerState>,
) -> Result<UserProfileResponse, UserProfileResponse> {
    let Some(cookie) = jar.get("sid").map(|cookie| cookie.value().to_owned()) else {
        return Err(UserProfileResponse::InvalidSession(
            "User not logged in".to_string().into(),
        ));
    };

    let Some(res) = sessions::Entity::find()
        .left_join(user::Entity)
        .columns([user::Column::Email, user::Column::Name])
        .filter(sessions::Column::SessionId.eq(cookie))
        .into_partial_model::<UserProfile>()
        .one(&state.conn)
        .await?
    else {
        return Err(UserProfileResponse::InvalidSession(
            "User not found".to_string().into(),
        ));
    };

    Ok(UserProfileResponse::UserProfile(res))
}

// /api/user/urls
#[instrument]
#[debug_handler]
#[utoipa::path(
    get,
    path = "/urls",
    context_path = super::USER_PREFIX,
    responses(UserLinksResponse),
    tag = super::USER_TAG,
    security(("session_id" = [])),
)]
pub async fn get_user_urls(
    jar: PrivateCookieJar,
    State(state): State<ServerState>,
) -> Result<UserLinksResponse, UserLinksResponse> {
    let Some(cookie) = jar.get("sid").map(|cookie| cookie.value().to_owned()) else {
        return Err(UserLinksResponse::InvalidSession(
            "User not logged in".to_string().into(),
        ));
    };

    let Some(res) = sessions::Entity::find()
        .left_join(user::Entity)
        .columns([user::Column::Email, user::Column::Name])
        .filter(sessions::Column::SessionId.eq(cookie))
        .one(&state.conn)
        .await?
    else {
        return Err(UserLinksResponse::InvalidSession(
            "User not found".to_string().into(),
        ));
    };

    let res = short_link::Entity::find()
        .filter(short_link::Column::UserId.eq(res.user_id))
        .find_with_related(views::Entity)
        .all(&state.conn)
        .await?;

    Ok(UserLinksResponse::UserLinks(res.into()))
}
