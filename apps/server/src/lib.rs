pub mod api;
pub mod config;
pub mod error;
pub mod logger;
pub mod state;
pub mod structs;
pub mod urls;
pub mod user;

use std::{
    env::current_dir,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use axum::{
    http::{HeaderName, Request},
    response::Redirect,
    routing::get,
    Router,
};
pub use config::ServerConfig;
use logger::{init_subscriber, telemetry};
use state::ServerState;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    services::fs::ServeDir,
};
use tracing::info;

#[derive(Clone, Default)]
struct MicroUrlMakeRequestId {
    counter: Arc<AtomicU64>,
}

impl MakeRequestId for MicroUrlMakeRequestId {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let request_id = self
            .counter
            .fetch_add(1, Ordering::SeqCst)
            .to_string()
            .parse()
            .unwrap();

        Some(RequestId::new(request_id))
    }
}

#[tracing::instrument]
pub async fn run(config: ServerConfig) {
    init_subscriber();
    let app = init_router(config.clone(), None).await;
    let addr = config.internal_url;
    info!("Listening on {}", addr);
    let listen = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listen, app.into_make_service()).await.unwrap();
}

#[tracing::instrument]
pub async fn init_router(config: config::ServerConfig, state: Option<ServerState>) -> Router {
    let trace_layer = telemetry();
    let state = match state {
        Some(state) => state,
        None => ServerState::new(&config).await,
    };

    let x_request_id = HeaderName::from_static("x-request-id");

    let mut asset_path = current_dir().unwrap();
    asset_path.push(&config.assets_path);

    info!("Assets path: {:?}", asset_path);

    if !asset_path.exists() {
        panic!("Assets path does not exist: {:?}", asset_path);
    }

    if !asset_path.is_dir() {
        panic!("Assets path is not a directory: {:?}", asset_path);
    }

    let api_routes = api::router(state.clone());

    let app_routes = Router::new()
        // Base routes
        .nest_service(
            "/ui",
            ServeDir::new(asset_path).append_index_html_on_directories(true),
        )
        .route("/", get(|| async { Redirect::to("/ui/index.html") }))
        .route("/auth/callback", get(user::oidc::oidc_callback))
        .with_state(state);

    let app = Router::new().merge(app_routes).merge(api_routes).layer(
        ServiceBuilder::new()
            .layer(SetRequestIdLayer::new(
                x_request_id.clone(),
                MicroUrlMakeRequestId::default(),
            ))
            .layer(trace_layer)
            .layer(PropagateRequestIdLayer::new(x_request_id))
            .layer(CompressionLayer::new()),
    );

    app
}
