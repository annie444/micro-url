use std::env;
#[cfg(feature = "ips")]
use std::str::FromStr;

#[cfg(feature = "ips")]
use axum_client_ip::ClientIpSource;
use openidconnect::{Scope, core::CoreClaimName};
use serde::{Deserialize, Serialize};
use shuttle_runtime::SecretStore;

#[cfg(feature = "ips")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::derivable_impls)]
pub struct ServerConfig {
    pub db: ServerDatabaseConfig,
    pub internal_url: String,
    pub external_url: String,
    pub addr: String,
    pub port: u16,
    pub scheme: String,
    pub assets_path: String,
    pub oidc: OidcConfig,
    pub ip_source: ClientIpSource,
}

#[cfg(not(feature = "ips"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::derivable_impls)]
pub struct ServerConfig {
    pub db: ServerDatabaseConfig,
    pub internal_url: String,
    pub external_url: String,
    pub addr: String,
    pub port: u16,
    pub scheme: String,
    pub assets_path: String,
    pub oidc: OidcConfig,
}

#[cfg(not(feature = "ips"))]
impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            db: ServerDatabaseConfig::default(),
            oidc: OidcConfig::default(),
            internal_url: "127.0.0.1:3000".to_string(),
            external_url: "https://example.com".to_string(),
            addr: "127.0.0.1".to_string(),
            port: 3000,
            scheme: "http".to_string(),
            assets_path: "../../js/frontend/dist".to_string(),
        }
    }
}

#[cfg(feature = "ips")]
impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            db: ServerDatabaseConfig::default(),
            oidc: OidcConfig::default(),
            internal_url: "127.0.0.1:3000".to_string(),
            external_url: "https://example.com".to_string(),
            addr: "127.0.0.1".to_string(),
            port: 3000,
            scheme: "http".to_string(),
            assets_path: "../../js/frontend/dist".to_string(),
            ip_source: ClientIpSource::RightmostXForwardedFor,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerDatabaseConfig {
    pub username: Option<String>,
    pub password: Option<String>,
    pub hostname: Option<String>,
    pub port: Option<String>,
    pub database: Option<String>,
    pub schema: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OidcConfig {
    pub name: String,
    pub client_id: String,
    pub client_secret: String,
    pub discovery_url: String,
    pub claims: Vec<CoreClaimName>,
    pub scopes: Vec<Scope>,
    pub cert_path: Option<String>,
}

#[cfg(feature = "ips")]
impl ServerConfig {
    #[tracing::instrument]
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        let db = ServerDatabaseConfig::from_env();
        let addr = get_env_var("ADDR").unwrap_or("127.0.0.1".to_string());
        let port: u16 = get_env_var("PORT")
            .unwrap_or("3000".to_string())
            .parse()
            .expect("PORT must be a number");
        let scheme = get_env_var("SCHEME").unwrap_or("http".to_string());
        let internal_url = get_env_var("INTERNAL_URL").unwrap_or(format!("{}:{}", addr, port));
        let external_url = get_env_var("EXTERNAL_URL")
            .unwrap_or_else(|| format!("{}://{}", &scheme, &internal_url));
        let assets_path =
            get_env_var("ASSETS_PATH").unwrap_or("../../js/frontend/dist".to_string());
        let oidc = OidcConfig::from_env();
        let ip_source: ClientIpSource = get_env_var("IP_SOURCE_HEADER")
            .map(|v| ClientIpSource::from_str(&v).expect("Unable to parse the IP_SOURCE_HEADER"))
            .unwrap_or(ClientIpSource::RightmostXForwardedFor);
        Self {
            db,
            internal_url,
            external_url,
            addr,
            port,
            scheme,
            assets_path,
            oidc,
            ip_source,
        }
    }

    #[tracing::instrument(skip(secrets))]
    pub fn from_secret(secrets: SecretStore) -> Self {
        let oidc = OidcConfig::from_secret(secrets.clone());
        let external_url = secrets
            .get("EXTERNAL_URL")
            .unwrap_or("http://localhost:8000".to_string());
        let assets_path = secrets
            .get("ASSETS_PATH")
            .unwrap_or("../../js/frontend/dist".to_string());
        let ip_source: ClientIpSource = secrets
            .get("IP_SOURCE_HEADER")
            .map(|v| ClientIpSource::from_str(&v).expect("Unable to parse the IP_SOURCE_HEADER"))
            .unwrap_or(ClientIpSource::RightmostXForwardedFor);
        Self {
            oidc,
            external_url,
            assets_path,
            ip_source,
            ..Self::default()
        }
    }
}

