use axum::extract::{MatchedPath, Request};
use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    trace::{MakeSpan, TraceLayer},
};
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

#[cfg(debug_assertions)]
pub const DEFAULT_MESSAGE_LEVEL: Level = Level::DEBUG;
#[cfg(not(debug_assertions))]
pub const DEFAULT_MESSAGE_LEVEL: Level = Level::INFO;

#[derive(Debug, Clone)]
pub struct MicroUrlMakeSpan {
    level: Level,
    include_headers: bool,
}

impl MicroUrlMakeSpan {
    /// Create a new `DefaultMakeSpan`.
    pub fn new() -> Self {
        Self {
            level: DEFAULT_MESSAGE_LEVEL,
            include_headers: false,
        }
    }

    /// Set the [`Level`] used for the [tracing span].
    ///
    /// Defaults to [`Level::DEBUG`].
    ///
    /// [tracing span]: https://docs.rs/tracing/latest/tracing/#spans
    pub fn level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    /// Include request headers on the [`Span`].
    ///
    /// By default headers are not included.
    ///
    /// [`Span`]: tracing::Span
    pub fn include_headers(mut self, include_headers: bool) -> Self {
        self.include_headers = include_headers;
        self
    }
}

impl Default for MicroUrlMakeSpan {
    fn default() -> Self {
        Self::new()
    }
}

impl<B> MakeSpan<B> for MicroUrlMakeSpan {
    fn make_span(&mut self, request: &Request<B>) -> tracing::Span {
        // This ugly macro is needed, unfortunately, because `tracing::span!`
        // required the level argument to be static. Meaning we can't just pass
        // `self.level`.
        macro_rules! make_span {
            ($level:expr) => {
                if self.include_headers {
                    tracing::span!(
                        $level,
                        "request",
                        method = %request.method(),
                        uri = %request.uri(),
                        version = ?request.version(),
                        headers = ?request.headers(),
                        path = ?request
                            .extensions()
                            .get::<MatchedPath>()
                            .map(|matched_path| matched_path.as_str()),
                    )
                } else {
                    tracing::span!(
                        $level,
                        "request",
                        method = %request.method(),
                        uri = %request.uri(),
                        version = ?request.version(),
                        path = ?request
                                    .extensions()
                                    .get::<MatchedPath>()
                                    .map(|matched_path| matched_path.as_str()),
                    )
                }
            }
        }

        match self.level {
            Level::ERROR => make_span!(Level::ERROR),
            Level::WARN => make_span!(Level::WARN),
            Level::INFO => make_span!(Level::INFO),
            Level::DEBUG => make_span!(Level::DEBUG),
            Level::TRACE => make_span!(Level::TRACE),
        }
    }
}

pub fn telemetry() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>, MicroUrlMakeSpan> {
    tracing_subscriber::Registry::default()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            // axum logs rejections from built-in extractors with the `axum::rejection`
            // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
            format!(
                "{}=debug,tower_http=debug,axum::rejection=trace",
                env!("CARGO_CRATE_NAME")
            )
            .into()
        }))
        .with(fmt::layer())
        .init();
    TraceLayer::new_for_http().make_span_with(MicroUrlMakeSpan::new().include_headers(true))
}
