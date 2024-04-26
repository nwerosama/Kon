use once_cell::sync::Lazy;
use tokenservice_client::TokenServiceApi;

pub static EMBED_COLOR: i32 = 0x5a99c7;

pub static BOT_VERSION: Lazy<String> = Lazy::new(|| {
  let cargo_version = cargo_toml::Manifest::from_path(std::env::var("CARGO_MANIFEST_DIR").unwrap()+"/Cargo.toml")
    .unwrap()
    .package
    .unwrap()
    .version
    .unwrap();
  format!("v{}", cargo_version)
});

pub async fn token_path() -> TokenServiceApi {
  let client = super::tsclient::TSClient::new().get().await.unwrap();
  client
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
