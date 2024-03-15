use std::sync::Arc;
use once_cell::sync::Lazy;
use reqwest::{
  Client,
  header::USER_AGENT
};

static CUSTOM_USER_AGENT: Lazy<String> = Lazy::new(||
  format!("Kon/{}/Rust", super::utils::BOT_VERSION.as_str())
);

pub struct HttpClient {
  client: Arc<Client>
}

impl HttpClient {
  pub fn new() -> Self {
    Self {
      client: Arc::new(Client::new())
    }
  }

  pub async fn get(&self, url: &str) -> Result<reqwest::Response, reqwest::Error> {
    let req = self.client.get(url)
      .header(USER_AGENT, CUSTOM_USER_AGENT.as_str())
      .send()
      .await?;
    Ok(req)
  }
}
