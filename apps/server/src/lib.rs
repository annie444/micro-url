pub mod actor;
pub mod api;
pub mod config;
pub mod error;
pub mod logger;
pub mod state;
pub mod urls;
pub mod user;
pub mod utils;

use std::{
    env::current_dir,
    net::SocketAddr,
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use axum::{
    Router,
    http::{HeaderName, Request},
    response::Redirect,
    routing::get,
};
#[cfg(feature = "ips")]
use axum_client_ip::ClientIpSource;
pub use config::{GetConfig, ServerConfig};
use logger::{init_subscriber, telemetry};
use state::ServerState;
use tower::{
    ServiceBuilder,
    layer::util::{Identity, Stack},
};
use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    compression::CompressionLayer,
    request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    services::fs::ServeDir,
    trace::TraceLayer,
};
use tracing::info;

use self::logger::MicroUrlMakeSpan;

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
    let addr = SocketAddr::from_str(config.internal_url.as_str())
        .unwrap_or_else(|_| panic!("Unable to parse socket {}", &config.internal_url.as_str()));
    info!("Listening on {}", addr);
    let listen = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(
        listen,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

#[tracing::instrument]
pub async fn init_router(config: config::ServerConfig, state: Option<ServerState>) -> Router {
    let trace_layer = telemetry();

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

    let layers = build_layers(x_request_id, trace_layer, &config);

    let state = match state {
        Some(state) => state,
        None => ServerState::new(config).await,
    };

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

    Router::new()
        .merge(app_routes)
        .merge(api_routes)
        .layer(layers)
}

#[cfg(feature = "ips")]
type MicroUrlServiceBuilder = ServiceBuilder<
    Stack<
        axum::Extension<ClientIpSource>,
        Stack<
            CompressionLayer,
            Stack<
                PropagateRequestIdLayer,
                Stack<
                    TraceLayer<SharedClassifier<ServerErrorsAsFailures>, MicroUrlMakeSpan>,
                    Stack<SetRequestIdLayer<MicroUrlMakeRequestId>, Identity>,
                >,
            >,
        >,
    >,
>;

#[cfg(feature = "ips")]
#[tracing::instrument]
pub(crate) fn build_layers(
    x_request_id: HeaderName,
    trace_layer: TraceLayer<SharedClassifier<ServerErrorsAsFailures>, MicroUrlMakeSpan>,
    config: &ServerConfig,
) -> MicroUrlServiceBuilder {
    ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(
            x_request_id.clone(),
            MicroUrlMakeRequestId::default(),
        ))
        .layer(trace_layer)
        .layer(PropagateRequestIdLayer::new(x_request_id))
        .layer(CompressionLayer::new())
        .layer(config.ip_source.clone().into_extension())
}

#[cfg(not(feature = "ips"))]
type MicroUrlServiceBuilder = ServiceBuilder<
    Stack<
        CompressionLayer,
        Stack<
            PropagateRequestIdLayer,
            Stack<
                TraceLayer<SharedClassifier<ServerErrorsAsFailures>, MicroUrlMakeSpan>,
                Stack<SetRequestIdLayer<MicroUrlMakeRequestId>, Identity>,
            >,
        >,
    >,
>;

#[cfg(not(feature = "ips"))]
#[tracing::instrument]
pub(crate) fn build_layers(
    x_request_id: HeaderName,
    trace_layer: TraceLayer<SharedClassifier<ServerErrorsAsFailures>, MicroUrlMakeSpan>,
    config: &ServerConfig,
) -> MicroUrlServiceBuilder {
    ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(
            x_request_id.clone(),
            MicroUrlMakeRequestId::default(),
        ))
        .layer(trace_layer)
        .layer(PropagateRequestIdLayer::new(x_request_id))
        .layer(CompressionLayer::new())
}
