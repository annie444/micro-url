use std::cmp::PartialEq;

use axum::{
    debug_handler,
    extract::{Path, Query, State},
    response::Redirect,
    Json,
};
use axum_extra::extract::cookie::{Cookie, PrivateCookieJar};
use chrono::{Duration, Local};
use entity::{sessions, short_link, user, user_pass};
use openidconnect::{
    core::{CoreAuthenticationFlow, CoreGenderClaim},
    AccessTokenHash, AuthorizationCode, CsrfToken, EmptyAdditionalClaims, Nonce,
    OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, TokenResponse, UserInfoClaims,
};
use sea_orm::{entity::*, query::*};
use time::Duration as TimeDuration;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    error::ServerError,
    state::ServerState,
    structs::{AuthRequest, NewUserRequest, UserProfile},
};

#[instrument]
#[debug_handler]
pub async fn add_local_user(
    State(state): State<ServerState>,
    Json(payload): Json<NewUserRequest>,
) -> Result<Json<user::Model>, ServerError> {
    let new_user = user::ActiveModel {
        user_id: ActiveValue::NotSet,
        name: ActiveValue::set(payload.name),
        email: ActiveValue::set(payload.email),
        created_at: ActiveValue::set(chrono::Utc::now().naive_utc()),
        updated_at: ActiveValue::set(chrono::Utc::now().naive_utc()),
    };
    let new = new_user.insert(&state.conn).await?;

    let new_user_pass = user_pass::ActiveModel {
        id: ActiveValue::NotSet,
        user_id: ActiveValue::set(new.user_id),
        password: ActiveValue::set(payload.password),
    };
    new_user_pass.insert(&state.conn).await?;

    Ok(Json(new))
}

#[instrument]
#[debug_handler]
pub async fn user_info(
    user: UserProfile,
    State(state): State<ServerState>,
) -> Result<Json<UserProfile>, ServerError> {
    Ok(Json(user))
}

#[instrument]
#[debug_handler]
pub async fn get_user_urls(
    State(state): State<ServerState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Vec<short_link::Model>>, ServerError> {
    let user_urls = short_link::Entity::find()
        .filter(short_link::Column::UserId.eq(user_id))
        .all(&state.conn)
        .await?;

    Ok(Json(user_urls))
}

#[instrument]
#[debug_handler]
pub async fn oidc_callback(
    State(state): State<ServerState>,
    jar: PrivateCookieJar,
    Query(query): Query<AuthRequest>,
) -> Result<(PrivateCookieJar, Redirect), ServerError> {
    let Some(csrf_token) = jar.get("csrf_token") else {
        return Err(ServerError::OptionError);
    };

    let csrf_state = CsrfToken::new(csrf_token.value().to_owned());
    let query_state = CsrfToken::new(query.state.to_owned());

    if query_state.into_secret() != csrf_state.into_secret() {
        return Err(ServerError::InvalidCsrfToken);
    }

    let Some(pkce_verifier) = jar.get("verifier") else {
        return Err(ServerError::OptionError);
    };

    let token_response = state
        .oidc_client
        .exchange_code(AuthorizationCode::new(query.code.clone()))?
        // Set the PKCE code verifier.
        .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier.value().to_owned()))
        .request_async(&state.client)
        .await?;

    let Some(id_token) = token_response.id_token() else {
        return Err(ServerError::OptionError);
    };
    let id_token_verifier = state.oidc_client.id_token_verifier();
    let Some(nonce) = jar.get("nonce") else {
        return Err(ServerError::OptionError);
    };
    let claims = id_token.claims(&id_token_verifier, &Nonce::new(nonce.value().to_owned()))?;

    if let Some(expected_access_token_hash) = claims.access_token_hash() {
        let actual_access_token_hash = AccessTokenHash::from_token(
            token_response.access_token(),
            id_token.signing_alg()?,
            id_token.signing_key(&id_token_verifier)?,
        )?;
        if actual_access_token_hash != *expected_access_token_hash {
            return Err(ServerError::InvalidAccessTokenHash);
        }
    }

    let profile: UserInfoClaims<EmptyAdditionalClaims, CoreGenderClaim> = state
        .oidc_client
        .user_info(token_response.access_token().clone(), None)?
        .request_async(&state.client)
        .await?;

    let Some(expiry) = token_response.expires_in() else {
        return Err(ServerError::OptionError);
    };

    let secs: i64 = expiry.as_secs().try_into()?;

    let max_age = Local::now().naive_local() + Duration::try_seconds(secs).unwrap();

    let Some(domain) = state.url.domain() else {
        return Err(ServerError::OptionError);
    };

    let token = token_response.access_token().secret().to_owned();

    let cookie = Cookie::build(("sid", token.clone()))
        .domain(format!(".{}", domain))
        .path("/")
        .secure(true)
        .http_only(true)
        .max_age(TimeDuration::seconds(secs));

    let Some(name_claim) = profile.name() else {
        return Err(ServerError::OptionError);
    };

    let Some(name) = name_claim.get(None) else {
        return Err(ServerError::OptionError);
    };

    let Some(email) = profile.email() else {
        return Err(ServerError::OptionError);
    };

    let new_user = user::ActiveModel {
        user_id: ActiveValue::NotSet,
        name: ActiveValue::set(name.as_str().to_owned()),
        email: ActiveValue::set(email.as_str().to_owned()),
        created_at: ActiveValue::set(chrono::Utc::now().naive_utc()),
        updated_at: ActiveValue::set(chrono::Utc::now().naive_utc()),
    };

    let user = new_user.insert(&state.conn).await?;

    let new_session = sessions::ActiveModel {
        id: ActiveValue::NotSet,
        session_id: ActiveValue::set(token),
        user_id: ActiveValue::set(user.user_id),
        expiry: ActiveValue::set(max_age),
    };

    new_session.insert(&state.conn).await?;

    Ok((
        jar.add(cookie)
            .remove("verifier")
            .remove("nonce")
            .remove("csrf_token"),
        Redirect::to("/protected"),
    ))
}

#[instrument]
#[debug_handler]
pub async fn login_oidc(
    jar: PrivateCookieJar,
    State(state): State<ServerState>,
) -> Result<(PrivateCookieJar, Redirect), ServerError> {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the full authorization URL.
    let (auth_url, csrf_token, nonce) = state
        .oidc_client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scopes(state.oidc_config.scopes.clone())
        .set_pkce_challenge(pkce_challenge)
        .url();

    let verifier_cookie = Cookie::build(("verifier", pkce_verifier.secret().to_owned()))
        .secure(true)
        .http_only(true)
        .max_age(TimeDuration::seconds(300))
        .path("/")
        .domain(format!(".{}", state.url.domain().unwrap()));

    let nonce_cookie = Cookie::build(("nonce", nonce.secret().to_owned()))
        .secure(true)
        .http_only(true)
        .max_age(TimeDuration::seconds(300))
        .path("/")
        .domain(format!(".{}", state.url.domain().unwrap()));

    let csrf_token_cookie = Cookie::build(("csrf_token", csrf_token.secret().to_owned()))
        .secure(true)
        .http_only(true)
        .max_age(TimeDuration::seconds(300))
        .path("/")
        .domain(format!(".{}", state.url.domain().unwrap()));

    Ok((
        jar.add(verifier_cookie)
            .add(nonce_cookie)
            .add(csrf_token_cookie),
        Redirect::to(auth_url.as_str()),
    ))
}
