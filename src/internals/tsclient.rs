use tokenservice_client::{TokenService, TokenServiceApi};

pub struct TSClient {
  client: TokenService
}

impl TSClient {
  pub fn new() -> Self {
    TSClient {
      client: TokenService::new("kon")
    }
  }
  pub async fn get(&self) -> Result<TokenServiceApi, Box<dyn std::error::Error>> {
    let api = self.client.connect().await.unwrap();
    Ok(api)
  }
}
