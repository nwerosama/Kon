mod config;
pub use config::BINARY_PROPERTIES;

mod types;
pub use types::*;

mod data;
pub use data::KonData;

mod http;
pub use http::HttpClient;

use {
  cargo_toml::Manifest,
  poise::serenity_prelude::UserId,
  std::sync::LazyLock
};

#[cfg(feature = "production")]
pub static GIT_COMMIT_HASH: &str = env!("GIT_COMMIT_HASH");
pub static GIT_COMMIT_BRANCH: &str = env!("GIT_COMMIT_BRANCH");

#[cfg(not(feature = "production"))]
pub static GIT_COMMIT_HASH: &str = "devel";

pub static BOT_VERSION: LazyLock<String> = LazyLock::new(|| {
  Manifest::from_str(include_str!("../../Cargo.toml"))
    .unwrap()
    .package
    .unwrap()
    .version
    .unwrap()
});

pub fn mention_dev(ctx: PoiseCtx<'_>) -> Option<String> {
  let devs = BINARY_PROPERTIES.developers.clone();
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
  let units = ["B", "KB", "MB", "GB"];
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
