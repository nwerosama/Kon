mod processor; // Process the feeds and send it off to Discord

mod esxi;
mod github;
mod gportal;
mod rust;

use super::{
  task_err,
  task_info
};

use {
  feed_rs::parser::parse,
  kon_libs::{
    HttpClient,
    KonResult
  },
  kon_repo::RedisController,
  once_cell::sync::OnceCell,
  poise::serenity_prelude::{
    Context,
    CreateEmbed,
    Timestamp
  },
  regex::Regex,
  reqwest::Response,
  std::sync::Arc,
  tokio::time::{
    Duration,
    interval
  }
};

const TASK_NAME: &str = "RSS";
static REDIS_EXPIRY_SECS: i64 = 7200;
static REDIS_SERVICE: OnceCell<Arc<RedisController>> = OnceCell::new();

async fn redis_() {
  let redis = RedisController::new().await.unwrap();
  REDIS_SERVICE.set(Arc::new(redis)).unwrap();
}

async fn get_redis() -> Arc<RedisController> {
  if REDIS_SERVICE.get().is_none() {
    redis_().await;
  }
  REDIS_SERVICE.get().unwrap().clone()
}

fn format_href_to_discord(input: &str) -> String {
  let re = Regex::new(r#"<a href="([^"]+)">([^<]+)</a>"#).unwrap();
  re.replace_all(input, r"[$2]($1)").to_string()
}

fn format_html_to_discord(input: String) -> String {
  let mut output = input;

  // Replace all instances of <p> and </p> with newlines
  output = Regex::new(r#"</?\s*p\s*>"#).unwrap().replace_all(&output, "\n").to_string();

  // Replace all instances of <br> and <br /> with newlines
  output = Regex::new(r#"<\s*br\s*/?\s*>"#).unwrap().replace_all(&output, "\n").to_string();

  // Replace all instances of <strong> with **
  output = Regex::new(r#"</?\s*strong\s*>"#).unwrap().replace_all(&output, "**").to_string();

  // Replace all instances of <var> and <small> with nothing
  output = Regex::new(r#"</?\s*(var|small)\s*>"#).unwrap().replace_all(&output, "").to_string();

  // Remove any other HTML tags
  output = Regex::new(r#"<[^>]+>"#).unwrap().replace_all(&output, "").to_string();

  // Replace all instances of <a href="url">text</a> with [text](url)
  output = format_href_to_discord(&output);

  output
}

async fn fetch_feed(url: &str) -> KonResult<Response> {
  let http = HttpClient::new();
  let res = match http.get(url, "RSS-Monitor").await {
    Ok(res) => res,
    Err(y) => return Err(y.into())
  };

  Ok(res)
}

async fn save_to_redis(
  key: &str,
  value: &str
) -> KonResult<()> {
  let redis = get_redis().await;
  redis.set(key, value).await.unwrap();
  if let Err(y) = redis.expire(key, REDIS_EXPIRY_SECS).await {
    task_err("RSS", format!("[RedisExpiry]: {}", y).as_str());
  }
  Ok(())
}

fn embed(
  color: u32,
  title: String,
  url: String,
  description: String,
  timestamp: Timestamp
) -> CreateEmbed {
  CreateEmbed::new()
    .color(color)
    .title(title)
    .url(url)
    .description(description)
    .timestamp(timestamp)
}

const MAX_CONTENT_LENGTH: usize = 4000;
fn trim_old_content(s: &str) -> String {
  if s.len() > MAX_CONTENT_LENGTH {
    s[..MAX_CONTENT_LENGTH].to_string()
  } else {
    s.to_string()
  }
}

enum IncidentColorMap {
  Update,
  Investigating,
  Monitoring,
  Resolved,
  Default
}

impl IncidentColorMap {
  fn color(&self) -> u32 {
    match self {
      Self::Update => 0xABDD9E,        // Madang
      Self::Investigating => 0xA5CCE0, // French Pass
      Self::Monitoring => 0x81CBAD,    // Monte Carlo
      Self::Resolved => 0x57F287,      // Emerald
      Self::Default => 0x81CBAD        // Monte Carlo
    }
  }
}

pub async fn rss(ctx: Arc<Context>) -> KonResult<()> {
  #[cfg(feature = "production")]
  let mut interval = interval(Duration::from_secs(300)); // Check feeds every 5 mins
  #[cfg(not(feature = "production"))]
  let mut interval = interval(Duration::from_secs(30)); // Check feeds every 30 secs
  let mut first_run = true;
  task_info(TASK_NAME, "Task loaded!");

  loop {
    interval.tick().await;

    if first_run {
      task_info(&format!("{TASK_NAME}:Processor"), "Starting up!");
      first_run = false;
    }
    processor::feed_processor(&ctx).await;
  }
}
