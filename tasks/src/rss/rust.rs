use super::{
  RSSFeed,
  RSSFeedOutput,
  fetch_feed,
  get_redis,
  parse,
  save_to_redis,
  task_err
};

use {
  kon_libs::KonResult,
  poise::serenity_prelude::{
    Context,
    async_trait
  },
  regex::Regex,
  std::{
    io::Cursor,
    sync::Arc
  }
};

pub struct RustBlog {
  url: String
}

impl RustBlog {
  pub fn new(url: String) -> Self { Self { url } }
}

#[async_trait]
impl RSSFeed for RustBlog {
  fn name(&self) -> &str { "RustBlog" }

  fn url(&self) -> &str { self.url.as_str() }

  async fn process(
    &self,
    _ctx: Arc<Context>
  ) -> KonResult<Option<RSSFeedOutput>> {
    let redis = get_redis().await;
    let rkey = "RSS_RustBlog";

    let res = fetch_feed(self.url()).await?;
    let data = res.text().await?;
    let cursor = Cursor::new(data);

    let feed = parse(cursor).map_err(|e| {
      task_err("RSS:RustBlog", &format!("Error parsing RSS feed: {e}"));
      e
    })?;

    if feed.entries.is_empty() {
      task_err("RSS:RustBlog", "No entries found in the feed!");
      return Ok(None);
    }

    let article = feed.entries[0].clone();
    let article_id = article.id.clone();

    fn get_blog_title(input: String) -> Option<String> {
      let re = Regex::new(r"https://blog\.rust-lang\.org/(\d{4}/\d{2}/\d{2}/[^/]+)").unwrap();
      re.captures(input.as_str()).and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
    }

    let cached_blog = redis.get(rkey).await.unwrap_or(None).unwrap_or_default();

    if cached_blog.is_empty() {
      save_to_redis(rkey, &get_blog_title(article.id).unwrap()).await?;
      return Ok(None);
    }

    if let Some(blog_title) = get_blog_title(article.id) {
      if blog_title == cached_blog {
        Ok(None)
      } else {
        save_to_redis(rkey, &blog_title).await?;

        Ok(Some(RSSFeedOutput::Content(format!(
          "Rust Team has put out a new article!\n**[{}](<{}>)**",
          article.links[0].title.clone().unwrap(),
          article.links[0].href
        ))))
      }
    } else {
      task_err(
        "RSS:RustBlog",
        &format!("Article URL does not match the expected RegEx pattern! ({article_id})")
      );
      Ok(None)
    }
  }
}
