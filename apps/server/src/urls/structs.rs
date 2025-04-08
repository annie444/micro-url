use axum::{
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Json,
};
use chrono::NaiveDateTime;
use entity::short_link;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::{IntoResponses, ToSchema};
use uuid::Uuid;

use crate::structs::BasicError;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[typeshare]
pub struct NewUrlRequest {
    pub url: String,
    pub short: Option<String>,
    pub user: Option<Uuid>,
    pub expiry: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, IntoResponses)]
#[typeshare]
pub enum NewUrlResponse {
    #[response(status = StatusCode::BAD_REQUEST)]
    UrlParseError(#[to_schema] BasicError),
    #[response(status = StatusCode::INTERNAL_SERVER_ERROR)]
    DatabaseError(#[to_schema] BasicError),
    #[response(status = StatusCode::NOT_FOUND)]
    UrlNotFound,
    #[response(status = StatusCode::OK)]
    UrlCreated(#[to_schema] short_link::Model),
}

impl IntoResponse for NewUrlResponse {
    fn into_response(self) -> Response {
        match self {
            NewUrlResponse::UrlCreated(model) => (StatusCode::OK, Json(model)).into_response(),
            NewUrlResponse::UrlNotFound => (
                StatusCode::NOT_FOUND,
                Json(BasicError {
                    error: "URL not found".to_string(),
                }),
            )
                .into_response(),
            NewUrlResponse::DatabaseError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
            NewUrlResponse::UrlParseError(e) => (StatusCode::BAD_REQUEST, Json(e)).into_response(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, IntoResponses)]
#[typeshare]
pub enum GetExistingUrlError {
    #[response(status = StatusCode::BAD_REQUEST)]
    UrlNotFound,
    #[response(status = StatusCode::INTERNAL_SERVER_ERROR)]
    DatabaseError(#[to_schema] BasicError),
}

impl IntoResponse for GetExistingUrlError {
    fn into_response(self) -> Response {
        match self {
            GetExistingUrlError::UrlNotFound => (
                StatusCode::NOT_FOUND,
                Json(BasicError {
                    error: "URL not found".to_string(),
                }),
            )
                .into_response(),
            GetExistingUrlError::DatabaseError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
        }
    }
}

impl From<GetExistingUrlError> for NewUrlResponse {
    fn from(e: GetExistingUrlError) -> Self {
        match e {
            GetExistingUrlError::UrlNotFound => NewUrlResponse::UrlNotFound,
            GetExistingUrlError::DatabaseError(e) => NewUrlResponse::DatabaseError(e),
        }
    }
}

impl From<sea_orm::DbErr> for GetExistingUrlError {
    fn from(e: sea_orm::DbErr) -> Self {
        GetExistingUrlError::DatabaseError(BasicError {
            error: e.to_string(),
        })
    }
}

impl From<sea_orm::DbErr> for NewUrlResponse {
    fn from(e: sea_orm::DbErr) -> Self {
        NewUrlResponse::DatabaseError(BasicError {
            error: e.to_string(),
        })
    }
}

impl From<url::ParseError> for NewUrlResponse {
    fn from(e: url::ParseError) -> Self {
        NewUrlResponse::UrlParseError(BasicError {
            error: e.to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, IntoResponses)]
#[typeshare]
pub enum DeleteUrlResponse {
    #[response(status = StatusCode::INTERNAL_SERVER_ERROR)]
    DatabaseError(#[to_schema] BasicError),
    #[response(status = StatusCode::BAD_REQUEST)]
    UrlNotFound,
    #[response(status = StatusCode::OK)]
    UrlDeleted,
}

impl IntoResponse for DeleteUrlResponse {
    fn into_response(self) -> Response {
        match self {
            DeleteUrlResponse::UrlDeleted => (
                StatusCode::OK,
                Json(BasicError {
                    error: "OK".to_string(),
                }),
            )
                .into_response(),
            DeleteUrlResponse::UrlNotFound => (
                StatusCode::BAD_REQUEST,
                Json(BasicError {
                    error: "URL not found".to_string(),
                }),
            )
                .into_response(),
            DeleteUrlResponse::DatabaseError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
        }
    }
}

impl From<sea_orm::DbErr> for DeleteUrlResponse {
    fn from(e: sea_orm::DbErr) -> Self {
        DeleteUrlResponse::DatabaseError(BasicError {
            error: e.to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, IntoResponses)]
#[typeshare]
pub enum UpdateUrlResponse {
    #[response(status = StatusCode::BAD_REQUEST)]
    UrlParseError(#[to_schema] BasicError),
    #[response(status = StatusCode::INTERNAL_SERVER_ERROR)]
    DatabaseError(#[to_schema] BasicError),
    #[response(status = StatusCode::BAD_REQUEST)]
    UrlNotFound,
    #[response(status = StatusCode::OK)]
    UrlUpdated(#[to_schema] short_link::Model),
}

impl IntoResponse for UpdateUrlResponse {
    fn into_response(self) -> Response {
        match self {
            UpdateUrlResponse::UrlUpdated(model) => (StatusCode::OK, Json(model)).into_response(),
            UpdateUrlResponse::UrlNotFound => (
                StatusCode::NOT_FOUND,
                Json(BasicError {
                    error: "URL not found".to_string(),
                }),
            )
                .into_response(),
            UpdateUrlResponse::DatabaseError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
            UpdateUrlResponse::UrlParseError(e) => {
                (StatusCode::BAD_REQUEST, Json(e)).into_response()
            }
        }
    }
}

impl From<sea_orm::DbErr> for UpdateUrlResponse {
    fn from(e: sea_orm::DbErr) -> Self {
        UpdateUrlResponse::DatabaseError(BasicError {
            error: e.to_string(),
        })
    }
}

impl From<url::ParseError> for UpdateUrlResponse {
    fn from(e: url::ParseError) -> Self {
        UpdateUrlResponse::UrlParseError(BasicError {
            error: e.to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, IntoResponses)]
#[typeshare]
pub enum GetUrlResponse {
    #[response(status = StatusCode::INTERNAL_SERVER_ERROR)]
    DatabaseError(#[to_schema] BasicError),
    #[response(status = StatusCode::NOT_FOUND)]
    UrlNotFound,
    #[response(status = StatusCode::PERMANENT_REDIRECT)]
    Redirect(#[to_schema] String),
}

impl IntoResponse for GetUrlResponse {
    fn into_response(self) -> Response {
        match self {
            GetUrlResponse::UrlNotFound => (
                StatusCode::NOT_FOUND,
                Json(BasicError {
                    error: "URL not found".to_string(),
                }),
            )
                .into_response(),
            GetUrlResponse::DatabaseError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
            GetUrlResponse::Redirect(url) => Redirect::permanent(&url).into_response(),
        }
    }
}

impl From<sea_orm::DbErr> for GetUrlResponse {
    fn from(e: sea_orm::DbErr) -> Self {
        GetUrlResponse::DatabaseError(BasicError {
            error: e.to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, IntoResponses)]
#[typeshare]
pub enum GetUrlInfoResponse {
    #[response(status = StatusCode::INTERNAL_SERVER_ERROR)]
    DatabaseError(#[to_schema] BasicError),
    #[response(status = StatusCode::NOT_FOUND)]
    UrlNotFound,
    #[response(status = StatusCode::OK)]
    Url(#[to_schema] short_link::Model),
}

impl IntoResponse for GetUrlInfoResponse {
    fn into_response(self) -> Response {
        match self {
            GetUrlInfoResponse::Url(model) => (StatusCode::OK, Json(model)).into_response(),
            GetUrlInfoResponse::UrlNotFound => (
                StatusCode::NOT_FOUND,
                Json(BasicError {
                    error: "URL not found".to_string(),
                }),
            )
                .into_response(),
            GetUrlInfoResponse::DatabaseError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
        }
    }
}

impl From<sea_orm::DbErr> for GetUrlInfoResponse {
    fn from(e: sea_orm::DbErr) -> Self {
        GetUrlInfoResponse::DatabaseError(BasicError {
            error: e.to_string(),
        })
    }
}
