use {
  std::{
    env::args,
    error::Error,
    sync::LazyLock
  },
  tokenservice_client::{
    TokenService,
    TokenServiceApi
  },
  tokio::sync::Mutex
};

static TSCLIENT: LazyLock<Mutex<TSClient>> = LazyLock::new(|| Mutex::new(TSClient::default()));

pub struct TSClient(TokenService);

impl Default for TSClient {
  fn default() -> Self { Self::new() }
}

impl TSClient {
  pub fn new() -> Self {
    let args: Vec<String> = args().collect();
    let service = if args.len() > 1 { &args[1] } else { "kon" };
    Self(TokenService::new(service))
  }

  pub async fn get(&self) -> Result<TokenServiceApi, Box<dyn Error + Send + Sync>> {
    match self.0.connect().await {
      Ok(a) => Ok(a),
      Err(e) => Err(e)
    }
  }
}

pub async fn token_path() -> TokenServiceApi {
  match TSCLIENT.lock().await.get().await {
    Ok(a) => a,
    Err(e) => panic!("TSClient[Error]: {e}")
  }
}
