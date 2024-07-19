use std::time::Duration;
use reqwest::{
  Client,
  Response,
  Error
};

pub struct HttpClient(Client);

impl HttpClient {
  pub fn new() -> Self {
    Self(Client::new())
  }

  pub async fn get(&self, url: &str, ua: &str) -> Result<Response, Error> {
    Ok(
      self.0.get(url).header(
        reqwest::header::USER_AGENT,
        format!("Kon ({}) - {}/reqwest", super::utils::BOT_VERSION.as_str(), ua)
      )
      .timeout(Duration::from_secs(15))
      .send()
      .await?
    )
  }
}
