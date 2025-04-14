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
use chrono::TimeDelta;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tokio::{
    runtime::{Builder, Handle, Runtime},
    sync::RwLock,
    time::sleep,
};
use tracing::instrument;

use super::{
    ActorInputMessage, DbInput,
    actor::{DefaultActor, PoolableActor},
};
use crate::{
    config::GetConfig,
    utils::{parse_duration, parse_time_delta},
};

#[derive(Clone, Debug)]
pub struct ActorPool {
    in_sender: Option<Sender<ActorInputMessage>>,
    rt: Arc<Runtime>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActorPoolConfig {
    pub clean_sessions: Duration,
    pub clean_links: Duration,
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
            stack_size: 2 * usize::pow(1024, 2),
            keep_alive: Duration::from_secs(10),
            event_interval: 61,
            clean_sessions: Duration::from_secs(15),
            clean_links: Duration::from_secs(1800),
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
            .unwrap_or(2 * usize::pow(1024, 2));
        let keep_alive: Duration = env::var("ACTOR_KEEP_ALIVE")
            .ok()
            .map(|s| {
                parse_duration(&s)
                    .expect("Unable to coerce ACTOR_KEEP_ALIVE into a duration string")
            })
            .unwrap_or(Duration::from_secs(10));
        let event_interval: u32 = env::var("ACTOR_EVENT_INTERVAL")
            .ok()
            .map(|s| {
                s.parse()
                    .expect("Unable to coerce ACTOR_EVENT_INTERVAL into an integer")
            })
            .unwrap_or(61);
        let clean_sessions = env::var("SESSION_CLEAN_INTERVAL")
            .ok()
            .map(|s| {
                parse_time_delta(&s)
                    .expect("Unable to coerce SESSION_CLEAN_INTERVAL to a duration string")
            })
            .unwrap_or(TimeDelta::seconds(10))
            .to_std()
            .expect("SESSION_CLEAN_INTERVAL is too large");
        let clean_links = env::var("SHORT_LINKS_CLEAN_INTERVAL")
            .ok()
            .map(|s| {
                parse_time_delta(&s)
                    .expect("Unable to coerce SHORT_LINKS_CLEAN_INTERVAL into a duration string")
            })
            .unwrap_or(TimeDelta::minutes(30))
            .to_std()
            .expect("SHORT_LINKS_CLEAN_INTERVAL is too large");
        Self {
            workers,
            blocking_workers,
            stack_size,
            keep_alive,
            event_interval,
            clean_sessions,
            clean_links,
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
            .unwrap_or(2 * usize::pow(1024, 2));
        let keep_alive: Duration = secrets
            .get("ACTOR_KEEP_ALIVE")
            .map(|s| parse_duration(&s).expect("Unable to coerce ACTOR_KEEP_ALIVE into a duration"))
            .unwrap_or(Duration::from_secs(10));
        let event_interval: u32 = secrets
            .get("ACTOR_EVENT_INTERVAL")
            .map(|s| {
                s.parse()
                    .expect("Unable to coerce ACTOR_EVENT_INTERVAL into an integer")
            })
            .unwrap_or(61);
        let clean_sessions = secrets
            .get("SESSION_CLEAN_INTERVAL")
            .map(|s| {
                parse_time_delta(&s)
                    .expect("Unable to coerce SESSION_CLEAN_INTERVAL to a duration string")
            })
            .unwrap_or(TimeDelta::seconds(10))
            .to_std()
            .expect("SESSION_CLEAN_INTERVAL is too large");
        let clean_links = secrets
            .get("SHORT_LINKS_CLEAN_INTERVAL")
            .map(|s| {
                parse_time_delta(&s)
                    .expect("Unable to coerce SHORT_LINKS_CLEAN_INTERVAL into a duration string")
            })
            .unwrap_or(TimeDelta::minutes(30))
            .to_std()
            .expect("SHORT_LINKS_CLEAN_INTERVAL is too large");
        Self {
            workers,
            blocking_workers,
            stack_size,
            keep_alive,
            event_interval,
            clean_sessions,
            clean_links,
        }
    }
}

async fn schedule_clean_sessions(
    in_sender: Sender<ActorInputMessage>,
    duration: Duration,
    conn: DatabaseConnection,
) -> Result<(), SendError<ActorInputMessage>> {
    loop {
        in_sender
            .send(ActorInputMessage::CleanSessions(DbInput {
                conn: conn.clone(),
            }))
            .await?;
        sleep(duration).await;
    }
}

async fn schedule_clean_links(
    in_sender: Sender<ActorInputMessage>,
    duration: Duration,
    conn: DatabaseConnection,
) -> Result<(), SendError<ActorInputMessage>> {
    loop {
        in_sender
            .send(ActorInputMessage::CleanUrls(DbInput { conn: conn.clone() }))
            .await?;
        sleep(duration).await;
    }
}

impl ActorPool {
    #[instrument]
    pub fn new(config: &ActorPoolConfig, conn: DatabaseConnection) -> Self {
        let num_chanels = (config.workers + config.blocking_workers + 2) * 2;
        let (in_sender, in_receiver) = bounded(num_chanels);
        let rt = Builder::new_multi_thread()
            .enable_all()
            .worker_threads(config.workers + 2)
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
        let duration = config.clean_sessions;
        let in_cleaner = in_sender.clone();
        let db_conn = conn.clone();
        rt.spawn(async move { schedule_clean_sessions(in_cleaner, duration, db_conn).await });
        let duration = config.clean_links;
        let in_cleaner = in_sender.clone();
        rt.spawn(async move { schedule_clean_links(in_cleaner, duration, conn).await });

        Self {
            in_sender: Some(in_sender),
            rt: Arc::new(rt),
        }
    }

    #[instrument]
    pub fn new_locked(config: &ActorPoolConfig, conn: DatabaseConnection) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self::new(config, conn)))
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
