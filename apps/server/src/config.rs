use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub db: ServerDatabaseConfig,
    pub internal_url: String,
    pub external_url: String,
    pub addr: String,
    pub port: u16,
    pub scheme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerDatabaseConfig {
    pub username: Option<String>,
    pub password: Option<String>,
    pub hostname: Option<String>,
    pub port: Option<String>,
    pub database: Option<String>,
    pub schema: Option<String>,
}

impl ServerConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        let db = ServerDatabaseConfig::from_env();
        let addr = get_env_var("ADDR").unwrap_or_else(|| "127.0.0.1".to_string());
        let port: u16 = get_env_var("PORT")
            .unwrap_or_else(|| "3000".to_string())
            .parse()
            .unwrap();
        let scheme = get_env_var("SCHEME").unwrap_or_else(|| "http".to_string());
        let internal_url = get_env_var("INTERNAL_URL")
            .unwrap_or_else(|| format!("{}://{}:{}", scheme, addr, port));
        let external_url = get_env_var("EXTERNAL_URL").unwrap_or_else(|| internal_url.clone());
        Self {
            db,
            internal_url,
            external_url,
            addr,
            port,
            scheme,
        }
    }
}

impl ServerDatabaseConfig {
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

    pub fn connection_string(&self) -> String {
        let mut connection_str = String::from("postgres://");
        if let Some(username) = &self.username {
            connection_str.push_str(username);
            connection_str.push_str(":");
            if let Some(password) = &self.password {
                connection_str.push_str(password);
                connection_str.push_str("@");
            }
        }
        if let Some(hostname) = &self.hostname {
            connection_str.push_str(hostname);
        } else {
            connection_str.push_str("localhost");
        }
        connection_str.push_str(":");
        connection_str.push_str(self.port.as_deref().unwrap_or("5432"));
        connection_str.push_str("/");
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

fn get_env_var(key: &str) -> Option<String> {
    match env::var(key) {
        Ok(val) => Some(val),
        Err(_) => None,
    }
}
