use std::num::NonZeroUsize;

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use entity::short_link::Entity as ShortLink;
use lru::LruCache;
use migration::{Migrator, MigratorTrait};
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata},
    ClientId, ClientSecret, IssuerUrl, RedirectUrl,
};
use reqwest::{redirect::Policy, tls::Certificate, ClientBuilder};
use sea_orm::{entity::*, query::*, Database, DatabaseConnection};
use url::Url;

use super::{
    config::{OidcConfig, ServerConfig},
    structs::OidcClient,
};

pub const CHARS: [char; 64] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I',
    'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b',
    'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u',
    'v', 'w', 'x', 'y', 'z', '-', '_',
];

#[cfg(not(feature = "test"))]
#[derive(Debug, Clone)]
pub struct ServerState {
    pub conn: DatabaseConnection,
    pub cache: LruCache<String, String>,
    pub url: Url,
    pub counter: usize,
    pub oidc_config: OidcConfig,
    pub oidc_client: OidcClient,
    pub client: reqwest::Client,
    pub key: Key,
}

impl FromRef<ServerState> for Key {
    fn from_ref(state: &ServerState) -> Self {
        state.key.clone()
    }
}

#[cfg(not(feature = "test"))]
impl ServerState {
    #[tracing::instrument]
    pub async fn new(config: &ServerConfig) -> Self {
        let connection_str = config.db.connection_string();
        let conn = Database::connect(connection_str).await.unwrap();
        Migrator::up(&conn, None).await.unwrap();

        Self::_defaults(config, conn).await
    }

    #[tracing::instrument]
    pub async fn new_with_pool(config: &ServerConfig, pool: sqlx::PgPool) -> Self {
        let conn = DatabaseConnection::from(pool);
        Migrator::up(&conn, None).await.unwrap();

        Self::_defaults(config, conn).await
    }

    #[tracing::instrument]
    pub async fn _defaults(config: &ServerConfig, conn: DatabaseConnection) -> Self {
        let mut counter: usize = 100000000000;

        let num_urls: u64 = ShortLink::find().count(&conn).await.unwrap();

        counter += num_urls as usize;

        let cache = LruCache::new(NonZeroUsize::new(1000).unwrap());

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
            RedirectUrl::new(format!("{}/auth/callback", config.external_url))
                .expect("Invalid redirect URL"),
        );

        let oidc_config = config.oidc.clone();

        let key = Key::generate();

        Self {
            conn,
            cache,
            url,
            counter,
            oidc_config,
            oidc_client,
            client,
            key,
        }
    }

    #[tracing::instrument]
    pub fn increment(&mut self) -> String {
        self.counter += 1;
        let mut num = self.counter.clone();
        let estimated_length = num.next_power_of_two().trailing_zeros().max(1);
        let mut b64 = String::with_capacity(estimated_length as usize);

        while num > 0 {
            // `num & 63`` is equivalent to `num % 64`,
            // and `num >> 6` is equivalent to `num / 64`.
            // Bitwise operations are usually faster, so we use them instead.
            b64.insert(0, CHARS[num & 63]);
            num >>= 6;
        }
        b64
    }
}
