use std::{collections::BTreeMap, fmt::Display, num::TryFromIntError};

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::cookie::PrivateCookieJar;
use chrono::NaiveDateTime;
use entity::{short_link, user, views};
use openidconnect::{
    ClaimsVerificationError, ConfigurationError, HttpClientError, RequestTokenError,
    SignatureVerificationError, SigningError, StandardErrorResponse, UserInfoError,
    core::CoreErrorResponseType,
};
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use tracing::{error, info, instrument, warn};
use ts_rs::TS;
use utoipa::{IntoParams, IntoResponses, ToSchema};
use uuid::Uuid;

#[cfg(feature = "headers")]
use crate::utils::HeaderMapDef;
use crate::utils::{BasicError, BasicResponse};

#[derive(Debug, Clone, Serialize, Deserialize, IntoParams, TS)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
#[into_params(parameter_in = Query, style = Form)]
pub struct Paginate {
    pub page: u64,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
pub struct OidcName {
    pub name: String,
}

impl Display for OidcName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, IntoResponses, TS)]
#[serde(untagged)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
pub enum OidcNameResponse {
    #[response(status = StatusCode::OK)]
    OidcName(#[to_schema] OidcName),
}

impl IntoResponse for OidcNameResponse {
    #[instrument]
    fn into_response(self) -> Response {
        match self {
            OidcNameResponse::OidcName(name) => {
                info!(%name);
                (StatusCode::OK, Json(name)).into_response()
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
pub struct NewUserRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, IntoResponses, TS)]
#[allow(clippy::large_enum_variant)]
#[serde(untagged)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
pub enum NewUserResponse {
    #[response(status = StatusCode::BAD_REQUEST)]
    UserAlreadyExists(#[to_schema] BasicError),
    #[response(status = StatusCode::INTERNAL_SERVER_ERROR)]
    DatabaseError(#[to_schema] BasicError),
    #[response(status = StatusCode::INTERNAL_SERVER_ERROR)]
    PasswordHashError(#[to_schema] BasicError),
    #[response(status = StatusCode::OK)]
    UserCreated(#[to_schema] user::Model),
}

impl IntoResponse for NewUserResponse {
    #[instrument]
    fn into_response(self) -> Response {
        match self {
            NewUserResponse::UserCreated(model) => {
                info!("{:?}", model);
                (StatusCode::OK, Json(model)).into_response()
            }
            NewUserResponse::UserAlreadyExists(e) => {
                error!(%e);
                (StatusCode::BAD_REQUEST, Json(e)).into_response()
            }
            NewUserResponse::DatabaseError(e) => {
                error!(%e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
            NewUserResponse::PasswordHashError(e) => {
                error!(%e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
        }
    }
}

impl From<argon2::password_hash::Error> for NewUserResponse {
    fn from(e: argon2::password_hash::Error) -> Self {
        Self::PasswordHashError(format!("Password hash error: {e}").into())
    }
}

impl From<sea_orm::DbErr> for NewUserResponse {
    fn from(e: sea_orm::DbErr) -> Self {
        Self::DatabaseError(format!("Database error: {e}").into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[allow(clippy::large_enum_variant)]
#[serde(untagged)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
pub enum LoginResponseType {
    InvalidCredentials(BasicError),
    DatabaseError(BasicError),
    UserLoggedIn(user::Model),
    InternalServerError(BasicError),
}

#[derive(Debug, Clone, IntoResponses)]
#[allow(clippy::large_enum_variant)]
pub enum LoginResponse {
    #[response(status = StatusCode::BAD_REQUEST)]
    InvalidCredentials(#[to_schema] BasicError),
    #[response(status = StatusCode::INTERNAL_SERVER_ERROR)]
    DatabaseError(#[to_schema] BasicError),
    #[response(status = StatusCode::OK)]
    UserLoggedIn(#[to_schema] user::Model, PrivateCookieJar),
    #[response(status = StatusCode::INTERNAL_SERVER_ERROR)]
    InternalServerError(#[to_schema] BasicError),
}

impl IntoResponse for LoginResponse {
    #[instrument]
    fn into_response(self) -> Response {
        match self {
            LoginResponse::UserLoggedIn(model, jar) => {
                info!("{model:?}");
                (StatusCode::OK, jar, Json(model)).into_response()
            }
            LoginResponse::InvalidCredentials(e) => {
                error!(%e);
                (StatusCode::BAD_REQUEST, Json(e)).into_response()
            }
            LoginResponse::DatabaseError(e) => {
                error!(%e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
            LoginResponse::InternalServerError(e) => {
                error!(%e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
        }
    }
}

impl From<sea_orm::DbErr> for LoginResponse {
    fn from(e: sea_orm::DbErr) -> Self {
        Self::DatabaseError(format!("Database error: {e}").into())
    }
}

impl From<argon2::password_hash::Error> for LoginResponse {
    fn from(e: argon2::password_hash::Error) -> Self {
        Self::InvalidCredentials(format!("Password hash error: {e}").into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(untagged)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
pub enum LogoutResponseType {
    InvalidSession(BasicError),
    DatabaseError(BasicError),
    UserLoggedOut(BasicResponse),
    UserNotLoggedIn(BasicResponse),
    SessionNotFound(BasicError),
}

#[derive(Debug, Clone, IntoResponses)]
#[allow(clippy::large_enum_variant)]
pub enum LogoutResponse {
    #[response(status = StatusCode::BAD_REQUEST)]
    InvalidSession(#[to_schema] BasicError),
    #[response(status = StatusCode::INTERNAL_SERVER_ERROR)]
    DatabaseError(#[to_schema] BasicError),
    #[response(status = StatusCode::OK)]
    UserLoggedOut(#[to_schema] BasicResponse, PrivateCookieJar),
    #[response(status = StatusCode::OK)]
    UserNotLoggedIn(#[to_schema] BasicResponse),
    #[response(status = StatusCode::INTERNAL_SERVER_ERROR)]
    SessionNotFound(#[to_schema] BasicError),
}

impl IntoResponse for LogoutResponse {
    #[instrument]
    fn into_response(self) -> Response {
        match self {
            LogoutResponse::UserLoggedOut(session_id, jar) => {
                info!("{session_id:?}");
                (StatusCode::OK, jar, Json(session_id)).into_response()
            }
            LogoutResponse::InvalidSession(e) => {
                error!(%e);
                (StatusCode::BAD_REQUEST, Json(e)).into_response()
            }
            LogoutResponse::DatabaseError(e) => {
                error!(%e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
            LogoutResponse::UserNotLoggedIn(e) => {
                warn!("{e:?}");
                (StatusCode::OK, Json(e)).into_response()
            }
            LogoutResponse::SessionNotFound(e) => {
                error!(%e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
        }
    }
}

impl From<sea_orm::DbErr> for LogoutResponse {
    fn from(e: sea_orm::DbErr) -> Self {
        Self::DatabaseError(format!("Database error: {e}").into())
    }
}

#[derive(Debug, Clone, IntoResponses, Serialize, Deserialize, TS)]
#[serde(untagged)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
pub enum UserProfileResponse {
    #[response(status = StatusCode::UNAUTHORIZED)]
    InvalidSession(#[to_schema] BasicError),
    #[response(status = StatusCode::UNAUTHORIZED)]
    DatabaseError(#[to_schema] BasicError),
    #[response(status = StatusCode::OK)]
    UserProfile(#[to_schema] user::Model),
}

impl From<sea_orm::DbErr> for UserProfileResponse {
    fn from(e: sea_orm::DbErr) -> Self {
        Self::DatabaseError(format!("Database error: {e}").into())
    }
}

impl IntoResponse for UserProfileResponse {
    #[instrument]
    fn into_response(self) -> Response {
        match self {
            UserProfileResponse::UserProfile(profile) => {
                info!("{profile:?}");
                (StatusCode::OK, Json(profile)).into_response()
            }
            UserProfileResponse::InvalidSession(e) => {
                error!(%e);
                (StatusCode::UNAUTHORIZED, Json(e)).into_response()
            }
            UserProfileResponse::DatabaseError(e) => {
                error!(%e);
                (StatusCode::UNAUTHORIZED, Json(e)).into_response()
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
pub struct UserLinksAndViews {
    pub urls: Vec<UserLinkWithViews>,
}

impl From<Vec<(short_link::Model, Vec<views::Model>)>> for UserLinksAndViews {
    fn from(models: Vec<(short_link::Model, Vec<views::Model>)>) -> Self {
        Self {
            urls: models
                .iter()
                .map(|v| v.to_owned().into())
                .collect::<Vec<UserLinkWithViews>>(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
pub struct UserLinkWithViews {
    pub id: String,
    pub short_url: String,
    pub original_url: String,
    pub user_id: Uuid,
    #[ts(optional)]
    pub expiry_date: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub views: Vec<UserView>,
}

impl From<(short_link::Model, Vec<views::Model>)> for UserLinkWithViews {
    fn from(values: (short_link::Model, Vec<views::Model>)) -> Self {
        let (sl, vi) = values;
        Self {
            id: sl.id,
            short_url: sl.short_url,
            original_url: sl.original_url,
            user_id: sl.user_id.unwrap(),
            expiry_date: sl.expiry_date,
            created_at: sl.created_at,
            updated_at: sl.updated_at,
            views: vi
                .iter()
                .map(|v| v.to_owned().into())
                .collect::<Vec<UserView>>(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
pub struct UserView {
    pub id: i32,
    #[ts(optional)]
    pub headers: Option<BTreeMap<String, Vec<String>>>,
    #[ts(optional)]
    pub ip: Option<String>,
    pub cache_hit: bool,
    pub created_at: NaiveDateTime,
}

#[cfg(feature = "headers")]
impl From<views::Model> for UserView {
    fn from(vi: views::Model) -> Self {
        let headers: Option<HeaderMapDef> = vi.headers.map(|val| val.into());
        Self {
            id: vi.id,
            headers: headers.map(|v| v.0),
            ip: vi.ip.map(|ip| ip.ip().to_string()),
            cache_hit: vi.cache_hit,
            created_at: vi.created_at,
        }
    }
}

#[cfg(not(feature = "headers"))]
impl From<views::Model> for UserView {
    fn from(vi: views::Model) -> Self {
        Self {
            id: vi.id,
            headers: None,
            ip: vi.ip.map(|ip| ip.ip().to_string()),
            cache_hit: vi.cache_hit,
            created_at: vi.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult, ToSchema, TS)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
pub struct UserLink {
    pub id: String,
    pub short_url: String,
    pub original_url: String,
    pub user_id: Uuid,
    #[ts(optional)]
    pub expiry_date: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub views: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, IntoResponses, TS)]
#[serde(untagged)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
pub enum UserLinksResponse {
    #[response(status = StatusCode::UNAUTHORIZED)]
    InvalidSession(#[to_schema] BasicError),
    #[response(status = StatusCode::UNAUTHORIZED)]
    DatabaseError(#[to_schema] BasicError),
    #[response(status = StatusCode::OK)]
    UserLinksAndViews(#[to_schema] UserLinksAndViews),
    #[response(status = StatusCode::OK)]
    UserLinks(#[to_schema] Vec<UserLink>),
}

impl From<sea_orm::DbErr> for UserLinksResponse {
    fn from(e: sea_orm::DbErr) -> Self {
        Self::DatabaseError(format!("Database error: {e}").into())
    }
}

impl From<sea_orm::TransactionError<sea_orm::DbErr>> for UserLinksResponse {
    fn from(value: sea_orm::TransactionError<sea_orm::DbErr>) -> Self {
        Self::DatabaseError(format!("Error commit database transaction: {value}").into())
    }
}

impl IntoResponse for UserLinksResponse {
    #[instrument]
    fn into_response(self) -> Response {
        match self {
            UserLinksResponse::UserLinksAndViews(links) => {
                info!("{links:?}");
                (StatusCode::OK, Json(links)).into_response()
            }
            UserLinksResponse::InvalidSession(e) => {
                error!(%e);
                (StatusCode::UNAUTHORIZED, Json(e)).into_response()
            }
            UserLinksResponse::DatabaseError(e) => {
                error!(%e);
                (StatusCode::UNAUTHORIZED, Json(e)).into_response()
            }
            Self::UserLinks(links) => {
                info!("{:?}", links);
                (StatusCode::OK, Json(links)).into_response()
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(untagged)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
pub enum OidcCallbackResponseType {
    InvalidCsrfToken(BasicError),
    InvalidClaims(BasicError),
    InvalidOidcConfig(BasicError),
    InvalidSignature(BasicError),
    UserInfoError(BasicError),
    SignatureError(BasicError),
    IntegerParseError(BasicError),
    OptionError(BasicError),
    DatabaseError(BasicError),
    OidcCallback(String),
    CookieNotFound(BasicError),
    TokenError(BasicError),
    InternalError(BasicError),
}

#[derive(Debug, Clone, IntoResponses)]
pub enum OidcCallbackResponse {
    #[response(status = StatusCode::BAD_REQUEST)]
    InvalidCsrfToken(#[to_schema] BasicError),
    #[response(status = StatusCode::BAD_REQUEST)]
    InvalidClaims(#[to_schema] BasicError),
    #[response(status = StatusCode::BAD_REQUEST)]
    InvalidOidcConfig(#[to_schema] BasicError),
    #[response(status = StatusCode::BAD_REQUEST)]
    InvalidSignature(#[to_schema] BasicError),
    #[response(status = StatusCode::INTERNAL_SERVER_ERROR)]
    UserInfoError(#[to_schema] BasicError),
    #[response(status = StatusCode::BAD_REQUEST)]
    SignatureError(#[to_schema] BasicError),
    #[response(status = StatusCode::BAD_REQUEST)]
    IntegerParseError(#[to_schema] BasicError),
    #[response(status = StatusCode::INTERNAL_SERVER_ERROR)]
    OptionError(#[to_schema] BasicError),
    #[response(status = StatusCode::INTERNAL_SERVER_ERROR)]
    DatabaseError(#[to_schema] BasicError),
    #[response(status = StatusCode::OK)]
    OidcCallback(#[to_schema] String, PrivateCookieJar),
    #[response(status = StatusCode::UNAUTHORIZED)]
    CookieNotFound(#[to_schema] BasicError),
    #[response(status = StatusCode::BAD_REQUEST)]
    TokenError(#[to_schema] BasicError),
    #[response(status = StatusCode::INTERNAL_SERVER_ERROR)]
    InternalError(#[to_schema] BasicError),
}

impl IntoResponse for OidcCallbackResponse {
    #[instrument]
    fn into_response(self) -> Response {
        match self {
            OidcCallbackResponse::OidcCallback(response, jar) => {
                info!(response);
                (jar, Redirect::temporary(&response)).into_response()
            }
            OidcCallbackResponse::InvalidCsrfToken(e) => {
                error!(%e);
                (StatusCode::BAD_REQUEST, Json(e)).into_response()
            }
            OidcCallbackResponse::OptionError(e) => {
                error!(%e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
            OidcCallbackResponse::DatabaseError(e) => {
                error!(%e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
            OidcCallbackResponse::InvalidClaims(e) => {
                error!(%e);
                (StatusCode::BAD_REQUEST, Json(e)).into_response()
            }
            OidcCallbackResponse::InvalidOidcConfig(e) => {
                error!(%e);
                (StatusCode::BAD_REQUEST, Json(e)).into_response()
            }
            OidcCallbackResponse::InvalidSignature(e) => {
                error!(%e);
                (StatusCode::BAD_REQUEST, Json(e)).into_response()
            }
            OidcCallbackResponse::UserInfoError(e) => {
                error!(%e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
            OidcCallbackResponse::SignatureError(e) => {
                error!(%e);
                (StatusCode::BAD_REQUEST, Json(e)).into_response()
            }
            OidcCallbackResponse::IntegerParseError(e) => {
                error!(%e);
                (StatusCode::BAD_REQUEST, Json(e)).into_response()
            }
            OidcCallbackResponse::CookieNotFound(e) => {
                error!(%e);
                (StatusCode::UNAUTHORIZED, Json(e)).into_response()
            }
            OidcCallbackResponse::TokenError(e) => {
                error!(%e);
                (StatusCode::BAD_REQUEST, Json(e)).into_response()
            }
            OidcCallbackResponse::InternalError(e) => {
                error!(%e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
        }
    }
}

impl
    From<
        RequestTokenError<
            HttpClientError<reqwest::Error>,
            StandardErrorResponse<CoreErrorResponseType>,
        >,
    > for OidcCallbackResponse
{
    fn from(
        e: RequestTokenError<
            HttpClientError<reqwest::Error>,
            StandardErrorResponse<CoreErrorResponseType>,
        >,
    ) -> Self {
        Self::InvalidCsrfToken(format!("Request token error: {e}").into())
    }
}

impl From<ClaimsVerificationError> for OidcCallbackResponse {
    fn from(e: ClaimsVerificationError) -> Self {
        Self::InvalidClaims(format!("Claims verification error: {e}").into())
    }
}

impl From<ConfigurationError> for OidcCallbackResponse {
    fn from(e: ConfigurationError) -> Self {
        Self::InvalidOidcConfig(format!("Configuration error: {e}").into())
    }
}

impl From<SignatureVerificationError> for OidcCallbackResponse {
    fn from(e: SignatureVerificationError) -> Self {
        Self::InvalidSignature(format!("Signature verification error: {e}").into())
    }
}

impl From<TryFromIntError> for OidcCallbackResponse {
    fn from(e: TryFromIntError) -> Self {
        Self::IntegerParseError(format!("Integer parse error: {e}").into())
    }
}

impl From<UserInfoError<HttpClientError<reqwest::Error>>> for OidcCallbackResponse {
    fn from(e: UserInfoError<HttpClientError<reqwest::Error>>) -> Self {
        Self::UserInfoError(format!("User info error: {e}").into())
    }
}

impl From<SigningError> for OidcCallbackResponse {
    fn from(e: SigningError) -> Self {
        Self::SignatureError(format!("Signing error: {e}").into())
    }
}

impl From<sea_orm::DbErr> for OidcCallbackResponse {
    fn from(e: sea_orm::DbErr) -> Self {
        Self::DatabaseError(format!("Database error: {e}").into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OidcLoginResponseType {
    OidcLogin(String),
}

#[derive(Debug, Clone, IntoResponses)]
pub enum OidcLoginResponse {
    #[response(status = StatusCode::TEMPORARY_REDIRECT)]
    OidcLogin(#[to_schema] String, PrivateCookieJar),
}

impl IntoResponse for OidcLoginResponse {
    #[instrument]
    fn into_response(self) -> Response {
        match self {
            OidcLoginResponse::OidcLogin(url, jar) => {
                info!(url);
                (StatusCode::TEMPORARY_REDIRECT, jar, Redirect::to(&url)).into_response()
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, IntoParams)]
pub struct AuthRequest {
    pub code: String,
    pub state: String,
}
