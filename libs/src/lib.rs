mod config;
pub use config::BINARY_PROPERTIES;

mod types;
pub use types::*;

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
