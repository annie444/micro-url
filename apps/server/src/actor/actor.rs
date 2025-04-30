use std::future::Future;

use async_channel::Receiver;
use tracing::{error, instrument, trace};

use super::{ActorInputMessage, ActorOutputMessage, tasks::*};

#[derive(Debug, Clone)]
pub(super) struct DefaultActor {
    in_receiver: Receiver<ActorInputMessage>,
}

pub trait PoolableActor: Clone + Send + Sync + std::fmt::Debug + 'static {
    fn set_channel(&mut self, in_receiver: Receiver<ActorInputMessage>) -> &mut Self;
    fn handle_message(&mut self, msg: ActorInputMessage) -> impl Future<Output = ()> + Send;
    fn run(&mut self) -> impl Future<Output = ()> + Send;
}

impl DefaultActor {
    #[instrument]
    pub(super) fn new(in_receiver: Receiver<ActorInputMessage>) -> Self {
        DefaultActor { in_receiver }
    }
}

impl PoolableActor for DefaultActor {
    #[instrument]
    fn set_channel(&mut self, in_receiver: Receiver<ActorInputMessage>) -> &mut Self {
        self.in_receiver = in_receiver;
        self
    }

    #[instrument]
    async fn handle_message(&mut self, msg: ActorInputMessage) {
        match async move {
            match msg {
                ActorInputMessage::CleanUrls(db) => clean_urls(db).await,
                ActorInputMessage::CleanSessions(db) => clean_sessions(db).await,
                ActorInputMessage::UpdateViews(view) => update_views(view).await,
                ActorInputMessage::None => {
                    {
                        async move {
                            Ok(ActorOutputMessage {
                                msg: "Ok".to_string(),
                            })
                        }
                    }
                    .await
                }
            }
        }
        .await
        {
            Ok(msg) => trace!("{}", msg.msg),
            Err(e) => error!("{}", e.to_string()),
        };
    }

    #[instrument]
    async fn run(&mut self) {
        while let Ok(msg) = self.in_receiver.recv().await {
            self.handle_message(msg).await;
        }
    }
}
