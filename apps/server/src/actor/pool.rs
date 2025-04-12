use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use async_channel::{SendError, Sender, bounded};
use serde::{Deserialize, Serialize};
use tokio::runtime::{Builder, Handle};
use tracing::instrument;

use super::{
    actor::{DefaultActor, PoolableActor},
    msgs::ActorInputMessage,
};

#[derive(Clone, Debug)]
pub struct ActorPool {
    in_sender: Option<Sender<ActorInputMessage>>,
    handle: Handle,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActorPoolConfig {
    pub cleaners: usize,
    pub workers: usize,
    pub blocking_workers: usize,
    pub stack_size: usize,
    pub keep_alive: Duration,
    pub event_interval: u32,
}

impl ActorPool {
    #[instrument(skip(actors))]
    pub fn new(config: ActorPoolConfig, actors: Vec<impl PoolableActor>) -> Self {
        let num_chanels = (config.workers + config.blocking_workers + config.cleaners) * 2;
        let (in_sender, in_receiver) = bounded(num_chanels);
        let rt = Builder::new_multi_thread()
            .enable_all()
            .worker_threads(config.workers + config.cleaners)
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
        for _ in 0..config.cleaners {
            let in_receiver = in_receiver.clone();
            rt.spawn(async move { DefaultActor::new(in_receiver).run().await });
        }
        for mut actor in actors {
            let in_receiver = in_receiver.clone();
            rt.spawn(async move { actor.set_channel(in_receiver).run().await });
        }

        Self {
            in_sender: Some(in_sender),
            handle: rt.handle().to_owned(),
        }
    }

    #[instrument]
    pub fn handle(&self) -> &Handle {
        &self.handle
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
        self.in_sender.take();
    }
}
