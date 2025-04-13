use std::{
    env,
    ops::Drop,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use async_channel::{SendError, Sender, bounded};
use serde::{Deserialize, Serialize};
use tokio::{
    runtime::{Builder, Handle, Runtime},
    sync::RwLock,
};
use tracing::instrument;

use super::{
    ActorInputMessage,
    actor::{DefaultActor, PoolableActor},
};
use crate::config::GetConfig;

#[derive(Clone, Debug)]
pub struct ActorPool {
    in_sender: Option<Sender<ActorInputMessage>>,
    rt: Arc<Runtime>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActorPoolConfig {
    pub workers: usize,
    pub blocking_workers: usize,
    pub stack_size: usize,
    pub keep_alive: Duration,
    pub event_interval: u32,
}

impl Default for ActorPoolConfig {
    fn default() -> Self {
        Self {
            workers: 4,
            blocking_workers: 2,
            stack_size: 32 * 1024,
            keep_alive: Duration::from_secs(10),
            event_interval: 61,
        }
    }
}

impl GetConfig for ActorPoolConfig {
    #[instrument]
    fn from_env() -> Self {
        dotenvy::dotenv().ok();
        let workers: usize = env::var("ACTOR_WORKERS")
            .ok()
            .map(|s| {
                s.parse()
                    .expect("Unable to coerce ACTOR_WORKERS into an integer")
            })
            .unwrap_or(4);
        let blocking_workers: usize = env::var("ACTOR_BLOCKING_WORKERS")
            .ok()
            .map(|s| {
                s.parse()
                    .expect("Unable to coerce ACTOR_BLOCKING_WORKERS into an integer")
            })
            .unwrap_or(2);
        let stack_size: usize = env::var("ACTOR_STACK_SIZE")
            .ok()
            .map(|s| {
                s.parse()
                    .expect("Unable to coerce ACTOR_STACK_SIZE into an integer")
            })
            .unwrap_or(32 * 1024);
        let keep_alive: Duration = env::var("ACTOR_KEEP_ALIVE")
            .ok()
            .map(|s| {
                Duration::from_secs(
                    s.parse()
                        .expect("Unable to coerce ACTOR_KEEP_ALIVE into a duration"),
                )
            })
            .unwrap_or(Duration::from_secs(10));
        let event_interval: u32 = env::var("ACTOR_EVENT_INTERVAL")
            .ok()
            .map(|s| {
                s.parse()
                    .expect("Unable to coerce ACTOR_EVENT_INTERVAL into an integer")
            })
            .unwrap_or(61);
        Self {
            workers,
            blocking_workers,
            stack_size,
            keep_alive,
            event_interval,
        }
    }

    #[instrument(skip(secrets))]
    fn from_secret(secrets: shuttle_runtime::SecretStore) -> Self {
        let workers: usize = secrets
            .get("ACTOR_WORKERS")
            .map(|s| {
                s.parse()
                    .expect("Unable to coerce ACTOR_WORKERS into an integer")
            })
            .unwrap_or(4);
        let blocking_workers: usize = secrets
            .get("ACTOR_BLOCKING_WORKERS")
            .map(|s| {
                s.parse()
                    .expect("Unable to coerce ACTOR_BLOCKING_WORKERS into an integer")
            })
            .unwrap_or(2);
        let stack_size: usize = secrets
            .get("ACTOR_STACK_SIZE")
            .map(|s| {
                s.parse()
                    .expect("Unable to coerce ACTOR_STACK_SIZE into an integer")
            })
            .unwrap_or(32 * 1024);
        let keep_alive: Duration = secrets
            .get("ACTOR_KEEP_ALIVE")
            .map(|s| {
                Duration::from_secs(
                    s.parse()
                        .expect("Unable to coerce ACTOR_KEEP_ALIVE into a duration"),
                )
            })
            .unwrap_or(Duration::from_secs(10));
        let event_interval: u32 = secrets
            .get("ACTOR_EVENT_INTERVAL")
            .map(|s| {
                s.parse()
                    .expect("Unable to coerce ACTOR_EVENT_INTERVAL into an integer")
            })
            .unwrap_or(61);
        Self {
            workers,
            blocking_workers,
            stack_size,
            keep_alive,
            event_interval,
        }
    }
}

impl ActorPool {
    #[instrument]
    pub fn new(config: &ActorPoolConfig) -> Self {
        let num_chanels = (config.workers + config.blocking_workers) * 2;
        let (in_sender, in_receiver) = bounded(num_chanels);
        let rt = Builder::new_multi_thread()
            .enable_all()
            .worker_threads(config.workers)
            .max_blocking_threads(config.blocking_workers)
            .thread_name_fn(|| {
                static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
                let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
                format!("micro-url-worker-{}", id)
            })
            .thread_stack_size(config.stack_size)
            .thread_keep_alive(config.keep_alive)
            .event_interval(config.event_interval)
            .build()
            .expect("Unable to build tokio runtime");
        for _ in 0..config.workers {
            let in_receiver = in_receiver.clone();
            rt.spawn(async move { DefaultActor::new(in_receiver).run().await });
        }

        Self {
            in_sender: Some(in_sender),
            rt: Arc::new(rt),
        }
    }

    #[instrument]
    pub fn new_locked(config: &ActorPoolConfig) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self::new(config)))
    }

    #[instrument]
    pub fn handle(&self) -> &Handle {
        self.rt.handle()
    }

    #[instrument]
    pub async fn send(&self, msg: ActorInputMessage) -> Result<(), SendError<ActorInputMessage>> {
        match &self.in_sender {
            Some(in_sender) => in_sender.send(msg).await,
            None => Err(SendError(msg)),
        }
    }

    #[instrument]
    pub fn send_blocking(
        &self,
        msg: ActorInputMessage,
    ) -> Result<(), SendError<ActorInputMessage>> {
        match &self.in_sender {
            Some(in_sender) => in_sender.send_blocking(msg),
            None => Err(SendError(msg)),
        }
    }

    #[instrument]
    pub fn close(&mut self) {
        if let Some(rt) = Arc::into_inner(self.rt.clone()) {
            rt.shutdown_timeout(Duration::from_secs(3));
        }
        self.in_sender.take();
    }
}

impl Drop for ActorPool {
    fn drop(&mut self) {
        self.close()
    }
}
