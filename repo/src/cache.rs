use kon_tokens::token_path;

use {
  bb8_redis::{
    RedisConnectionManager,
    bb8::Pool,
    redis::{
      AsyncCommands,
      RedisError,
      RedisResult,
      cmd
    }
  },
  tokio::time::{
    Duration,
    sleep
  }
};

#[derive(Debug)]
pub struct RedisController {
  pool: Pool<RedisConnectionManager>
}

impl RedisController {
  pub async fn new() -> Result<Self, RedisError> {
    let manager = RedisConnectionManager::new(token_path().await.redis_uri.as_str())?;
    let pool = Self::create_pool(manager).await;
    Ok(Self { pool })
  }

  async fn create_pool(manager: RedisConnectionManager) -> Pool<RedisConnectionManager> {
    let mut backoff = 1;
    let redis_err = "Redis[Error]: {{ e }}, retrying in {{ backoff }} seconds";

    loop {
      match Pool::builder().max_size(20).retry_connection(true).build(manager.clone()).await {
        Ok(pool) => match pool.get().await {
          Ok(mut conn) => {
            let ping: RedisResult<String> = cmd("PING").query_async(&mut *conn).await;
            match ping {
              Ok(_) => {
                println!("Redis[Info]: Successfully connected");
                return pool.clone();
              },
              Err(e) => {
                eprintln!(
                  "{}",
                  redis_err
                    .replace("{{ e }}", &e.to_string())
                    .replace("{{ backoff }}", &backoff.to_string())
                );
                Self::apply_backoff(&mut backoff).await;
              }
            }
          },
          Err(e) => {
            eprintln!(
              "{}",
              redis_err
                .replace("{{ e }}", &e.to_string())
                .replace("{{ backoff }}", &backoff.to_string())
            );
            Self::apply_backoff(&mut backoff).await;
          }
        },
        Err(e) => {
          eprintln!("Redis[PoolError]: {e}, retrying in {backoff} seconds");
          Self::apply_backoff(&mut backoff).await;
        }
      }
    }
  }

  async fn apply_backoff(backoff: &mut u64) {
    sleep(Duration::from_secs(*backoff)).await;
    if *backoff < 64 {
      *backoff *= 2;
    }
  }

  /// Get a key from the cache
  pub async fn get(
    &self,
    key: &str
  ) -> RedisResult<Option<String>> {
    let mut conn = self.pool.get().await.unwrap();
    conn.get(key).await
  }

  pub async fn del(
    &self,
    key: &str
  ) -> RedisResult<()> {
    let mut conn = self.pool.get().await.unwrap();
    conn.del(key).await
  }

  /// Set a key with a value in the cache
  pub async fn set(
    &self,
    key: &str,
    value: &str
  ) -> RedisResult<()> {
    let mut conn = self.pool.get().await.unwrap();
    conn.set(key, value).await
  }

  /// Set a key with an expiration time in seconds
  pub async fn expire(
    &self,
    key: &str,
    seconds: i64
  ) -> RedisResult<()> {
    let mut conn = self.pool.get().await.unwrap();
    conn.expire(key, seconds).await
  }
}
