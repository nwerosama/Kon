use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use tokenservice_client::TokenServiceApi;
use super::tsclient::TSClient;

pub static BOT_VERSION: Lazy<String> = Lazy::new(|| {
  let cargo_version = cargo_toml::Manifest::from_str(&include_str!("../../Cargo.toml"))
    .unwrap()
    .package
    .unwrap()
    .version
    .unwrap();
  format!("v{}", cargo_version)
});

static TSCLIENT: Lazy<Mutex<TSClient>> = Lazy::new(|| Mutex::new(TSClient::new()));

pub async fn token_path() -> TokenServiceApi {
  TSCLIENT.lock().await.get().await.unwrap()
}

pub fn concat_message(messages: Vec<String>) -> String {
  messages.join("\n")
}

pub fn format_duration(secs: u64) -> String {
  let days = secs / 86400;
  let hours = (secs % 86400) / 3600;
  let minutes = (secs % 3600) / 60;
  let seconds = secs % 60;

  let mut formatted_string = String::new();
  if days > 0 {
    formatted_string.push_str(&format!("{}d, ", days));
  }
  if hours > 0 || days > 0 {
    formatted_string.push_str(&format!("{}h, ", hours));
  }
  if minutes > 0 || hours > 0 {
    formatted_string.push_str(&format!("{}m, ", minutes));
  }
  formatted_string.push_str(&format!("{}s", seconds));

  formatted_string
}
