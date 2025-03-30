use axum::body::Body;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sea_orm::error::DbErr;

pub enum ServerError {
    DbError(DbErr),
}

impl From<DbErr> for ServerError {
    fn from(e: DbErr) -> Self {
        Self::DbError(e)
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            Self::DbError(e) => {
                let status = match e {
                    DbErr::RecordNotFound(_) => StatusCode::NOT_FOUND,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                };
                Response::builder()
                    .status(status)
                    .body(Body::from(format!("{:?}", e)))
                    .unwrap()
            }
        }
    }
}
