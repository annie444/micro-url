use server::{init_router, state::ServerState, ServerConfig};
use shuttle_runtime::SecretStore;
use sqlx::PgPool;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] db: PgPool,
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_axum::ShuttleAxum {
    let config = ServerConfig::from_secret(secrets);
    let state = ServerState::new_with_pool(&config, db).await;
    let router = init_router(config, Some(state)).await;
    Ok(router.into())
}
