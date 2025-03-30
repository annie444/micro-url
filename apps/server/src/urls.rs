use crate::error::ServerError;
use crate::ServerState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::{debug_handler, Json};
use chrono::{DateTime, FixedOffset};
use entity::short_link;
use sea_orm::{entity::*, query::*};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUrl {
    #[serde(rename = "_id")]
    pub id: Option<i32>,
    pub url: String,
    pub short: Option<String>,
    pub user: Option<Uuid>,
    pub expiry: Option<DateTime<FixedOffset>>,
}

#[instrument]
#[debug_handler]
pub async fn new_url(
    State(mut state): State<ServerState>,
    Json(payload): Json<NewUrl>,
) -> Result<Json<NewUrl>, ServerError> {
    let short: String = payload.short.unwrap_or_else(|| state.increment());

    let new_url = short_link::ActiveModel {
        id: ActiveValue::NotSet,
        url: ActiveValue::set(short.clone()),
        short_url: ActiveValue::set(format!("{}/{}", state.url, short)),
        user_id: ActiveValue::set(payload.user),
        expiry_date: ActiveValue::set(payload.expiry),
        original_url: ActiveValue::set(payload.url.clone()),
        created_at: ActiveValue::set(chrono::Utc::now().naive_utc()),
        updated_at: ActiveValue::set(chrono::Utc::now().naive_utc()),
        views: ActiveValue::set(0),
    };
    let new = new_url.insert(&state.conn).await?;

    state.cache.put(short, payload.url);

    Ok(Json(NewUrl {
        id: Some(new.id),
        url: new.url,
        short: Some(new.short_url),
        user: new.user_id,
        expiry: new.expiry_date,
    }))
}

#[instrument]
#[debug_handler]
pub async fn get_url(
    Path(id): Path<String>,
    State(mut state): State<ServerState>,
) -> Result<Redirect, ServerError> {
    if id.len() <= 1 {
        return Ok(Redirect::permanent("/ui"));
    }
    let url = state.cache.get(&id);
    if let Some(url) = url {
        return Ok(Redirect::permanent(url));
    } else {
        let short = short_link::Entity::find()
            .filter(short_link::Column::Url.eq(&id))
            .one(&state.conn)
            .await?
            .unwrap();
        state.cache.put(id, short.original_url.clone());
        Ok(Redirect::permanent(&short.original_url))
    }
}

#[instrument]
#[debug_handler]
pub async fn delete_url(
    Path(id): Path<String>,
    State(mut state): State<ServerState>,
) -> Result<StatusCode, ServerError> {
    let short = short_link::Entity::find()
        .filter(short_link::Column::Url.eq(&id))
        .one(&state.conn)
        .await?
        .unwrap();
    short.delete(&state.conn).await?;
    state.cache.pop(&id);
    Ok(StatusCode::OK)
}
