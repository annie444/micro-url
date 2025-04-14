use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tracing::{error, warn};

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("SQL error: {0}")]
    DbError(#[from] sea_orm::error::DbErr),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Unknown OIDC client: {0}")]
    UnknownClient(String),
    #[error("HTTP request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Attempted to get a non-none value but found none")]
    OptionError,
    #[error("Encountered an error trying to convert an infallible value: {0}")]
    FromRequestPartsError(#[from] std::convert::Infallible),
    #[error("Unable to parse URL: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("Error parsing number: {0}")]
    IntParseError(#[from] std::num::TryFromIntError),
    #[error("User not authorized")]
    Unauthorized,
    #[error("Invalid OIDC client configuration: {0}")]
    OidcConfigurationError(#[from] openidconnect::ConfigurationError),
    #[error("Invalid OIDC token request: {0}")]
    OidcTokenRequestError(
        #[from]
        openidconnect::RequestTokenError<
            openidconnect::HttpClientError<reqwest::Error>,
            openidconnect::StandardErrorResponse<openidconnect::core::CoreErrorResponseType>,
        >,
    ),
    #[error("OIDC claims verification error: {0}")]
    OidcClaimError(#[from] openidconnect::ClaimsVerificationError),
    #[error("Invalid OIDC token signature: {0}")]
    OidcSignatureError(#[from] openidconnect::SigningError),
    #[error("Invalid OIDC token signature: {0}")]
    OidcSignatureVerificationError(#[from] openidconnect::SignatureVerificationError),
    #[error("Invalid OIDC token hash")]
    InvalidAccessTokenHash,
    #[error("Error getting user info: {0}")]
    OidcUserInfoError(
        #[from] openidconnect::UserInfoError<openidconnect::HttpClientError<reqwest::Error>>,
    ),
    #[error("Error validating OIDC server response")]
    InvalidCsrfToken,
    #[error("Error running password cyphers: {0}")]
    PasswordError(#[from] argon2::Error),
    #[error("Error hashing password: {0}")]
    PasswordHashError(#[from] argon2::password_hash::Error),
    #[error("Error decoding header value: {0}")]
    HeaderError(#[from] axum::http::header::ToStrError),
    #[error("Error coercing string into integer: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let response = match self {
            Self::DbError(e) => {
                error!("Database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
            Self::UnknownClient(e) => {
                warn!("Unknown OIDC client: {}", e);
                (StatusCode::NOT_FOUND, format!("Unknown OIDC client: {}", e))
            }
            Self::Request(e) => {
                error!("Request error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
            Self::OptionError => {
                error!("Attempted to get a nonexistent value");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Attempted to get a nonexistent value".to_string(),
                )
            }
            Self::FromRequestPartsError(e) => {
                error!("Infallible error from request parts: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
            Self::UrlParseError(e) => {
                error!("URL parse error: {}", e);
                (StatusCode::BAD_REQUEST, e.to_string())
            }
            Self::IntParseError(e) => {
                error!("Integer parse error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
            Self::Unauthorized => {
                warn!("User not authorized");
                (StatusCode::UNAUTHORIZED, "User not authorized".to_string())
            }
            Self::OidcConfigurationError(e) => {
                error!("OIDC configuration error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
            Self::OidcTokenRequestError(e) => {
                error!("OIDC token request error: {}", e);
                (StatusCode::BAD_REQUEST, e.to_string())
            }
            Self::OidcClaimError(e) => {
                error!("OIDC claim error: {}", e);
                (StatusCode::BAD_REQUEST, e.to_string())
            }
            Self::OidcSignatureError(e) => {
                error!("OIDC signature error: {}", e);
                (StatusCode::BAD_REQUEST, e.to_string())
            }
            Self::InvalidAccessTokenHash => {
                error!("Invalid access token hash");
                (
                    StatusCode::BAD_REQUEST,
                    "Invalid access token hash".to_string(),
                )
            }
            Self::OidcSignatureVerificationError(e) => {
                error!("OIDC signature verification error: {}", e);
                (StatusCode::BAD_REQUEST, e.to_string())
            }
            Self::OidcUserInfoError(e) => {
                error!("OIDC user info error: {}", e);
                (StatusCode::BAD_REQUEST, e.to_string())
            }
            Self::InvalidCsrfToken => {
                error!("Invalid CSRF token");
                (StatusCode::BAD_REQUEST, "Invalid CSRF token".to_string())
            }
            Self::DatabaseError(e) => {
                error!("Database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e)
            }
            Self::PasswordError(e) => {
                error!("Password error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
            Self::PasswordHashError(e) => {
                error!("Password hash error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
            Self::HeaderError(e) => {
                error!("Ascii decoding error: {}", e.to_string());
                (StatusCode::BAD_REQUEST, e.to_string())
            }
            Self::ParseIntError(e) => {
                error!("Unable to coerce string into integer: {}", e.to_string());
                (StatusCode::BAD_REQUEST, e.to_string())
            }
        };
        response.into_response()
    }
}
