use chrono::Utc;
use entity::{sessions, short_link, views};
#[cfg(feature = "ips")]
use sea_orm::prelude::IpNetwork;
use sea_orm::{entity::*, query::*};
use serde_json::json;
use tracing::{error, instrument, trace};

use super::{ActorError, ActorOutputMessage, DbInput, ViewInput};
#[cfg(feature = "headers")]
use crate::utils::HeaderMapDef;

#[instrument]
pub(super) async fn clean_urls(db: DbInput) -> Result<ActorOutputMessage, ActorError> {
    let DbInput { conn } = db;

    let txn = conn.begin().await?;

    let links = short_link::Entity::find()
        .filter(short_link::Column::ExpiryDate.lt(Utc::now().naive_utc()))
        .all(&txn)
        .await?;

    let count = links.len();

    for link in links {
        link.delete(&txn).await?;
    }

    match txn.commit().await {
        Ok(_) => (),
        Err(e) => {
            error!(
                "Transaction error when attempting to clean expired links: {}",
                e.to_string()
            );
            return Err(e.into());
        }
    };

    Ok(ActorOutputMessage {
        msg: format!("Short links were cleaned deleting {} urls", count),
    })
}

#[instrument]
pub(super) async fn clean_sessions(db: DbInput) -> Result<ActorOutputMessage, ActorError> {
    let DbInput { conn } = db;

    let txn = conn.begin().await?;

    let expired_sessions = sessions::Entity::find()
        .filter(sessions::Column::Expiry.lt(Utc::now().naive_utc()))
        .all(&txn)
        .await?;

    let count = expired_sessions.len();

    for session in expired_sessions {
        session.delete(&txn).await?;
    }

    match txn.commit().await {
        Ok(_) => (),
        Err(e) => {
            error!(
                "Transaction error when attempting to clean expired sessions: {}",
                e.to_string()
            );
            return Err(e.into());
        }
    };

    Ok(ActorOutputMessage {
        msg: format!("Sessions were cleaned deleting {} expired sessions", count),
    })
}

#[cfg(all(not(feature = "ips"), not(feature = "headers")))]
#[instrument]
pub(super) async fn update_views(msg: ViewInput) -> Result<ActorOutputMessage, ActorError> {
    let ViewInput { id, cached, conn } = msg;
    let view = views::ActiveModel {
        short_link: ActiveValue::Set(id.clone()),
        cache_hit: ActiveValue::Set(cached),
        created_at: ActiveValue::Set(Utc::now().naive_utc()),
        ..Default::default()
    };
    match view.insert(&conn).await {
        Ok(model) => {
            trace!("Views updated successfully for url {}: {:?}", id, model);
            Ok(ActorOutputMessage {
                msg: format!("Views updated successfully for url {}: {:?}", id, model),
            })
        }
        Err(e) => {
            error!("Error updating views for url {}: {}", id, e.to_string());
            Err(e.into())
        }
    }
}

#[cfg(all(feature = "ips", not(feature = "headers")))]
#[instrument]
pub(super) async fn update_views(msg: ViewInput) -> Result<ActorOutputMessage, ActorError> {
    let ViewInput {
        id,
        cached,
        ip,
        conn,
    } = msg;
    let view = views::ActiveModel {
        short_link: ActiveValue::Set(id.clone()),
        ip: ActiveValue::Set(IpNetwork::new(ip, 0).ok()),
        cache_hit: ActiveValue::Set(cached),
        created_at: ActiveValue::Set(Utc::now().naive_utc()),
        ..Default::default()
    };
    match view.insert(&conn).await {
        Ok(model) => {
            trace!("Views updated successfully for url {}: {:?}", id, model);
            Ok(ActorOutputMessage {
                msg: format!("Views updated successfully for url {}: {:?}", id, model),
            })
        }
        Err(e) => {
            error!("Error updating views for url {}: {}", id, e.to_string());
            Err(e.into())
        }
    }
}

#[cfg(all(feature = "headers", not(feature = "ips")))]
#[instrument]
pub(super) async fn update_views(msg: ViewInput) -> Result<ActorOutputMessage, ActorError> {
    let ViewInput {
        id,
        cached,
        headers,
        conn,
    } = msg;
    let headers: HeaderMapDef = match headers.try_into() {
        Ok(hm) => hm,
        Err(e) => {
            error!("Unable to serialize the headers: {}", e.to_string());
            return Err(e.into());
        }
    };
    let view = views::ActiveModel {
        short_link: ActiveValue::Set(id.clone()),
        headers: ActiveValue::Set(Some(json!(headers))),
        cache_hit: ActiveValue::Set(cached),
        created_at: ActiveValue::Set(Utc::now().naive_utc()),
        ..Default::default()
    };
    match view.insert(&conn).await {
        Ok(model) => {
            trace!("Views updated successfully for url {}: {:?}", id, model);
            Ok(ActorOutputMessage {
                msg: format!("Views updated successfully for url {}: {:?}", id, model),
            })
        }
        Err(e) => {
            error!("Error updating views for url {}: {}", id, e.to_string());
            Err(e.into())
        }
    }
}

#[cfg(all(feature = "headers", feature = "ips"))]
#[instrument]
pub(super) async fn update_views(msg: ViewInput) -> Result<ActorOutputMessage, ActorError> {
    let ViewInput {
        ip,
        id,
        cached,
        headers,
        conn,
    } = msg;
    let headers: HeaderMapDef = match headers.try_into() {
        Ok(hm) => hm,
        Err(e) => {
            error!("Unable to serialize the headers: {}", e.to_string());
            return Err(e.into());
        }
    };
    let view = views::ActiveModel {
        short_link: ActiveValue::Set(id.clone()),
        ip: ActiveValue::Set(IpNetwork::new(ip, 0).ok()),
        headers: ActiveValue::Set(Some(json!(headers))),
        cache_hit: ActiveValue::Set(cached),
        created_at: ActiveValue::Set(chrono::Utc::now().naive_utc()),
        ..Default::default()
    };
    match view.insert(&conn).await {
        Ok(model) => {
            trace!("Views updated successfully for url {}: {:?}", id, model);
            Ok(ActorOutputMessage {
                msg: format!("Views updated successfully for url {}: {:?}", id, model),
            })
        }
        Err(e) => {
            error!("Error updating views for url {}: {}", id, e.to_string());
            Err(e.into())
        }
    }
}
