use super::{
  REDIS_EXPIRY_SECS,
  fetch_feed,
  get_redis,
  parse,
  save_to_redis,
  task_err
};

use {
  kon_libs::KonResult,
  regex::Regex,
  std::io::Cursor
};

pub async fn rust_message() -> KonResult<Option<String>> {
  let redis = get_redis().await;
  let rkey = "RSS_RustBlog";
  let url = "https://blog.rust-lang.org/feed.xml";

  let res = fetch_feed(url).await?;
  let data = res.text().await?;
  let cursor = Cursor::new(data);

  let feed = parse(cursor).unwrap();
  let article = feed.entries[0].clone();
  let article_id = article.id.clone();

  fn get_blog_title(input: String) -> Option<String> {
    let re = Regex::new(r"https://blog\.rust-lang\.org/(\d{4}/\d{2}/\d{2}/[^/]+)").unwrap();
    re.captures(input.as_str()).and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
  }

  let cached_blog = redis.get(rkey).await.unwrap().unwrap_or_default();

  if cached_blog.is_empty() {
    redis.set(rkey, get_blog_title(article.id).unwrap().as_str()).await.unwrap();
    if let Err(y) = redis.expire(rkey, REDIS_EXPIRY_SECS).await {
      task_err("RSS", format!("[RedisExpiry]: {}", y).as_str());
    }
    return Ok(None);
  }

  if let Some(blog) = get_blog_title(article.id) {
    if blog == cached_blog {
      Ok(None)
    } else {
      save_to_redis(rkey, &blog).await?;
      Ok(Some(format!(
        "Rust Team has put out a new article!\n**[{}](<{}>)**",
        article.links[0].title.clone().unwrap(),
        article.links[0].href
      )))
    }
  } else {
    task_err(
      "RSS:RustBlog",
      &format!("Article URL does not match the expected RegEx pattern! ({})", article_id)
    );
    Ok(None)
  }
}
