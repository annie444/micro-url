use std::net::SocketAddr;

use axum::Router;
use server::{GetConfig, ServerConfig, init_router, state::ServerState};
use shuttle_runtime::{SecretStore, Service};
use sqlx::PgPool;

pub struct MicroUrlService(pub Router);

#[shuttle_runtime::async_trait]
impl Service for MicroUrlService {
    /// Takes the router that is returned by the user in the main function
    /// and binds to an address passed in by shuttle.
    async fn bind(mut self, addr: SocketAddr) -> Result<(), shuttle_runtime::Error> {
        axum::serve(
            shuttle_runtime::tokio::net::TcpListener::bind(addr).await?,
            self.0.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await?;

        Ok(())
    }
}

impl From<Router> for MicroUrlService {
    fn from(router: Router) -> Self {
        Self(router)
    }
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] db: PgPool,
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> Result<MicroUrlService, shuttle_runtime::Error> {
    let config = ServerConfig::from_secret(secrets);
    let state = ServerState::new_with_pool(config.clone(), db).await;
    let router = init_router(config, Some(state)).await;
    Ok(router.into())
}
