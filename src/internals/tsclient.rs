use tokenservice_client::{
  TokenService,
  TokenServiceApi
};

pub struct TSClient(TokenService);

impl TSClient {
  pub fn new() -> Self {
    let args: Vec<String> = std::env::args().collect();
    let service = if args.len() > 1 { &args[1] } else { "kon" };
    Self(TokenService::new(service))
  }

  pub async fn get(&self) -> Result<TokenServiceApi, crate::Error> {
    match self.0.connect().await {
      Ok(api) => {
        Ok(api)
      }
      Err(e) => Err(e)
    }
  }
}