#[cfg(not(feature = "ips"))]
impl ServerConfig {
    #[tracing::instrument]
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        let db = ServerDatabaseConfig::from_env();
        let addr = get_env_var("ADDR").unwrap_or_else(|| "127.0.0.1".to_string());
        let port: u16 = get_env_var("PORT")
            .unwrap_or_else(|| "3000".to_string())
            .parse()
            .expect("PORT must be a number");
        let scheme = get_env_var("SCHEME").unwrap_or_else(|| "http".to_string());
        let internal_url =
            get_env_var("INTERNAL_URL").unwrap_or_else(|| format!("{}:{}", addr, port));
        let external_url = get_env_var("EXTERNAL_URL")
            .unwrap_or_else(|| format!("{}://{}", &scheme, &internal_url));
        let assets_path =
            get_env_var("ASSETS_PATH").unwrap_or_else(|| "../../js/frontend/dist".to_string());
        let oidc = OidcConfig::from_env();
        Self {
            db,
            internal_url,
            external_url,
            addr,
            port,
            scheme,
            assets_path,
            oidc,
        }
    }

    #[tracing::instrument(skip(secrets))]
    pub fn from_secret(secrets: SecretStore) -> Self {
        let oidc = OidcConfig::from_secret(secrets.clone());
        let external_url = secrets
            .get("EXTERNAL_URL")
            .unwrap_or_else(|| "http://localhost:8000".to_string());
        let assets_path = secrets
            .get("ASSETS_PATH")
            .unwrap_or_else(|| "../../js/frontend/dist".to_string());
        Self {
            oidc,
            external_url,
            assets_path,
            ..Self::default()
        }
    }
}

impl OidcConfig {
    #[tracing::instrument]
    pub fn from_env() -> Self {
        let name = get_env_var("OIDC_NAME").unwrap_or_else(|| "default".to_string());
        let client_id: String = get_env_var("OIDC_CLIENT_ID").expect("OIDC_CLIENT_ID is required");
        let client_secret: String =
            get_env_var("OIDC_CLIENT_SECRET").expect("OIDC_CLIENT_SECRET is required");
        let discovery_url: String =
            get_env_var("OIDC_DISCOVERY_URL").expect("OIDC_DISCOVERY_URL is required");
        let scopes: Vec<Scope> = get_env_var("OIDC_SCOPES")
            .unwrap_or_else(|| "openid email profile".to_string())
            .split_whitespace()
            .map(|s| Scope::new(s.to_string()))
            .collect();
        let claims: Vec<CoreClaimName> = get_env_var("OIDC_CLAIMS")
            .unwrap_or_else(|| {
                "sub aud email email_verified exp iat iss name given_name family_name \
                 preferred_username picture locale"
                    .to_string()
            })
            .split_whitespace()
            .map(|s| CoreClaimName::new(s.to_string()))
            .collect();
        let cert_path = get_env_var("OIDC_CERT_PATH");
        Self {
            name,
            client_id,
            client_secret,
            discovery_url,
            scopes,
            claims,
            cert_path,
        }
    }

    #[tracing::instrument(skip(secrets))]
    pub fn from_secret(secrets: SecretStore) -> Self {
        let name: String = secrets
            .get("OIDC_NAME")
            .unwrap_or_else(|| "default".to_string());
        let client_id: String = secrets
            .get("OIDC_CLIENT_ID")
            .expect("OIDC_CLIENT_ID is required");
        let client_secret: String = secrets
            .get("OIDC_CLIENT_SECRET")
            .expect("OIDC_CLIENT_SECRET is required");
        let discovery_url: String = secrets
            .get("OIDC_DISCOVERY_URL")
            .expect("OIDC_DISCOVERY_URL is required");
        let scopes: Vec<Scope> = secrets
            .get("OIDC_SCOPES")
            .unwrap_or_else(|| "openid email profile".to_string())
            .split_whitespace()
            .map(|s| Scope::new(s.to_string()))
            .collect();
        let claims: Vec<CoreClaimName> = secrets
            .get("OIDC_CLAIMS")
            .unwrap_or_else(|| {
                "sub aud email email_verified exp iat iss name given_name family_name \
                 preferred_username picture locale"
                    .to_string()
            })
            .split_whitespace()
            .map(|s| CoreClaimName::new(s.to_string()))
            .collect();
        let cert_path = secrets.get("OIDC_CERT_PATH");
        Self {
            name,
            client_id,
            client_secret,
            discovery_url,
            scopes,
            claims,
            cert_path,
        }
    }
}

impl ServerDatabaseConfig {
    #[tracing::instrument]
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        let username = get_env_var("DB_USER");
        let password = get_env_var("DB_PASS");
        let hostname = get_env_var("DB_HOST");
        let port = get_env_var("DB_PORT");
        let database = get_env_var("DB_NAME");
        let schema = get_env_var("DB_SCHEMA");
        Self {
            username,
            password,
            hostname,
            port,
            database,
            schema,
        }
    }

    #[tracing::instrument]
    pub fn connection_string(&self) -> String {
        let mut connection_str = String::from("postgres://");
        if let Some(username) = &self.username {
            connection_str.push_str(username);
            connection_str.push(':');
            if let Some(password) = &self.password {
                connection_str.push_str(password);
                connection_str.push('@');
            }
        }
        if let Some(hostname) = &self.hostname {
            connection_str.push_str(hostname);
        } else {
            connection_str.push_str("localhost");
        }
        connection_str.push(':');
        connection_str.push_str(self.port.as_deref().unwrap_or("5432"));
        connection_str.push('/');
        if let Some(database) = &self.database {
            connection_str.push_str(database);
        }
        if let Some(schema) = &self.schema {
            connection_str.push_str("?schema=");
            connection_str.push_str(schema);
        }
        connection_str
    }
}

#[tracing::instrument]
fn get_env_var(key: &str) -> Option<String> {
    env::var(key).ok()
}
