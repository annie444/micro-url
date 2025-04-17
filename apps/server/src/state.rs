use std::{
    num::NonZeroUsize,
    sync::{Arc, Mutex, MutexGuard},
};

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use entity::short_link::Entity as ShortLink;
use lru::LruCache;
use migration::{Migrator, MigratorTrait};
use openidconnect::{
    ClientId, ClientSecret, IssuerUrl, RedirectUrl,
    core::{CoreClient, CoreProviderMetadata},
};
use reqwest::{ClientBuilder, redirect::Policy, tls::Certificate};
use sea_orm::{Database, DatabaseConnection, entity::*, query::*};
use url::Url;

use super::{config::ServerConfig, utils::OidcClient};
use crate::{actor::ActorPool, error::ArcMutexError};

pub const CHARS: [char; 64] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I',
    'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b',
    'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u',
    'v', 'w', 'x', 'y', 'z', '-', '_',
];

#[derive(FromRef, Debug, Clone)]
pub struct ServerState {
    pub conn: DatabaseConnection,
    cache: Arc<Mutex<LruCache<String, String>>>,
    pub url: Url,
    counter: Arc<Mutex<usize>>,
    pub config: ServerConfig,
    pub oidc_client: OidcClient,
    pub client: reqwest::Client,
    pub key: Key,
    pub pool: ActorPool,
}

impl ServerState {
    #[tracing::instrument]
    pub async fn new(config: ServerConfig) -> Self {
        let connection_str = config.db.connection_string();
        let conn = Database::connect(connection_str).await.unwrap();
        Migrator::up(&conn, None).await.unwrap();

        Self::_defaults(config, conn).await
    }

    #[tracing::instrument]
    pub async fn new_with_pool(config: ServerConfig, pool: sqlx::PgPool) -> Self {
        let conn = DatabaseConnection::from(pool);
        Migrator::up(&conn, None).await.unwrap();

        Self::_defaults(config, conn).await
    }

    #[tracing::instrument]
    pub async fn _defaults(config: ServerConfig, conn: DatabaseConnection) -> Self {
        let mut counter: usize = 100000000000;

        let num_urls: u64 = ShortLink::find().count(&conn).await.unwrap();

        counter += num_urls as usize;

        let counter = Arc::new(Mutex::new(counter));

        let cache = Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(1000).unwrap())));

        let url = Url::parse(&config.external_url).unwrap();

        let client = match config.oidc.cert_path.clone() {
            Some(cert_path) => {
                let cert_data = std::fs::read(cert_path).expect("Failed to read cert file");
                let cert = match Certificate::from_pem(&cert_data) {
                    Ok(cert) => vec![cert],
                    Err(_) => match Certificate::from_der(&cert_data) {
                        Ok(cert) => vec![cert],
                        Err(_) => {
                            Certificate::from_pem_bundle(&cert_data).expect("Invalid cert file")
                        }
                    },
                };
                let mut client = ClientBuilder::new().redirect(Policy::none());
                for c in cert {
                    client = client.add_root_certificate(c);
                }
                client.build().expect("Client should build")
            }
            None => ClientBuilder::new()
                .redirect(Policy::none())
                .build()
                .expect("Client should build"),
        };

        let provider_metadata = CoreProviderMetadata::discover_async(
            IssuerUrl::new(config.oidc.discovery_url.clone()).expect("Invalid issuer URL"),
            &client,
        )
        .await
        .expect("Failed to discover OIDC provider metadata");

        let oidc_client = CoreClient::from_provider_metadata(
            provider_metadata,
            ClientId::new(config.oidc.client_id.clone()),
            Some(ClientSecret::new(config.oidc.client_secret.clone())),
        )
        .set_redirect_uri(
            RedirectUrl::new(format!("{}/api/user/oidc/callback", config.external_url))
                .expect("Invalid redirect URL"),
        );

        let key = Key::generate();

        let pool = ActorPool::new(&config.actors, conn.clone());

        Self {
            conn,
            cache,
            url,
            counter,
            oidc_client,
            client,
            key,
            pool,
            config,
        }
    }

    #[tracing::instrument]
    pub fn put(&self, key: String, val: String) -> Result<Option<String>, ArcMutexError> {
        let mut cache: MutexGuard<LruCache<String, String>> =
            self.cache.lock().map_err(|e| ArcMutexError {
                error: format!(
                    "Unable to acquire lock on the mutex with key {} and value {}. Got error: {}",
                    key, val, e
                ),
            })?;
        Ok((*cache).put(key, val))
    }

    #[tracing::instrument]
    pub fn get(&self, key: &str) -> Result<Option<String>, ArcMutexError> {
        let mut cache: MutexGuard<LruCache<String, String>> =
            self.cache.lock().map_err(|e| ArcMutexError {
                error: format!(
                    "Unable to acquire lock on the mutex with key {}. Got error: {}",
                    key, e
                ),
            })?;
        Ok((*cache).get(key).cloned())
    }

    #[tracing::instrument]
    pub fn pop(&self, key: &str) -> Result<Option<(String, String)>, ArcMutexError> {
        let mut cache: MutexGuard<LruCache<String, String>> =
            self.cache.lock().map_err(|e| ArcMutexError {
                error: format!(
                    "Unable to acquire lock on the mutex with key {}. Got error: {}",
                    key, e
                ),
            })?;
        Ok((*cache).pop_entry(key))
    }

    #[tracing::instrument]
    pub fn increment(&self) -> Result<String, ArcMutexError> {
        let mut num: MutexGuard<usize> = self.counter.lock().map_err(|e| ArcMutexError {
            error: format!(
                "Unable to acquire lock on the mutex for the counter. Got error: {}",
                e
            ),
        })?;
        *num += 1;
        let mut num: usize = *num;
        let estimated_length = num.next_power_of_two().trailing_zeros().max(1);
        let mut b64 = String::with_capacity(estimated_length as usize);

        while num > 0 {
            // `num & 63`` is equivalent to `num % 64`,
            // and `num >> 6` is equivalent to `num / 64`.
            // Bitwise operations are usually faster, so we use them instead.
            b64.insert(0, CHARS[num & 63]);
            num >>= 6;
        }
        Ok(b64)
    }
}
