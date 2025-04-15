use std::io::Cursor;

#[cfg(feature = "headers")]
use axum::http::HeaderMap;
use axum::{
    Json, debug_handler,
    extract::{Path, Query, State},
};
#[cfg(feature = "ips")]
use axum_client_ip::ClientIp;
use entity::short_link;
use image::{ImageFormat, Rgba};
use qrcode::{EcLevel, QrCode, Version, render::Renderer};
use sea_orm::{DbErr, RuntimeErr, entity::*, query::*};
use tracing::{error, instrument};

use super::structs::{
    GetExistingUrlError, ImageFormats, NewUrlRequest, NewUrlResponse, QrCodeResponse,
    UpdateUrlResponse,
};
use crate::{
    actor::{ActorInputMessage, ViewInput},
    state::ServerState,
    urls::structs::{DeleteUrlResponse, GetUrlInfoResponse, GetUrlResponse, QrCodeParams},
};

#[instrument]
#[debug_handler]
#[utoipa::path(get, path = "/qr/{id}", context_path = super::URL_PREFIX, params(("id", description = "The short url ID"), QrCodeParams), responses(QrCodeResponse), tag = super::URL_TAG)]
pub async fn qr_code(
    Path(id): Path<String>,
    Query(format): Query<QrCodeParams>,
    State(mut state): State<ServerState>,
) -> Result<QrCodeResponse, QrCodeResponse> {
    if (format.fg_red.is_some()
        || format.fg_green.is_some()
        || format.fg_blue.is_some()
        || format.fg_alpha.is_some())
        && !(format.fg_red.is_some() && format.fg_green.is_some() && format.fg_blue.is_some())
    {
        return Err(QrCodeResponse::IncorrectParams(
            "Must supply all of fg_red, fg_green, and fg_blue"
                .to_string()
                .into(),
        ));
    }
    if (format.bg_red.is_some()
        || format.bg_green.is_some()
        || format.bg_blue.is_some()
        || format.bg_alpha.is_some())
        && !(format.bg_red.is_some() && format.bg_green.is_some() && format.bg_blue.is_some())
    {
        return Err(QrCodeResponse::IncorrectParams(
            "Must supply all of bg_red, bg_green, and bg_blue"
                .to_string()
                .into(),
        ));
    }

    let Some(short) = short_link::Entity::find_by_id(&id).one(&state.conn).await? else {
        return Err(QrCodeResponse::UrlNotFound);
    };

    state.cache.put(id, short.original_url.clone());

    let qr = QrCode::with_version(
        short.short_url.into_bytes(),
        Version::Normal(15),
        EcLevel::H,
    )?;

    let mut qr: Renderer<'_, Rgba<u8>> = qr.render();

    if let (Some(red), Some(green), Some(blue)) = (format.fg_red, format.fg_green, format.fg_blue) {
        if let Some(alpha) = format.fg_alpha {
            qr.dark_color(Rgba([red, green, blue, alpha]));
        } else {
            qr.dark_color(Rgba([red, green, blue, 100]));
        }
    }

    if let (Some(red), Some(green), Some(blue)) = (format.bg_red, format.bg_green, format.bg_blue) {
        if let Some(alpha) = format.bg_alpha {
            qr.light_color(Rgba([red, green, blue, alpha]));
        } else {
            qr.light_color(Rgba([red, green, blue, 100]));
        }
    }

    let mut img_buf = Cursor::new(Vec::new());

    let img = qr.build();

    match format.format {
        Some(format) => match format {
            ImageFormats::Png => {
                img.write_to(&mut img_buf, ImageFormat::Png)?;
                let data = img_buf.into_inner();
                Ok(QrCodeResponse::QrCodePng(data))
            }
            ImageFormats::Webp => {
                img.write_to(&mut img_buf, ImageFormat::WebP)?;
                let data = img_buf.into_inner();
                Ok(QrCodeResponse::QrCodeWebp(data))
            }
            ImageFormats::Jpeg => {
                img.write_to(&mut img_buf, ImageFormat::Jpeg)?;
                let data = img_buf.into_inner();
                Ok(QrCodeResponse::QrCodeJpeg(data))
            }
        },
        None => {
            img.write_to(&mut img_buf, ImageFormat::Png)?;
            let data = img_buf.into_inner();
            Ok(QrCodeResponse::QrCodePng(data))
        }
    }
}

