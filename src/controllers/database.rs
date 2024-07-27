use crate::internals::utils::token_path;

use std::sync::LazyLock;
use bb8::{Pool, PooledConnection};
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::{
  Client,
  NoTls,
  Error,
  config::Config
};
use tokio::time::{
  sleep,
  Duration
};
use std::{
  ops::Deref,
  str::FromStr,
  sync::Mutex
};

pub static DATABASE: LazyLock<Mutex<Option<DatabaseController>>> = LazyLock::new(|| Mutex::new(None));

pub struct DatabaseController {
  pub pool: Pool<PostgresConnectionManager<NoTls>>
}

impl DatabaseController {
  pub async fn new() -> Result<(), Error> {
    let manager = PostgresConnectionManager::new(Config::from_str(token_path().await.postgres_uri.as_str())?, NoTls);
    let pool = Pool::builder().build(manager).await?;
    let err_name = "Postgres[Error]";

    let pool_clone = pool.clone();
    tokio::spawn(async move {
      let conn = match Self::attempt_connect(&pool_clone).await {
        Ok(conn) => {
          println!("Postgres[Info]: Successfully connected");
          conn
        },
        Err(y) => {
          eprintln!("{}: {}", err_name, y);
          return;
        }
      };
      let client: &Client = conn.deref();

      /* if let Err(e) = client.batch_execute("").await {
        eprintln!("{}: {}", err_name, e);
      } */ // Uncomment this if bot is going to use a database
    }).await.unwrap();

    let controller = Self { pool };
    *DATABASE.lock().unwrap() = Some(controller);

    Ok(())
  }

  async fn attempt_connect<'a>(pool: &'a Pool<PostgresConnectionManager<NoTls>>) -> Result<PooledConnection<'a, PostgresConnectionManager<NoTls>>, bb8::RunError<Error>> {
    let mut backoff = 1;

    loop {
      match pool.get().await {
        Ok(conn) => return Ok(conn),
        Err(y) => {
          eprintln!("Postgres[ConnError]: {}, retrying in {} seconds", y, backoff);
          sleep(Duration::from_secs(backoff)).await;
          if backoff < 64 {
            backoff *= 2;
          }
        }
      }
    }
  }
}
