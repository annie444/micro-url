pub mod actor;
pub mod msgs;
pub mod pool;
pub(super) mod tasks;

use std::sync::Arc;

pub use actor::PoolableActor;
pub use msgs::*;
pub use pool::*;
use tokio::sync::RwLock;
use tracing::instrument;

#[instrument(skip(actors))]
pub fn actor_pool(
    config: ActorPoolConfig,
    actors: Vec<impl PoolableActor>,
) -> Arc<RwLock<ActorPool>> {
    Arc::new(RwLock::new(ActorPool::new(config, actors)))
}