#[instrument]
#[debug_handler]
#[utoipa::path(post, path = "/new", context_path = super::URL_PREFIX, request_body = NewUrlRequest, responses(NewUrlResponse), tag = super::URL_TAG)]
pub async fn new_url(
    State(mut state): State<ServerState>,
    Json(payload): Json<NewUrlRequest>,
) -> Result<NewUrlResponse, NewUrlResponse> {
    let short: String = payload.short.unwrap_or_else(|| state.increment());

    let short_url = state.url.join(&short)?;

    let new_url = short_link::ActiveModel {
        id: ActiveValue::set(short.clone()),
        short_url: ActiveValue::set(short_url.as_str().to_string().to_owned()),
        user_id: ActiveValue::set(payload.user),
        expiry_date: ActiveValue::set(payload.expiry),
        original_url: ActiveValue::set(payload.url.clone()),
        created_at: ActiveValue::set(chrono::Utc::now().naive_utc()),
        updated_at: ActiveValue::set(chrono::Utc::now().naive_utc()),
    };

    let new = match new_url.insert(&state.conn).await {
        Ok(new) => new,
        Err(e) => match e {
            DbErr::Query(err) => match err {
                RuntimeErr::SqlxError(error) => match error {
                    sqlx::Error::Database(sql_err) => {
                        error!("SQL Error: {}", sql_err);
                        if sql_err.is_unique_violation() {
                            get_existing_url(payload.url.clone(), &state.conn).await?
                        } else {
                            return Err(NewUrlResponse::DatabaseError(sql_err.to_string().into()));
                        }
                    }
                    _ => {
                        return Err(NewUrlResponse::DatabaseError(error.to_string().into()));
                    }
                },
                _ => return Err(NewUrlResponse::DatabaseError(err.to_string().into())),
            },
            DbErr::Conn(err) => match err {
                RuntimeErr::SqlxError(error) => match error {
                    sqlx::Error::Database(sql_err) => {
                        error!("SQL Error: {}", sql_err);
                        if sql_err.is_unique_violation() {
                            get_existing_url(payload.url.clone(), &state.conn).await?
                        } else {
                            return Err(NewUrlResponse::DatabaseError(sql_err.to_string().into()));
                        }
                    }
                    _ => {
                        return Err(NewUrlResponse::DatabaseError(error.to_string().into()));
                    }
                },
                _ => return Err(NewUrlResponse::DatabaseError(err.to_string().into())),
            },
            DbErr::Exec(err) => match err {
                RuntimeErr::SqlxError(error) => match error {
                    sqlx::Error::Database(sql_err) => {
                        error!("SQL Error: {}", sql_err);
                        if sql_err.is_unique_violation() {
                            get_existing_url(payload.url.clone(), &state.conn).await?
                        } else {
                            return Err(NewUrlResponse::DatabaseError(sql_err.to_string().into()));
                        }
                    }
                    _ => {
                        return Err(NewUrlResponse::DatabaseError(error.to_string().into()));
                    }
                },
                _ => return Err(NewUrlResponse::DatabaseError(err.to_string().into())),
            },
            DbErr::RecordNotInserted => {
                error!("Record not inserted");
                get_existing_url(payload.url.clone(), &state.conn).await?
            }
            _ => return Err(NewUrlResponse::DatabaseError(e.to_string().into())),
        },
    };

    state.cache.put(short, payload.url);

    Ok(NewUrlResponse::UrlCreated(new))
}

#[instrument(skip(conn))]
pub async fn get_existing_url(
    url: String,
    conn: &impl ConnectionTrait,
) -> Result<short_link::Model, GetExistingUrlError> {
    let Some(link) = short_link::Entity::find()
        .filter(short_link::Column::OriginalUrl.eq(url))
        .one(conn)
        .await?
    else {
        return Err(GetExistingUrlError::UrlNotFound);
    };
    Ok(link)
}

