use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{debug_handler, extract::State, Json};
use axum_extra::extract::cookie::{Cookie, PrivateCookieJar};
use entity::{sessions, user, user_pass};
use sea_orm::{entity::*, query::*};
use tracing::instrument;
use uuid::Uuid;

use super::structs::{LoginRequest, LoginResponse, NewUserRequest, NewUserResponse};
use crate::state::ServerState;

#[instrument]
#[debug_handler]
#[utoipa::path(
    post,
    path = "/register",
    context_path = super::LOCAL_PREFIX,
    request_body = NewUserRequest,
    responses(NewUserResponse),
    tag = super::LOCAL_TAG,
)]
pub async fn add_local_user(
    State(state): State<ServerState>,
    Json(payload): Json<NewUserRequest>,
) -> Result<NewUserResponse, NewUserResponse> {
    let new_user = user::ActiveModel {
        user_id: ActiveValue::NotSet,
        name: ActiveValue::set(payload.name),
        email: ActiveValue::set(payload.email),
        created_at: ActiveValue::set(chrono::Utc::now().naive_utc()),
        updated_at: ActiveValue::set(chrono::Utc::now().naive_utc()),
    };
    let new = new_user.insert(&state.conn).await?;

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)?
        .to_string();

    let new_user_pass = user_pass::ActiveModel {
        id: ActiveValue::NotSet,
        user_id: ActiveValue::set(new.user_id),
        password: ActiveValue::set(password_hash),
    };
    new_user_pass.insert(&state.conn).await?;

    Ok(NewUserResponse::UserCreated(new))
}

#[instrument]
#[debug_handler]
#[utoipa::path(
    post,
    path = "/login",
    context_path = super::LOCAL_PREFIX,
    request_body = LoginRequest,
    responses(LoginResponse),
    tag = super::LOCAL_TAG,
)]
pub async fn local_login(
    State(state): State<ServerState>,
    jar: PrivateCookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<LoginResponse, LoginResponse> {
    let user = user::Entity::find()
        .filter(user::Column::Email.eq(payload.email))
        .one(&state.conn)
        .await?;

    let user = match user {
        Some(user) => user,
        None => {
            return Err(LoginResponse::InvalidCredentials(
                "Unknown user".to_string().into(),
            ))
        }
    };

    let user_pass = user_pass::Entity::find()
        .filter(user_pass::Column::UserId.eq(user.user_id))
        .one(&state.conn)
        .await?;

    let user_pass = match user_pass {
        Some(user_pass) => user_pass,
        None => {
            return Err(LoginResponse::InvalidCredentials(
                "Unknown user password".to_string().into(),
            ))
        }
    };

    let argon2 = Argon2::default();
    let password_hash = PasswordHash::new(&user_pass.password).unwrap();
    argon2.verify_password(payload.password.as_bytes(), &password_hash)?;

    let session_id = Uuid::new_v4().to_string();
    let expiry = chrono::Utc::now().naive_utc() + chrono::Duration::days(1);

    let session = sessions::ActiveModel {
        id: ActiveValue::NotSet,
        session_id: ActiveValue::set(session_id),
        user_id: ActiveValue::set(user.user_id),
        expiry: ActiveValue::set(expiry),
    };

    let session = session.insert(&state.conn).await?;

    let Some(domain) = state.url.domain() else {
        return Err(LoginResponse::InternalServerError(
            "Domain not set".to_string().into(),
        ));
    };

    let cookie = Cookie::build(("sid", session.session_id))
        .domain(format!(".{}", domain))
        .path("/")
        .secure(true)
        .http_only(true);

    Ok(LoginResponse::UserLoggedIn(
        user,
        jar.add(cookie)
            .remove("verifier")
            .remove("nonce")
            .remove("csrf_token"),
    ))
}
