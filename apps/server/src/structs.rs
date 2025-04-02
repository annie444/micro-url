use axum::{
    body::Body,
    extract::{FromRequest, FromRequestParts},
    http::{request::Parts, Request},
};
use axum_extra::extract::cookie::PrivateCookieJar;
use chrono::{DateTime, FixedOffset};
use entity::{sessions, short_link, user};
use openidconnect::{
    core::{
        CoreAuthDisplay, CoreAuthPrompt, CoreErrorResponseType, CoreGenderClaim, CoreJsonWebKey,
        CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm, CoreRevocableToken,
        CoreTokenType,
    },
    Client, EmptyAdditionalClaims, EmptyExtraTokenFields, EndpointMaybeSet, EndpointNotSet,
    EndpointSet, IdTokenFields, RevocationErrorResponseType, StandardErrorResponse,
    StandardTokenIntrospectionResponse, StandardTokenResponse,
};
use sea_orm::{entity::*, query::*, DerivePartialModel, FromQueryResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::ServerError, state::ServerState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUrlRequest {
    pub url: String,
    pub short: Option<String>,
    pub user: Option<Uuid>,
    pub expiry: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUserRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUrl {
    pub url: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUrls(pub Vec<AuthUrl>); // New struct to hold a vector of AuthUrl

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequest {
    pub code: String,
    pub state: String,
}

pub type OidcClient = Client<
    EmptyAdditionalClaims,
    CoreAuthDisplay,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJsonWebKey,
    CoreAuthPrompt,
    StandardErrorResponse<CoreErrorResponseType>,
    StandardTokenResponse<
        IdTokenFields<
            EmptyAdditionalClaims,
            EmptyExtraTokenFields,
            CoreGenderClaim,
            CoreJweContentEncryptionAlgorithm,
            CoreJwsSigningAlgorithm,
        >,
        CoreTokenType,
    >,
    StandardTokenIntrospectionResponse<EmptyExtraTokenFields, CoreTokenType>,
    CoreRevocableToken,
    StandardErrorResponse<RevocationErrorResponseType>,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfoResponse {
    pub sub: String,
    pub email: String,
    pub name: String,
    pub preferred_username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, DerivePartialModel, FromQueryResult)]
#[sea_orm(entity = "user::Entity")]
pub struct UserProfile {
    pub email: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLinks {
    pub urls: Vec<short_link::Model>,
}

impl FromRequest<ServerState> for UserProfile {
    type Rejection = ServerError;
    async fn from_request(
        req: Request<Body>,
        state: &ServerState,
    ) -> Result<Self, Self::Rejection> {
        let state = state.to_owned();
        let (mut parts, _body) = req.into_parts();
        let cookiejar: PrivateCookieJar =
            PrivateCookieJar::from_request_parts(&mut parts, &state).await?;

        let Some(cookie) = cookiejar.get("sid").map(|cookie| cookie.value().to_owned()) else {
            return Err(ServerError::Unauthorized);
        };

        let Some(res) = sessions::Entity::find()
            .left_join(user::Entity)
            .columns([user::Column::Email, user::Column::Name])
            .filter(sessions::Column::SessionId.eq(cookie))
            .into_partial_model::<UserProfile>()
            .one(&state.conn)
            .await?
        else {
            return Err(ServerError::Unauthorized);
        };

        Ok(res)
    }
}

impl FromRequestParts<ServerState> for UserProfile {
    type Rejection = ServerError;
    async fn from_request_parts(
        req: &mut Parts,
        state: &ServerState,
    ) -> Result<Self, Self::Rejection> {
        let state = state.to_owned();
        let cookiejar: PrivateCookieJar = PrivateCookieJar::from_request_parts(req, &state).await?;

        let Some(cookie) = cookiejar.get("sid").map(|cookie| cookie.value().to_owned()) else {
            return Err(ServerError::Unauthorized);
        };

        let Some(res) = sessions::Entity::find()
            .left_join(user::Entity)
            .columns([user::Column::Email, user::Column::Name])
            .filter(sessions::Column::SessionId.eq(cookie))
            .into_partial_model::<UserProfile>()
            .one(&state.conn)
            .await?
        else {
            return Err(ServerError::Unauthorized);
        };

        Ok(res)
    }
}

impl FromRequest<ServerState> for UserLinks {
    type Rejection = ServerError;
    async fn from_request(
        req: Request<Body>,
        state: &ServerState,
    ) -> Result<Self, Self::Rejection> {
        let state = state.to_owned();
        let (mut parts, _body) = req.into_parts();
        let cookiejar: PrivateCookieJar =
            PrivateCookieJar::from_request_parts(&mut parts, &state).await?;

        let Some(cookie) = cookiejar.get("sid").map(|cookie| cookie.value().to_owned()) else {
            return Err(ServerError::Unauthorized);
        };

        let Some(res) = user::Entity::find()
            .column(user::Column::UserId)
            .filter(sessions::Column::SessionId.eq(cookie))
            .one(&state.conn)
            .await?
        else {
            return Err(ServerError::Unauthorized);
        };

        let res = short_link::Entity::find()
            .filter(short_link::Column::UserId.eq(res.user_id))
            .all(&state.conn)
            .await?;

        Ok(Self { urls: res })
    }
}

impl FromRequestParts<ServerState> for UserLinks {
    type Rejection = ServerError;
    async fn from_request_parts(
        req: &mut Parts,
        state: &ServerState,
    ) -> Result<Self, Self::Rejection> {
        let state = state.to_owned();
        let cookiejar: PrivateCookieJar = PrivateCookieJar::from_request_parts(req, &state).await?;

        let Some(cookie) = cookiejar.get("sid").map(|cookie| cookie.value().to_owned()) else {
            return Err(ServerError::Unauthorized);
        };

        let Some(res) = user::Entity::find()
            .column(user::Column::UserId)
            .filter(sessions::Column::SessionId.eq(cookie))
            .one(&state.conn)
            .await?
        else {
            return Err(ServerError::Unauthorized);
        };

        let res = short_link::Entity::find()
            .filter(short_link::Column::UserId.eq(res.user_id))
            .all(&state.conn)
            .await?;

        Ok(Self { urls: res })
    }
}
