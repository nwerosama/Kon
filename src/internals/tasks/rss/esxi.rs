use crate::Error;
use super::{
  super::task_err,
  REDIS_EXPIRY_SECS,
  get_redis,
  save_to_redis,
  fetch_feed,
  parse,
  format_href_to_discord
};

use std::io::Cursor;
use regex::Regex;
use poise::serenity_prelude::{
  CreateEmbed,
  CreateEmbedAuthor,
  Timestamp
};

pub async fn esxi_embed() -> Result<Option<CreateEmbed>, Error> {
  let redis = get_redis().await;
  let rkey = "RSS_ESXi";
  let url = "https://esxi-patches.v-front.de/atom/ESXi-7.0.0.xml";

  let res = fetch_feed(url).await?;
  let data = res.text().await?;
  let cursor = Cursor::new(data);

  let feed = parse(cursor).unwrap();
  let home_page = feed.links[0].clone().href;
  let article = feed.entries[0].clone();

  fn get_patch_version(input: &str) -> Option<String> {
    let re = Regex::new(r#"(?i)Update\s+([0-9]+)([a-z]?)"#).unwrap();

    if let Some(caps) = re.captures(input) {
      let update_num = caps[1].to_string();
      let letter = caps.get(2).map_or("", |m| m.as_str());
      Some(format!("Update {}{}", update_num, letter))
    } else {
      None
    }
  }

  let cached_patch = redis.get(&rkey).await.unwrap().unwrap_or_default();

  if cached_patch.is_empty() {
    redis.set(&rkey, &article.categories[3].term).await.unwrap();
    if let Err(y) = redis.expire(&rkey, REDIS_EXPIRY_SECS).await {
      task_err("RSS", format!("[RedisExpiry]: {}", y).as_str());
    }
    return Ok(None);
  }

  if let Some(patch) = get_patch_version(&article.categories[3].term) {
    if patch == cached_patch {
      return Ok(None);
    } else {
      save_to_redis(&rkey, &article.categories[3].term).await?;
      Ok(Some(CreateEmbed::new()
        .color(0x4EFBCB)
        .author(CreateEmbedAuthor::new(feed.title.unwrap().content).url(home_page))
        .thumbnail(feed.logo.unwrap().uri)
        .description(format!(
          "{} {} for {} {} has been rolled out!\n{}",
          article.categories[2].term,
          article.categories[3].term,
          article.categories[0].term,
          article.categories[1].term,
          format_href_to_discord(article.summary.unwrap().content.as_str())
        ))
        .timestamp(Timestamp::from(article.updated.unwrap())))
      )
    }
  } else {
    task_err("RSS:ESXi", &format!("Article term does not match the expected RegEx pattern! ({})", article.categories[3].term.as_str()));
    Ok(None)
  }
}
