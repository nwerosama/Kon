use {
  super::tsclient::TSClient,
  poise::serenity_prelude::UserId,
  std::sync::LazyLock,
  tokenservice_client::TokenServiceApi,
  tokio::sync::Mutex
};

pub static BOT_VERSION: LazyLock<String> = LazyLock::new(|| {
  let cargo_version = cargo_toml::Manifest::from_str(include_str!("../../Cargo.toml"))
    .unwrap()
    .package
    .unwrap()
    .version
    .unwrap();
  format!("v{cargo_version}")
});

static TSCLIENT: LazyLock<Mutex<TSClient>> = LazyLock::new(|| Mutex::new(TSClient::new()));

pub async fn token_path() -> TokenServiceApi { TSCLIENT.lock().await.get().await.unwrap() }

pub fn mention_dev(ctx: poise::Context<'_, (), crate::Error>) -> Option<String> {
  let devs = super::config::BINARY_PROPERTIES.developers.clone();
  let app_owners = ctx.framework().options().owners.clone();

  let mut mentions = Vec::new();

  for dev in devs {
    if app_owners.contains(&UserId::new(dev)) {
      mentions.push(format!("<@{dev}>"));
    }
  }

  if mentions.is_empty() { None } else { Some(mentions.join(", ")) }
}

pub fn format_duration(secs: u64) -> String {
  let days = secs / 86400;
  let hours = (secs % 86400) / 3600;
  let minutes = (secs % 3600) / 60;
  let seconds = secs % 60;

  let mut formatted_string = String::new();
  if days > 0 {
    formatted_string.push_str(&format!("{days}d, "));
  }
  if hours > 0 || days > 0 {
    formatted_string.push_str(&format!("{hours}h, "));
  }
  if minutes > 0 || hours > 0 {
    formatted_string.push_str(&format!("{minutes}m, "));
  }
  formatted_string.push_str(&format!("{seconds}s"));

  formatted_string
}

pub fn format_bytes(bytes: u64) -> String {
  let units = ["B", "KB", "MB", "GB", "TB", "PB"];
  let mut value = bytes as f64;
  let mut unit = units[0];

  for &u in &units[1..] {
    if value < 1024.0 {
      break;
    }

    value /= 1024.0;
    unit = u;
  }

  if unit == "B" {
    format!("{value}{unit}")
  } else {
    format!("{value:.2}{unit}")
  }
}
