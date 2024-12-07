use super::{
  RSSFeed,
  RSSFeedOutput,
  fetch_feed,
  format_href_to_discord,
  get_redis,
  parse,
  save_to_redis,
  task_err
};

use {
  kon_libs::KonResult,
  poise::serenity_prelude::{
    Context,
    CreateEmbed,
    CreateEmbedAuthor,
    Timestamp,
    async_trait
  },
  regex::Regex,
  std::{
    io::Cursor,
    sync::Arc
  }
};

pub struct Esxi {
  url: String
}

impl Esxi {
  pub fn new(url: String) -> Self { Self { url } }
}

#[async_trait]
impl RSSFeed for Esxi {
  fn name(&self) -> &str { "ESXi" }

  fn url(&self) -> &str { self.url.as_str() }

  async fn process(
    &self,
    _ctx: Arc<Context>
  ) -> KonResult<Option<RSSFeedOutput>> {
    let redis = get_redis().await;
    let rkey = "RSS_ESXi";

    let res = fetch_feed(self.url()).await?;
    let data = res.text().await?;
    let cursor = Cursor::new(data);

    let feed = parse(cursor).map_err(|e| {
      task_err("RSS:ESXi", &format!("Error parsing RSS feed: {e}"));
      e
    })?;

    if feed.entries.is_empty() {
      task_err("RSS:ESXi", "No entries found in the feed!");
      return Ok(None);
    }

    let home_page = feed.links[0].clone().href;
    let article = feed.entries[0].clone();

    fn get_patch_version(input: &str) -> Option<String> {
      let re = Regex::new(r#"(?i)Update\s+([0-9]+)([a-z]?)"#).unwrap();

      if let Some(caps) = re.captures(input) {
        let update_num = caps[1].to_string();
        let letter = caps.get(2).map_or("", |m| m.as_str());
        Some(format!("Update {update_num}{letter}"))
      } else {
        None
      }
    }

    let cached_patch = redis.get(rkey).await.unwrap_or(None).unwrap_or_default();

    if cached_patch.is_empty() {
      save_to_redis(rkey, &article.categories[3].term).await?;
      return Ok(None);
    }

    if let Some(patch) = get_patch_version(&article.categories[3].term) {
      if patch == cached_patch {
        Ok(None)
      } else {
        save_to_redis(rkey, &article.categories[3].term).await?;

        Ok(Some(RSSFeedOutput::RegularEmbed(
          CreateEmbed::new()
            .color(0x4EFBCB)
            .author(CreateEmbedAuthor::new(feed.title.unwrap().content).url(home_page))
            .thumbnail(feed.logo.unwrap().uri)
            .description(format!(
              "{} {} for {} {} has been rolled out!\n{}",
              article.categories[2].term,
              article.categories[3].term,
              article.categories[0].term,
              article.categories[1].term,
              format_href_to_discord(&article.summary.unwrap().content)
            ))
            .timestamp(Timestamp::from(article.updated.unwrap()))
        )))
      }
    } else {
      task_err(
        "RSS:ESXi",
        &format!("Article term does not match the expected RegEx pattern! ({})", article.categories[3].term)
      );
      Ok(None)
    }
  }
}