// /{id}
#[cfg(all(not(feature = "ips"), not(feature = "headers")))]
#[instrument]
#[debug_handler]
#[utoipa::path(get, path = "/{id}", params(("id", description = "The short url ID")), responses(GetUrlResponse), tag = super::URL_TAG)]
pub async fn get_url(
    Path(id): Path<String>,
    State(mut state): State<ServerState>,
) -> Result<GetUrlResponse, GetUrlResponse> {
    if id.starts_with("/api") || id.starts_with("/ui") || id.starts_with("/auth") {
        Ok(GetUrlResponse::Redirect(id))
    } else if let Some(url) = state.cache.get(&id) {
        state
            .pool
            .send(ActorInputMessage::UpdateViews(ViewInput {
                id,
                cached: true,
                conn: state.conn.clone(),
            }))
            .await?;
        Ok(GetUrlResponse::Redirect(url.to_owned()))
    } else {
        state
            .pool
            .send(ActorInputMessage::UpdateViews(ViewInput {
                id: id.clone(),
                cached: false,
                conn: state.conn.clone(),
            }))
            .await?;
        let Some(short) = short_link::Entity::find_by_id(&id).one(&state.conn).await? else {
            return Err(GetUrlResponse::UrlNotFound);
        };
        state.cache.put(id, short.original_url.clone());
        Ok(GetUrlResponse::Redirect(short.original_url))
    }
}

#[cfg(all(feature = "ips", not(feature = "headers")))]
#[instrument]
#[debug_handler]
#[utoipa::path(get, path = "/{id}", params(("id", description = "The short url ID")), responses(GetUrlResponse), tag = super::URL_TAG)]
pub async fn get_url(
    Path(id): Path<String>,
    State(mut state): State<ServerState>,
    ClientIp(ip): ClientIp,
) -> Result<GetUrlResponse, GetUrlResponse> {
    if id.starts_with("/api") || id.starts_with("/ui") || id.starts_with("/auth") {
        Ok(GetUrlResponse::Redirect(id))
    } else if let Some(url) = state.cache.get(&id) {
        state
            .pool
            .send(ActorInputMessage::UpdateViews(ViewInput {
                id,
                cached: true,
                ip,
                conn: state.conn.clone(),
            }))
            .await?;
        Ok(GetUrlResponse::Redirect(url.to_owned()))
    } else {
        state
            .pool
            .send(ActorInputMessage::UpdateViews(ViewInput {
                id: id.clone(),
                cached: false,
                ip,
                conn: state.conn.clone(),
            }))
            .await?;
        let Some(short) = short_link::Entity::find_by_id(&id).one(&state.conn).await? else {
            return Err(GetUrlResponse::UrlNotFound);
        };
        state.cache.put(id, short.original_url.clone());
        Ok(GetUrlResponse::Redirect(short.original_url))
    }
}

#[cfg(all(feature = "headers", not(feature = "ips")))]
#[instrument]
#[debug_handler]
#[utoipa::path(get, path = "/{id}", params(("id", description = "The short url ID")), responses(GetUrlResponse), tag = super::URL_TAG)]
pub async fn get_url(
    Path(id): Path<String>,
    State(mut state): State<ServerState>,
    headers: HeaderMap,
) -> Result<GetUrlResponse, GetUrlResponse> {
    if id.starts_with("/api") || id.starts_with("/ui") || id.starts_with("/auth") {
        Ok(GetUrlResponse::Redirect(id))
    } else if let Some(url) = state.cache.get(&id) {
        state
            .pool
            .send(ActorInputMessage::UpdateViews(ViewInput {
                id,
                cached: true,
                headers,
                conn: state.conn.clone(),
            }))
            .await?;
        Ok(GetUrlResponse::Redirect(url.to_owned()))
    } else {
        state
            .pool
            .send(ActorInputMessage::UpdateViews(ViewInput {
                id: id.clone(),
                cached: false,
                headers,
                conn: state.conn.clone(),
            }))
            .await?;
        let Some(short) = short_link::Entity::find_by_id(&id).one(&state.conn).await? else {
            return Err(GetUrlResponse::UrlNotFound);
        };
        state.cache.put(id, short.original_url.clone());
        Ok(GetUrlResponse::Redirect(short.original_url))
    }
}

