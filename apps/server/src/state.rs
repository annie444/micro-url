use super::config::ServerConfig;
use entity::short_link::Entity as ShortLink;
use lru::LruCache;
use migration::{Migrator, MigratorTrait};
use sea_orm::{entity::*, query::*};
use sea_orm::{Database, DatabaseConnection};
use std::num::NonZeroUsize;

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
    pub url: String,
    pub counter: usize,
}

#[cfg(not(feature = "test"))]
impl ServerState {
    pub async fn new(config: &ServerConfig) -> Self {
        let connection_str = config.db.connection_string();
        let conn = Database::connect(connection_str).await.unwrap();
        Migrator::up(&conn, None).await.unwrap();

        let mut counter: usize = 100000000000;

        let num_urls: u64 = ShortLink::find().count(&conn).await.unwrap();

        counter += num_urls as usize;

        let cache = LruCache::new(NonZeroUsize::new(1000).unwrap());

        let url = config.external_url.clone();

        Self {
            conn,
            cache,
            url,
            counter,
        }
    }

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
