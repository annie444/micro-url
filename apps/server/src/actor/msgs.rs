#[cfg(feature = "ips")]
use std::net::IpAddr;

#[cfg(feature = "headers")]
use axum::http::HeaderMap;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::error::ServerError;

#[derive(Debug, Clone, Default)]
pub enum ActorInputMessage {
    #[default]
    None,
    CleanSessions(DbInput),
    CleanUrls(DbInput),
    UpdateViews(ViewInput),
}

#[derive(Debug, Clone, Default)]
pub struct DbInput {
    pub conn: DatabaseConnection,
}

#[cfg(all(feature = "headers", feature = "ips"))]
#[derive(Debug, Clone)]
pub struct ViewInput {
    pub id: String,
    pub cached: bool,
    pub ip: IpAddr,
    pub headers: HeaderMap,
    pub conn: DatabaseConnection,
}

#[cfg(all(feature = "headers", not(feature = "ips")))]
#[derive(Debug, Clone)]
pub struct ViewInput {
    pub id: String,
    pub cached: bool,
    pub headers: HeaderMap,
    pub conn: DatabaseConnection,
}

#[cfg(all(not(feature = "headers"), feature = "ips"))]
#[derive(Debug, Clone)]
pub struct ViewInput {
    pub id: String,
    pub cached: bool,
    pub ip: IpAddr,
    pub conn: DatabaseConnection,
}

#[cfg(all(not(feature = "headers"), not(feature = "ips")))]
#[derive(Debug, Clone)]
pub struct ViewInput {
    pub id: String,
    pub cached: bool,
    pub conn: DatabaseConnection,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActorOutputMessage {
    pub msg: String,
}

#[derive(Error, Debug)]
pub enum ActorError {
    #[error("Database error: {0}")]
    DbErr(#[from] sea_orm::DbErr),
    #[error("Database transaction Error: {0}")]
    TransactionError(#[from] sea_orm::TransactionError<sea_orm::DbErr>),
    #[error("Actor error from server error: {0}")]
    ServerError(#[from] ServerError),
    #[error("Actor error: {msg}")]
    Basic { msg: String },
}
