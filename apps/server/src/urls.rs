use axum::{
    debug_handler,
    extract::{Path, State},
    http::StatusCode,
    response::Redirect,
    Json,
};
use entity::short_link;
use sea_orm::{entity::*, query::*, DatabaseConnection, DbErr, RuntimeErr};
use tracing::{error, instrument};

use crate::{error::ServerError, state::ServerState, structs::NewUrlRequest};

// /api/shorten
#[instrument]
#[debug_handler]
pub async fn new_url(
    State(mut state): State<ServerState>,
    Json(payload): Json<NewUrlRequest>,
) -> Result<Json<short_link::Model>, ServerError> {
    let short: String = payload.short.unwrap_or_else(|| state.increment());

    let short_url = state.url.join(&short)?;

    let new_url = short_link::ActiveModel {
        id: ActiveValue::NotSet,
        url: ActiveValue::set(short.clone()),
        short_url: ActiveValue::set(short_url.as_str().to_string().to_owned()),
        user_id: ActiveValue::set(payload.user),
        expiry_date: ActiveValue::set(payload.expiry),
        original_url: ActiveValue::set(payload.url.clone()),
        created_at: ActiveValue::set(chrono::Utc::now().naive_utc()),
        updated_at: ActiveValue::set(chrono::Utc::now().naive_utc()),
        views: ActiveValue::set(0),
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
                            return Err(ServerError::DatabaseError(sql_err.to_string()));
                        }
                    }
                    _ => return Err(ServerError::DatabaseError(error.to_string())),
                },
                _ => return Err(ServerError::DatabaseError(err.to_string())),
            },
            DbErr::Conn(err) => match err {
                RuntimeErr::SqlxError(error) => match error {
                    sqlx::Error::Database(sql_err) => {
                        error!("SQL Error: {}", sql_err);
                        if sql_err.is_unique_violation() {
                            get_existing_url(payload.url.clone(), &state.conn).await?
                        } else {
                            return Err(ServerError::DatabaseError(sql_err.to_string()));
                        }
                    }
                    _ => return Err(ServerError::DatabaseError(error.to_string())),
                },
                _ => return Err(ServerError::DatabaseError(err.to_string())),
            },
            DbErr::Exec(err) => match err {
                RuntimeErr::SqlxError(error) => match error {
                    sqlx::Error::Database(sql_err) => {
                        error!("SQL Error: {}", sql_err);
                        if sql_err.is_unique_violation() {
                            get_existing_url(payload.url.clone(), &state.conn).await?
                        } else {
                            return Err(ServerError::DatabaseError(sql_err.to_string()));
                        }
                    }
                    _ => return Err(ServerError::DatabaseError(error.to_string())),
                },
                _ => return Err(ServerError::DatabaseError(err.to_string())),
            },
            DbErr::RecordNotInserted => {
                error!("Record not inserted");
                get_existing_url(payload.url.clone(), &state.conn).await?
            }
            _ => return Err(ServerError::DatabaseError(e.to_string())),
        },
    };

    state.cache.put(short, payload.url);

    Ok(Json(new))
}

#[instrument]
pub async fn get_existing_url(
    url: String,
    conn: &DatabaseConnection,
) -> Result<short_link::Model, ServerError> {
    let Some(link) = short_link::Entity::find()
        .filter(short_link::Column::OriginalUrl.eq(url))
        .one(conn)
        .await?
    else {
        return Err(ServerError::OptionError);
    };
    Ok(link)
}

// /{id}
#[instrument]
#[debug_handler]
pub async fn get_url(
    Path(id): Path<String>,
    State(mut state): State<ServerState>,
) -> Result<Redirect, ServerError> {
    if id.starts_with("/api") || id.starts_with("/ui") || id.starts_with("/auth") {
        return Ok(Redirect::permanent(&id));
    }
    let url = state.cache.get(&id);
    if let Some(url) = url {
        return Ok(Redirect::permanent(url));
    } else {
        let Some(short) = short_link::Entity::find()
            .filter(short_link::Column::Url.eq(&id))
            .one(&state.conn)
            .await?
        else {
            return Err(ServerError::OptionError);
        };
        state.cache.put(id, short.original_url.clone());
        Ok(Redirect::permanent(&short.original_url))
    }
}

// /api/url/delete/{id}
#[instrument]
#[debug_handler]
pub async fn delete_url(
    Path(id): Path<String>,
    State(mut state): State<ServerState>,
) -> Result<StatusCode, ServerError> {
    let Some(short) = short_link::Entity::find()
        .filter(short_link::Column::Url.eq(&id))
        .one(&state.conn)
        .await?
    else {
        return Err(ServerError::OptionError);
    };
    short.delete(&state.conn).await?;
    state.cache.pop(&id);
    Ok(StatusCode::OK)
}

// /api/url/update/{id}

#[instrument]
#[debug_handler]
pub async fn update_url(
    Path(id): Path<String>,
    State(state): State<ServerState>,
    Json(payload): Json<NewUrlRequest>,
) -> Result<Json<short_link::Model>, ServerError> {
    let Some(short) = short_link::Entity::find()
        .filter(short_link::Column::Url.eq(&id))
        .one(&state.conn)
        .await?
    else {
        return Err(ServerError::OptionError);
    };
    let mut new_url = short.into_active_model();
    if let Some(short_url) = payload.short {
        new_url.url = ActiveValue::Set(short_url.clone());
        new_url.short_url = ActiveValue::Set(state.url.clone().join(&short_url)?.into());
    }
    new_url.expiry_date = ActiveValue::Set(payload.expiry);
    new_url.url = ActiveValue::Set(payload.url);
    new_url.updated_at = ActiveValue::set(chrono::Utc::now().naive_utc());
    let short = new_url.insert(&state.conn).await?;

    Ok(Json(short))
}

// /api/url/{id}

#[instrument]
#[debug_handler]
pub async fn url_info(
    Path(id): Path<String>,
    State(state): State<ServerState>,
) -> Result<Json<short_link::Model>, ServerError> {
    let Some(short) = short_link::Entity::find()
        .filter(short_link::Column::Url.eq(&id))
        .one(&state.conn)
        .await?
    else {
        return Err(ServerError::OptionError);
    };
    Ok(Json(short))
}