#[cfg(all(feature = "headers", feature = "ips"))]
#[instrument]
#[debug_handler]
#[utoipa::path(get, path = "/{id}", params(("id", description = "The short url ID")), responses(GetUrlResponse), tag = super::URL_TAG)]
pub async fn get_url(
    Path(id): Path<String>,
    State(mut state): State<ServerState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
) -> Result<GetUrlResponse, GetUrlResponse> {
    if id.starts_with("/api") || id.starts_with("/ui") || id.starts_with("/auth") {
        Ok(GetUrlResponse::Redirect(id))
    } else if let Some(url) = state.cache.get(&id) {
        state
            .pool
            .send(ActorInputMessage::UpdateViews(ViewInput {
                id,
                cached: true,
                ip,
                headers,
                conn: state.conn.clone(),
            }))
            .await?;
        Ok(GetUrlResponse::Redirect(url.to_owned()))
    } else {
        state
            .pool
            .send(ActorInputMessage::UpdateViews(ViewInput {
                id: id.clone(),
                cached: false,
                ip,
                headers,
                conn: state.conn.clone(),
            }))
            .await?;
        let Some(short) = short_link::Entity::find_by_id(&id).one(&state.conn).await? else {
            return Err(GetUrlResponse::UrlNotFound);
        };
        state.cache.put(id, short.original_url.clone());
        Ok(GetUrlResponse::Redirect(short.original_url))
    }
}

// /api/url/delete/{id}
#[instrument]
#[debug_handler]
#[utoipa::path(delete, path = "/delete/{id}", params(("id", description = "The short url ID")), context_path = super::URL_PREFIX, responses(DeleteUrlResponse), tag = super::URL_TAG)]
pub async fn delete_url(
    Path(id): Path<String>,
    State(mut state): State<ServerState>,
) -> Result<DeleteUrlResponse, DeleteUrlResponse> {
    let Some(short) = short_link::Entity::find_by_id(&id).one(&state.conn).await? else {
        return Err(DeleteUrlResponse::UrlNotFound);
    };
    short.delete(&state.conn).await?;
    state.cache.pop(&id);
    Ok(DeleteUrlResponse::UrlDeleted)
}

// /api/url/update/{id}
#[instrument]
#[debug_handler]
#[utoipa::path(put, path = "/update/{id}", params(("id", description = "The short url ID")), context_path = super::URL_PREFIX, request_body = NewUrlRequest, responses(UpdateUrlResponse), tag = super::URL_TAG)]
pub async fn update_url(
    Path(id): Path<String>,
    State(state): State<ServerState>,
    Json(payload): Json<NewUrlRequest>,
) -> Result<UpdateUrlResponse, UpdateUrlResponse> {
    let Some(short) = short_link::Entity::find_by_id(&id).one(&state.conn).await? else {
        return Err(UpdateUrlResponse::UrlNotFound);
    };
    let mut new_url = short.into_active_model();
    if let Some(short_url) = payload.short {
        new_url.id = ActiveValue::Set(short_url.clone());
        new_url.short_url = ActiveValue::Set(state.url.clone().join(&short_url)?.into());
    }
    new_url.expiry_date = ActiveValue::Set(payload.expiry);
    new_url.id = ActiveValue::Set(payload.url);
    new_url.updated_at = ActiveValue::set(chrono::Utc::now().naive_utc());
    let short = new_url.update(&state.conn).await?;

    Ok(UpdateUrlResponse::UrlUpdated(short))
}

// /api/url/{id}
#[instrument]
#[debug_handler]
#[utoipa::path(get, path = "/{id}", params(("id", description = "The short url ID")), context_path = super::URL_PREFIX, responses(GetUrlInfoResponse), tag = super::URL_TAG)]
pub async fn url_info(
    Path(id): Path<String>,
    State(state): State<ServerState>,
) -> Result<GetUrlInfoResponse, GetUrlInfoResponse> {
    let Some(short) = short_link::Entity::find_by_id(&id).one(&state.conn).await? else {
        return Err(GetUrlInfoResponse::UrlNotFound);
    };
    Ok(GetUrlInfoResponse::Url(short))
}
