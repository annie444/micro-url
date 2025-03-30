use axum::routing::{delete, get, post};
use axum::Router;

pub mod config;
pub mod error;
pub mod logger;
pub mod state;
pub mod urls;

use logger::telemetry;
use state::ServerState;

#[tokio::main]
async fn main() {
    let trace_layer = telemetry();
    let config = config::ServerConfig::from_env();
    let state = ServerState::new(&config).await;

    let app = Router::new()
        .route("/new_url", post(urls::new_url))
        .route("/delete/{id}", delete(urls::delete_url))
        .route("/{id}", get(urls::get_url))
        .with_state(state)
        .layer(trace_layer);

    let listener = tokio::net::TcpListener::bind(&config.internal_url)
        .await
        .unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
