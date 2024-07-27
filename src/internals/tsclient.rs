use std::sync::LazyLock;
use tokio::sync::RwLock;
use tokenservice_client::{
  TokenService,
  TokenServiceApi
};

static TS_GLOBAL_CACHE: LazyLock<RwLock<Option<TokenServiceApi>>> = LazyLock::new(|| RwLock::new(None));

pub struct TSClient(TokenService);

impl TSClient {
  pub fn new() -> Self {
    let args: Vec<String> = std::env::args().collect();
    let service = if args.len() > 1 { &args[1] } else { "kon" };
    Self(TokenService::new(service))
  }

  pub async fn get(&self) -> Result<TokenServiceApi, crate::Error> {
    {
      let cache = TS_GLOBAL_CACHE.read().await;
      if let Some(ref api) = *cache {
        return Ok(api.clone());
      }
    }

    match self.0.connect().await {
      Ok(api) => {
        let mut cache = TS_GLOBAL_CACHE.write().await;
        *cache = Some(api.clone());
        Ok(api)
      }
      Err(e) => Err(e)
    }
  }
}
