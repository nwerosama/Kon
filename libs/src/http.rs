use {
  reqwest::{
    Client,
    Error,
    Response
  },
  std::time::Duration
};

const ERROR_PREFIX: &str = "HTTPClient[Error]";

pub struct HttpClient(Client);

impl Default for HttpClient {
  fn default() -> Self { Self::new() }
}

impl HttpClient {
  pub fn new() -> Self { Self(Client::new()) }

  pub async fn get(
    &self,
    url: &str,
    ua: &str
  ) -> Result<Response, Error> {
    let response = self
      .0
      .get(url)
      .header(
        reqwest::header::USER_AGENT,
        format!("Kon ({}-{}) - {ua}/reqwest", crate::BOT_VERSION.as_str(), crate::GIT_COMMIT_HASH)
      )
      .timeout(Duration::from_secs(30))
      .send()
      .await;

    match response {
      Ok(res) => Ok(res),
      Err(y) if y.is_timeout() => {
        eprintln!("{ERROR_PREFIX} Request timed out for \"{url}\"");
        Err(y)
      },
      Err(y) if y.is_connect() => {
        eprintln!("{ERROR_PREFIX} Connection failed for \"{url}\"");
        Err(y)
      },
      Err(y) => Err(y)
    }
  }
}
