use tokenservice_client::{TokenService, TokenServiceApi};

pub struct TSClient {
  client: TokenService
}

impl TSClient {
  pub fn new() -> Self {
    let args: Vec<String> = std::env::args().collect();
    let service = if args.len() > 1 { args[1].as_str() } else { "kon" };
    TSClient {
      client: TokenService::new(service)
    }
  }
  pub async fn get(&self) -> Result<TokenServiceApi, Box<dyn std::error::Error>> {
    let api = self.client.connect().await.unwrap();
    Ok(api)
  }
}
