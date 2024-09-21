use std::sync::LazyLock;

pub struct ConfigMeta {
  pub embed_color: i32,
  pub ready_notify: u64,
  pub rss_channel: u64,
  pub kon_logs: u64,
  pub developers: Vec<u64>
}

#[cfg(feature = "production")]
pub static BINARY_PROPERTIES: LazyLock<ConfigMeta> = LazyLock::new(|| ConfigMeta::new());

#[cfg(not(feature = "production"))]
pub static BINARY_PROPERTIES: LazyLock<ConfigMeta> = LazyLock::new(||
  ConfigMeta::new()
    .embed_color(0xf1d63c)
    .ready_notify(865673694184996888)
    .rss_channel(865673694184996888)
);

impl ConfigMeta {
  fn new() -> Self {
    Self {
      embed_color: 0x5a99c7,
      ready_notify: 865673694184996888,
      rss_channel: 865673694184996888,
      kon_logs: 1268493237912604672,
      developers: vec![
        190407856527376384 // toast.ts
      ]
    }
  }

  // Scalable functions below;
  #[cfg(not(feature = "production"))]
  fn embed_color(mut self, color: i32) -> Self {
    self.embed_color = color;
    self
  }

  #[cfg(not(feature = "production"))]
  fn ready_notify(mut self, channel_id: u64) -> Self {
    self.ready_notify = channel_id;
    self
  }

  #[cfg(not(feature = "production"))]
  fn rss_channel(mut self, channel_id: u64) -> Self {
    self.rss_channel = channel_id;
    self
  }
}
