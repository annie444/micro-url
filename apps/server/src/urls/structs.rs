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
use tracing::{error, info, instrument};
use ts_rs::TS;
use utoipa::{IntoParams, IntoResponses, ToSchema};
use uuid::Uuid;

use crate::{error::ArcMutexError, utils::BasicError};

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
    #[response(status = StatusCode::INTERNAL_SERVER_ERROR)]
    CacheError(#[to_schema] BasicError),
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

impl From<ArcMutexError> for QrCodeResponse {
    fn from(value: ArcMutexError) -> Self {
        Self::CacheError(value.to_string().into())
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
    #[instrument]
    fn into_response(self) -> Response {
        match self {
            Self::CacheError(e) => {
                error!(%e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
            Self::UrlNotFound => {
                error!("Url not found");
                (
                    StatusCode::NOT_FOUND,
                    Json(BasicError {
                        error: "URL not found".to_string(),
                    }),
                )
                    .into_response()
            }
            Self::EncodingError(e) => {
                error!(%e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
            Self::DatabaseError(e) => {
                error!(%e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
            Self::UrlParseError(e) => {
                error!(%e);
                (StatusCode::BAD_REQUEST, Json(e)).into_response()
            }
            Self::IncorrectParams(e) => {
                error!(%e);
                (StatusCode::BAD_REQUEST, Json(e)).into_response()
            }
            Self::QrCodePng(png) => {
                info!("Encoding PNG");
                image_response(png, "png")
            }
            Self::QrCodeJpeg(jpeg) => {
                info!("Encoding JPEG");
                image_response(jpeg, "jpeg")
            }
            Self::QrCodeWebp(webp) => {
                info!("Encoding WEBP");
                image_response(webp, "webp")
            }
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
    #[response(status = StatusCode::INTERNAL_SERVER_ERROR)]
    CacheError(#[to_schema] BasicError),
    #[response(status = StatusCode::OK)]
    UrlCreated(#[to_schema] short_link::Model),
}

impl IntoResponse for NewUrlResponse {
    #[instrument]
    fn into_response(self) -> Response {
        match self {
            NewUrlResponse::UrlCreated(model) => {
                info!("{:?}", model);
                (StatusCode::OK, Json(model)).into_response()
            }
            Self::CacheError(e) => {
                error!(%e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
            NewUrlResponse::UrlNotFound => {
                error!("URL not found");
                (
                    StatusCode::NOT_FOUND,
                    Json(BasicError {
                        error: "URL not found".to_string(),
                    }),
                )
                    .into_response()
            }
            NewUrlResponse::DatabaseError(e) => {
                error!(%e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
            NewUrlResponse::UrlParseError(e) => {
                error!(%e);
                (StatusCode::BAD_REQUEST, Json(e)).into_response()
            }
        }
    }
}

impl From<ArcMutexError> for NewUrlResponse {
    fn from(value: ArcMutexError) -> Self {
        Self::CacheError(value.to_string().into())
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
    #[response(status = StatusCode::INTERNAL_SERVER_ERROR)]
    CacheError(#[to_schema] BasicError),
    #[response(status = StatusCode::BAD_REQUEST)]
    UrlNotFound,
    #[response(status = StatusCode::OK)]
    UrlDeleted,
}

impl IntoResponse for DeleteUrlResponse {
    #[instrument]
    fn into_response(self) -> Response {
        match self {
            Self::CacheError(e) => {
                error!(%e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
            DeleteUrlResponse::UrlDeleted => {
                info!("URL deleted");
                (
                    StatusCode::OK,
                    Json(BasicError {
                        error: "OK".to_string(),
                    }),
                )
                    .into_response()
            }
            DeleteUrlResponse::UrlNotFound => {
                error!("Url not found");
                (
                    StatusCode::BAD_REQUEST,
                    Json(BasicError {
                        error: "URL not found".to_string(),
                    }),
                )
                    .into_response()
            }
            DeleteUrlResponse::DatabaseError(e) => {
                error!(%e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
        }
    }
}

impl From<ArcMutexError> for DeleteUrlResponse {
    fn from(value: ArcMutexError) -> Self {
        Self::CacheError(value.to_string().into())
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
    #[instrument]
    fn into_response(self) -> Response {
        match self {
            UpdateUrlResponse::UrlUpdated(model) => {
                info!("{:?}", model);
                (StatusCode::OK, Json(model)).into_response()
            }
            UpdateUrlResponse::UrlNotFound => {
                error!("URL not found");
                (
                    StatusCode::NOT_FOUND,
                    Json(BasicError {
                        error: "URL not found".to_string(),
                    }),
                )
                    .into_response()
            }
            UpdateUrlResponse::DatabaseError(e) => {
                error!(%e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
            UpdateUrlResponse::UrlParseError(e) => {
                error!(%e);
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
    #[response(status = StatusCode::INTERNAL_SERVER_ERROR)]
    CacheError(#[to_schema] BasicError),
}

impl IntoResponse for GetUrlResponse {
    #[instrument]
    fn into_response(self) -> Response {
        match self {
            Self::CacheError(e) => {
                error!(%e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
            GetUrlResponse::UrlNotFound => {
                error!("URL not found");
                (
                    StatusCode::NOT_FOUND,
                    Json(BasicError {
                        error: "URL not found".to_string(),
                    }),
                )
                    .into_response()
            }
            GetUrlResponse::DatabaseError(e) => {
                error!(%e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
            GetUrlResponse::Redirect(url) => {
                info!(url);
                Redirect::permanent(&url).into_response()
            }
            GetUrlResponse::ViewError(e) => {
                error!(%e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response()
            }
        }
    }
}

impl From<ArcMutexError> for GetUrlResponse {
    fn from(value: ArcMutexError) -> Self {
        Self::CacheError(value.to_string().into())
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
    #[instrument]
    fn into_response(self) -> Response {
        match self {
            GetUrlInfoResponse::Url(model) => {
                info!("{:?}", model);
                (StatusCode::OK, Json(model)).into_response()
            }
            GetUrlInfoResponse::UrlNotFound => {
                error!("URL not found");
                (
                    StatusCode::NOT_FOUND,
                    Json(BasicError {
                        error: "URL not found".to_string(),
                    }),
                )
                    .into_response()
            }
            GetUrlInfoResponse::DatabaseError(e) => {
                error!(%e);
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
