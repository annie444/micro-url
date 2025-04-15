use std::{borrow::Cow, fmt::Display};

use axum::{
    Json,
    body::Body,
    http::{StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use chrono::NaiveDateTime;
use entity::short_link;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::{IntoParams, IntoResponses, ToSchema};
use uuid::Uuid;

use crate::utils::BasicError;

#[derive(Debug, Clone, Serialize, Deserialize, IntoParams, TS)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
#[into_params(parameter_in = Query, style = Form)]
pub struct QrCodeParams {
    #[ts(optional)]
    pub format: Option<ImageFormats>,
    #[ts(optional)]
    pub bg_red: Option<u8>,
    #[ts(optional)]
    pub bg_green: Option<u8>,
    #[ts(optional)]
    pub bg_blue: Option<u8>,
    #[ts(optional)]
    pub bg_alpha: Option<u8>,
    #[ts(optional)]
    pub fg_red: Option<u8>,
    #[ts(optional)]
    pub fg_green: Option<u8>,
    #[ts(optional)]
    pub fg_blue: Option<u8>,
    #[ts(optional)]
    pub fg_alpha: Option<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
#[serde(rename_all = "lowercase")]
pub enum ImageFormats {
    Png,
    Webp,
    Jpeg,
}

impl Display for ImageFormats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Png => write!(f, "png"),
            Self::Webp => write!(f, "webp"),
            Self::Jpeg => write!(f, "jpeg"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, IntoResponses, TS)]
#[serde(untagged)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
pub enum QrCodeResponse {
    #[response(status = StatusCode::BAD_REQUEST)]
    UrlParseError(#[to_schema] BasicError),
    #[response(status = StatusCode::INTERNAL_SERVER_ERROR)]
    DatabaseError(#[to_schema] BasicError),
    #[response(status = StatusCode::NOT_FOUND)]
    UrlNotFound,
    #[response(status = StatusCode::INTERNAL_SERVER_ERROR)]
    EncodingError(#[to_schema] BasicError),
    #[response(status = StatusCode::BAD_REQUEST)]
    IncorrectParams(#[to_schema] BasicError),
    #[response(status = StatusCode::OK, content_type = "image/png; charset=utf-8", headers(["content-disposition: attachment; filename=\"qrcode.png\""]))]
    QrCodePng(#[to_schema] Vec<u8>),
    #[response(status = StatusCode::OK, content_type = "image/jpeg; charset=utf-8", headers(["content-disposition: attachment; filename=\"qrcode.jpeg\""]))]
    QrCodeJpeg(#[to_schema] Vec<u8>),
    #[response(status = StatusCode::OK, content_type = "image/webp; charset=utf-8", headers(["Content-Disposition: attachment; filename=\"qrcode.webp\""]))]
    QrCodeWebp(#[to_schema] Vec<u8>),
}

impl From<sea_orm::DbErr> for QrCodeResponse {
    fn from(value: sea_orm::DbErr) -> Self {
        Self::DatabaseError(value.to_string().into())
    }
}

impl From<qrcode::types::QrError> for QrCodeResponse {
    fn from(value: qrcode::types::QrError) -> Self {
        Self::EncodingError(value.to_string().into())
    }
}

impl From<image::error::ImageError> for QrCodeResponse {
    fn from(value: image::error::ImageError) -> Self {
        Self::EncodingError(value.to_string().into())
    }
}

fn image_response(img: Vec<u8>, content_type: &str) -> Response {
    match Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            format!("image/{}; charset=utf-8", content_type),
        )
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"qrcode.{}\"", content_type),
        )
        .body(Body::from(Cow::<'static, [u8]>::Owned(img)))
    {
        Ok(res) => res,
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(BasicError {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

impl IntoResponse for QrCodeResponse {
    fn into_response(self) -> Response {
        match self {
            Self::UrlNotFound => (
                StatusCode::NOT_FOUND,
                Json(BasicError {
                    error: "URL not found".to_string(),
                }),
            )
                .into_response(),
            Self::EncodingError(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response(),
            Self::DatabaseError(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response(),
            Self::UrlParseError(e) => (StatusCode::BAD_REQUEST, Json(e)).into_response(),
            Self::IncorrectParams(e) => (StatusCode::BAD_REQUEST, Json(e)).into_response(),
            Self::QrCodePng(png) => image_response(png, "png"),
            Self::QrCodeJpeg(jpeg) => image_response(jpeg, "jpeg"),
            Self::QrCodeWebp(webp) => image_response(webp, "webp"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
pub struct NewUrlRequest {
    pub url: String,
    pub short: Option<String>,
    pub user: Option<Uuid>,
    pub expiry: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, IntoResponses, TS)]
#[serde(untagged)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
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

#[derive(Debug, Clone, Serialize, Deserialize, IntoResponses, TS)]
#[serde(untagged)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
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

#[derive(Debug, Clone, Serialize, Deserialize, IntoResponses, TS)]
#[serde(untagged)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
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

#[derive(Debug, Clone, Serialize, Deserialize, IntoResponses, TS)]
#[serde(untagged)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
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

#[derive(Debug, Clone, Serialize, Deserialize, IntoResponses, TS)]
#[serde(untagged)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
pub enum GetUrlResponse {
    #[response(status = StatusCode::INTERNAL_SERVER_ERROR)]
    DatabaseError(#[to_schema] BasicError),
    #[response(status = StatusCode::NOT_FOUND)]
    UrlNotFound,
    #[response(status = StatusCode::PERMANENT_REDIRECT)]
    Redirect(#[to_schema] String),
    #[response(status = StatusCode::INTERNAL_SERVER_ERROR)]
    ViewError(#[to_schema] BasicError),
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
            GetUrlResponse::ViewError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
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

impl From<async_channel::SendError<crate::actor::ActorInputMessage>> for GetUrlResponse {
    fn from(value: async_channel::SendError<crate::actor::ActorInputMessage>) -> Self {
        Self::ViewError(BasicError {
            error: value.to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, IntoResponses, TS)]
#[serde(untagged)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
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
