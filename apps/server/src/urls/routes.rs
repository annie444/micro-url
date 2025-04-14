#[cfg(feature = "ips")]
use std::net::IpAddr;

#[cfg(feature = "headers")]
use axum::http::header::HeaderMap;
use axum::{
    Json, debug_handler,
    extract::{Path, State},
};
#[cfg(feature = "ips")]
use axum_client_ip::ClientIp;
use entity::{short_link, views};
#[cfg(feature = "ips")]
use sea_orm::prelude::IpNetwork;
use sea_orm::{DatabaseConnection, DbErr, RuntimeErr, entity::*, query::*};
#[cfg(feature = "headers")]
use serde_json::json;
use tracing::{error, instrument, trace};

use super::structs::{GetExistingUrlError, NewUrlRequest, NewUrlResponse, UpdateUrlResponse};
#[cfg(feature = "headers")]
use crate::structs::HeaderMapDef;
use crate::{
    state::ServerState,
    urls::structs::{DeleteUrlResponse, GetUrlInfoResponse, GetUrlResponse},
};
// /api/shorten
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
#[cfg(not(feature = "ips"))]
#[cfg(not(feature = "headers"))]
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
        update_views(id.clone(), true, state.conn.clone());
        Ok(GetUrlResponse::Redirect(url.to_owned()))
    } else {
        update_views(id.clone(), false, state.conn.clone());
        let Some(short) = short_link::Entity::find_by_id(&id).one(&state.conn).await? else {
            return Err(GetUrlResponse::UrlNotFound);
        };
        state.cache.put(id, short.original_url.clone());
        Ok(GetUrlResponse::Redirect(short.original_url))
    }
}

#[cfg(feature = "ips")]
#[cfg(not(feature = "headers"))]
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
        update_views(id.clone(), true, ip, state.conn.clone());
        Ok(GetUrlResponse::Redirect(url.to_owned()))
    } else {
        update_views(id.clone(), false, ip, state.conn.clone());
        let Some(short) = short_link::Entity::find_by_id(&id).one(&state.conn).await? else {
            return Err(GetUrlResponse::UrlNotFound);
        };
        state.cache.put(id, short.original_url.clone());
        Ok(GetUrlResponse::Redirect(short.original_url))
    }
}

#[cfg(feature = "headers")]
#[cfg(not(feature = "ips"))]
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
        update_views(id.clone(), true, headers, state.conn.clone());
        Ok(GetUrlResponse::Redirect(url.to_owned()))
    } else {
        update_views(id.clone(), false, headers, state.conn.clone());
        let Some(short) = short_link::Entity::find_by_id(&id).one(&state.conn).await? else {
            return Err(GetUrlResponse::UrlNotFound);
        };
        state.cache.put(id, short.original_url.clone());
        Ok(GetUrlResponse::Redirect(short.original_url))
    }
}

#[cfg(feature = "headers")]
#[cfg(feature = "ips")]
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
        update_views(id.clone(), true, ip, headers, state.conn.clone());
        Ok(GetUrlResponse::Redirect(url.to_owned()))
    } else {
        update_views(id.clone(), false, ip, headers, state.conn.clone());
        let Some(short) = short_link::Entity::find_by_id(&id).one(&state.conn).await? else {
            return Err(GetUrlResponse::UrlNotFound);
        };
        state.cache.put(id, short.original_url.clone());
        Ok(GetUrlResponse::Redirect(short.original_url))
    }
}

#[cfg(not(feature = "ips"))]
#[cfg(not(feature = "headers"))]
#[instrument]
fn update_views(id: String, cached: bool, conn: DatabaseConnection) {
    tokio::spawn(async move {
        let view = views::ActiveModel {
            short_link: ActiveValue::Set(id.clone()),
            cache_hit: ActiveValue::Set(cached),
            created_at: ActiveValue::Set(chrono::Utc::now().naive_utc()),
            ..Default::default()
        };
        match view.insert(&conn).await {
            Ok(model) => trace!("Views updated successfully for url {}: {:?}", id, model),
            Err(e) => error!("Error updating views for url {}: {}", id, e.to_string()),
        };
    });
}

#[cfg(feature = "ips")]
#[cfg(not(feature = "headers"))]
#[instrument]
fn update_views(id: String, cached: bool, ip: IpAddr, conn: DatabaseConnection) {
    tokio::spawn(async move {
        let view = views::ActiveModel {
            short_link: ActiveValue::Set(id.clone()),
            ip: ActiveValue::Set(IpNetwork::new(ip, 0).ok()),
            cache_hit: ActiveValue::Set(cached),
            created_at: ActiveValue::Set(chrono::Utc::now().naive_utc()),
            ..Default::default()
        };
        match view.insert(&conn).await {
            Ok(model) => trace!("Views updated successfully for url {}: {:?}", id, model),
            Err(e) => error!("Error updating views for url {}: {}", id, e.to_string()),
        };
    });
}

#[cfg(feature = "headers")]
#[cfg(not(feature = "ips"))]
#[instrument]
fn update_views(id: String, cached: bool, headers: HeaderMap, conn: DatabaseConnection) {
    tokio::spawn(async move {
        let headers: HeaderMapDef = match headers.try_into() {
            Ok(hm) => hm,
            Err(e) => {
                error!("Unable to serialize the headers: {}", e.to_string());
                return;
            }
        };
        let view = views::ActiveModel {
            short_link: ActiveValue::Set(id.clone()),
            headers: ActiveValue::Set(Some(json!(headers))),
            cache_hit: ActiveValue::Set(cached),
            created_at: ActiveValue::Set(chrono::Utc::now().naive_utc()),
            ..Default::default()
        };
        match view.insert(&conn).await {
            Ok(model) => trace!("Views updated successfully for url {}: {:?}", id, model),
            Err(e) => error!("Error updating views for url {}: {}", id, e.to_string()),
        };
    });
}

#[cfg(feature = "headers")]
#[cfg(feature = "ips")]
#[instrument]
fn update_views(
    id: String,
    cached: bool,
    ip: IpAddr,
    headers: HeaderMap,
    conn: DatabaseConnection,
) {
    tokio::spawn(async move {
        let headers: HeaderMapDef = match headers.try_into() {
            Ok(hm) => hm,
            Err(e) => {
                error!("Unable to serialize the headers: {}", e.to_string());
                return;
            }
        };
        let view = views::ActiveModel {
            short_link: ActiveValue::Set(id.clone()),
            ip: ActiveValue::Set(IpNetwork::new(ip, 0).ok()),
            headers: ActiveValue::Set(Some(json!(headers))),
            cache_hit: ActiveValue::Set(cached),
            created_at: ActiveValue::Set(chrono::Utc::now().naive_utc()),
            ..Default::default()
        };
        match view.insert(&conn).await {
            Ok(model) => trace!("Views updated successfully for url {}: {:?}", id, model),
            Err(e) => error!("Error updating views for url {}: {}", id, e.to_string()),
        };
    });
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
