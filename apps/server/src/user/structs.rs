use std::num::TryFromIntError;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::cookie::PrivateCookieJar;
use entity::{short_link, user};
use openidconnect::{
    ClaimsVerificationError, ConfigurationError, HttpClientError, RequestTokenError,
    SignatureVerificationError, SigningError, StandardErrorResponse, UserInfoError,
    core::CoreErrorResponseType,
};
use sea_orm::{DerivePartialModel, FromQueryResult};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::{IntoParams, IntoResponses, ToSchema};

use crate::structs::{BasicError, BasicResponse};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
pub struct OidcName {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, IntoResponses, TS)]
#[serde(untagged)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
pub enum OidcNameResponse {
    #[response(status = StatusCode::OK)]
    OidcName(#[to_schema] OidcName),
}

impl IntoResponse for OidcNameResponse {
    fn into_response(self) -> Response {
        match self {
            OidcNameResponse::OidcName(name) => (StatusCode::OK, Json(name)).into_response(),
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
    fn into_response(self) -> Response {
        match self {
            NewUserResponse::UserCreated(model) => (StatusCode::OK, Json(model)).into_response(),
            NewUserResponse::UserAlreadyExists(e) => {
                (StatusCode::BAD_REQUEST, Json(e)).into_response()
            }
            NewUserResponse::DatabaseError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
            NewUserResponse::PasswordHashError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
        }
    }
}

impl From<argon2::password_hash::Error> for NewUserResponse {
    fn from(e: argon2::password_hash::Error) -> Self {
        Self::PasswordHashError(format!("Password hash error: {}", e).into())
    }
}

impl From<sea_orm::DbErr> for NewUserResponse {
    fn from(e: sea_orm::DbErr) -> Self {
        Self::DatabaseError(format!("Database error: {}", e).into())
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
    fn into_response(self) -> Response {
        match self {
            LoginResponse::UserLoggedIn(model, jar) => {
                (StatusCode::OK, jar, Json(model)).into_response()
            }
            LoginResponse::InvalidCredentials(e) => {
                (StatusCode::BAD_REQUEST, Json(e)).into_response()
            }
            LoginResponse::DatabaseError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
            LoginResponse::InternalServerError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
        }
    }
}

impl From<sea_orm::DbErr> for LoginResponse {
    fn from(e: sea_orm::DbErr) -> Self {
        Self::DatabaseError(format!("Database error: {}", e).into())
    }
}

impl From<argon2::password_hash::Error> for LoginResponse {
    fn from(e: argon2::password_hash::Error) -> Self {
        Self::InvalidCredentials(format!("Password hash error: {}", e).into())
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
    fn into_response(self) -> Response {
        match self {
            LogoutResponse::UserLoggedOut(session_id, jar) => {
                (StatusCode::OK, jar, Json(session_id)).into_response()
            }
            LogoutResponse::InvalidSession(e) => (StatusCode::BAD_REQUEST, Json(e)).into_response(),
            LogoutResponse::DatabaseError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
            LogoutResponse::UserNotLoggedIn(e) => (StatusCode::OK, Json(e)).into_response(),
            LogoutResponse::SessionNotFound(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
        }
    }
}

impl From<sea_orm::DbErr> for LogoutResponse {
    fn from(e: sea_orm::DbErr) -> Self {
        Self::DatabaseError(format!("Database error: {}", e).into())
    }
}

#[derive(
    Debug, Clone, Serialize, Deserialize, ToSchema, DerivePartialModel, FromQueryResult, TS,
)]
#[sea_orm(entity = "user::Entity")]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
pub struct UserProfile {
    pub name: String,
    pub email: String,
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
    UserProfile(#[to_schema] UserProfile),
}

impl From<sea_orm::DbErr> for UserProfileResponse {
    fn from(e: sea_orm::DbErr) -> Self {
        Self::DatabaseError(format!("Database error: {}", e).into())
    }
}

impl IntoResponse for UserProfileResponse {
    fn into_response(self) -> Response {
        match self {
            UserProfileResponse::UserProfile(profile) => {
                (StatusCode::OK, Json(profile)).into_response()
            }
            UserProfileResponse::InvalidSession(e) => {
                (StatusCode::UNAUTHORIZED, Json(e)).into_response()
            }
            UserProfileResponse::DatabaseError(e) => {
                (StatusCode::UNAUTHORIZED, Json(e)).into_response()
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
pub struct UserLinks {
    pub urls: Vec<short_link::Model>,
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
    UserLinks(#[to_schema] UserLinks),
}

impl From<sea_orm::DbErr> for UserLinksResponse {
    fn from(e: sea_orm::DbErr) -> Self {
        Self::DatabaseError(format!("Database error: {}", e).into())
    }
}

impl IntoResponse for UserLinksResponse {
    fn into_response(self) -> Response {
        match self {
            UserLinksResponse::UserLinks(links) => (StatusCode::OK, Json(links)).into_response(),
            UserLinksResponse::InvalidSession(e) => {
                (StatusCode::UNAUTHORIZED, Json(e)).into_response()
            }
            UserLinksResponse::DatabaseError(e) => {
                (StatusCode::UNAUTHORIZED, Json(e)).into_response()
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
    OidcCallback(BasicResponse),
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
    OidcCallback(#[to_schema] BasicResponse, PrivateCookieJar),
    #[response(status = StatusCode::UNAUTHORIZED)]
    CookieNotFound(#[to_schema] BasicError),
    #[response(status = StatusCode::BAD_REQUEST)]
    TokenError(#[to_schema] BasicError),
    #[response(status = StatusCode::INTERNAL_SERVER_ERROR)]
    InternalError(#[to_schema] BasicError),
}

impl IntoResponse for OidcCallbackResponse {
    fn into_response(self) -> Response {
        match self {
            OidcCallbackResponse::OidcCallback(response, jar) => {
                (StatusCode::OK, jar, Json(response)).into_response()
            }
            OidcCallbackResponse::InvalidCsrfToken(e) => {
                (StatusCode::BAD_REQUEST, Json(e)).into_response()
            }
            OidcCallbackResponse::OptionError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
            OidcCallbackResponse::DatabaseError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
            OidcCallbackResponse::InvalidClaims(e) => {
                (StatusCode::BAD_REQUEST, Json(e)).into_response()
            }
            OidcCallbackResponse::InvalidOidcConfig(e) => {
                (StatusCode::BAD_REQUEST, Json(e)).into_response()
            }
            OidcCallbackResponse::InvalidSignature(e) => {
                (StatusCode::BAD_REQUEST, Json(e)).into_response()
            }
            OidcCallbackResponse::UserInfoError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
            OidcCallbackResponse::SignatureError(e) => {
                (StatusCode::BAD_REQUEST, Json(e)).into_response()
            }
            OidcCallbackResponse::IntegerParseError(e) => {
                (StatusCode::BAD_REQUEST, Json(e)).into_response()
            }
            OidcCallbackResponse::CookieNotFound(e) => {
                (StatusCode::UNAUTHORIZED, Json(e)).into_response()
            }
            OidcCallbackResponse::TokenError(e) => {
                (StatusCode::BAD_REQUEST, Json(e)).into_response()
            }
            OidcCallbackResponse::InternalError(e) => {
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
        Self::InvalidCsrfToken(format!("Request token error: {}", e).into())
    }
}

impl From<ClaimsVerificationError> for OidcCallbackResponse {
    fn from(e: ClaimsVerificationError) -> Self {
        Self::InvalidClaims(format!("Claims verification error: {}", e).into())
    }
}

impl From<ConfigurationError> for OidcCallbackResponse {
    fn from(e: ConfigurationError) -> Self {
        Self::InvalidOidcConfig(format!("Configuration error: {}", e).into())
    }
}

impl From<SignatureVerificationError> for OidcCallbackResponse {
    fn from(e: SignatureVerificationError) -> Self {
        Self::InvalidSignature(format!("Signature verification error: {}", e).into())
    }
}

impl From<TryFromIntError> for OidcCallbackResponse {
    fn from(e: TryFromIntError) -> Self {
        Self::IntegerParseError(format!("Integer parse error: {}", e).into())
    }
}

impl From<UserInfoError<HttpClientError<reqwest::Error>>> for OidcCallbackResponse {
    fn from(e: UserInfoError<HttpClientError<reqwest::Error>>) -> Self {
        Self::UserInfoError(format!("User info error: {}", e).into())
    }
}

impl From<SigningError> for OidcCallbackResponse {
    fn from(e: SigningError) -> Self {
        Self::SignatureError(format!("Signing error: {}", e).into())
    }
}

impl From<sea_orm::DbErr> for OidcCallbackResponse {
    fn from(e: sea_orm::DbErr) -> Self {
        Self::DatabaseError(format!("Database error: {}", e).into())
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
    fn into_response(self) -> Response {
        match self {
            OidcLoginResponse::OidcLogin(url, jar) => {
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
