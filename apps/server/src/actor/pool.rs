use std::{env, ops::Drop, sync::Arc, time::Duration};

use async_channel::{SendError, Sender, bounded};
use chrono::TimeDelta;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tokio::{sync::RwLock, time::sleep};
use tracing::instrument;

use super::{
    ActorInputMessage, DbInput,
    actor::{DefaultActor, PoolableActor},
};
use crate::{config::GetConfig, utils::parse_time_delta};

#[derive(Clone, Debug)]
pub struct ActorPool {
    in_sender: Option<Sender<ActorInputMessage>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActorPoolConfig {
    pub clean_sessions: Duration,
    pub clean_links: Duration,
    pub workers: usize,
}

impl Default for ActorPoolConfig {
    fn default() -> Self {
        Self {
            workers: 4,
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
        let num_channels = (config.workers + 2) * 2;
        let (in_sender, in_receiver) = bounded(num_channels);
        for _ in 0..config.workers {
            let in_receiver = in_receiver.clone();
            tokio::spawn(async move { DefaultActor::new(in_receiver).run().await });
        }
        let duration = config.clean_sessions;
        let in_cleaner = in_sender.clone();
        let db_conn = conn.clone();
        tokio::spawn(async move { schedule_clean_sessions(in_cleaner, duration, db_conn).await });
        let duration = config.clean_links;
        let in_cleaner = in_sender.clone();
        tokio::spawn(async move { schedule_clean_links(in_cleaner, duration, conn).await });

        Self {
            in_sender: Some(in_sender),
        }
    }

    #[instrument]
    pub fn new_locked(config: &ActorPoolConfig, conn: DatabaseConnection) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self::new(config, conn)))
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
}

impl Drop for ActorPool {
    fn drop(&mut self) {
        self.in_sender.take();
    }
}
