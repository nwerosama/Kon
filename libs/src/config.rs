use std::sync::LazyLock;

pub struct ConfigMeta {
  pub env:          String,
  pub embed_color:  u32,
  pub ready_notify: u64,
  pub rss_channel:  u64,
  pub kon_logs:     u64,
  pub developers:   Vec<u64>
}

#[cfg(feature = "production")]
pub static BINARY_PROPERTIES: LazyLock<ConfigMeta> = LazyLock::new(ConfigMeta::new);

#[cfg(not(feature = "production"))]
pub static BINARY_PROPERTIES: LazyLock<ConfigMeta> = LazyLock::new(|| {
  ConfigMeta::new()
    .env("dev")
    .embed_color(0xF1D63C)
    .ready_notify(1311282815601741844)
    .rss_channel(1311282815601741844)
});

impl ConfigMeta {
  fn new() -> Self {
    Self {
      env:          "prod".to_string(),
      embed_color:  0x5A99C7,
      ready_notify: 1268493237912604672,
      rss_channel:  865673694184996888,
      kon_logs:     1268493237912604672,
      developers:   vec![
        190407856527376384, // nwero.sama
      ]
    }
  }

  // Scalable functions below;
  #[cfg(not(feature = "production"))]
  fn env(
    mut self,
    env: &str
  ) -> Self {
    self.env = env.to_string();
    self
  }

  #[cfg(not(feature = "production"))]
  fn embed_color(
    mut self,
    color: u32
  ) -> Self {
    self.embed_color = color;
    self
  }

  #[cfg(not(feature = "production"))]
  fn ready_notify(
    mut self,
    channel_id: u64
  ) -> Self {
    self.ready_notify = channel_id;
    self
  }

  #[cfg(not(feature = "production"))]
  fn rss_channel(
    mut self,
    channel_id: u64
  ) -> Self {
    self.rss_channel = channel_id;
    self
  }
}
