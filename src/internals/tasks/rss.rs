use crate::{
  Error,
  controllers::cache::RedisController
};
use super::{
  super::{
    http::HttpClient,
    config::BINARY_PROPERTIES
  },
  task_info,
  task_err
};

use once_cell::sync::OnceCell;
use feed_rs::parser::parse;
use reqwest::Response;
use regex::Regex;
use std::{
  sync::Arc,
  io::Cursor
};
use poise::serenity_prelude::{
  Context,
  ChannelId,
  EditMessage,
  CreateMessage,
  CreateEmbed,
  CreateEmbedAuthor,
  Timestamp
};
use tokio::time::{
  Duration,
  interval
};

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

  // Moved up here as a copy-paste

  //  This is for building up the embed with the feed data
  // std::fs::File::create("rss_name.log").unwrap();
  // std::fs::write("rss_name.log", format!("{:#?}", feed))?;

fn format_href_to_discord(input: &str) -> String {
  let re = Regex::new(r#"<a href="([^"]+)">([^<]+)</a>"#).unwrap();
  re.replace_all(input, r"[$2]($1)").to_string()
}

fn format_html_to_discord(input: String) -> String {
  let mut output = input;

  // Replace all instances of <p> with newlines
  output = Regex::new(r#"<\s*p\s*>"#).unwrap().replace_all(&output, "\n").to_string();
  output = Regex::new(r#"<\s*/\s*p\s*>"#).unwrap().replace_all(&output, "\n").to_string();

  // Replace all instances of <br> and <br /> with newlines
  output = Regex::new(r#"<\s*br\s*>"#).unwrap().replace_all(&output, "\n").to_string();
  output = Regex::new(r#"<\s*br\s*/\s*>"#).unwrap().replace_all(&output, "\n").to_string();

  // Replace all instances of <strong> with **
  output = Regex::new(r#"<\s*strong\s*>"#).unwrap().replace_all(&output, "**").to_string();
  output = Regex::new(r#"<\s*/\s*strong\s*>"#).unwrap().replace_all(&output, "**").to_string();

  // Replace all instances of <var> and <small> with nothing
  output = Regex::new(r#"<\s*var\s*>"#).unwrap().replace_all(&output, "").to_string();
  output = Regex::new(r#"<\s*/\s*var\s*>"#).unwrap().replace_all(&output, "").to_string();
  output = Regex::new(r#"<\s*small\s*>"#).unwrap().replace_all(&output, "").to_string();
  output = Regex::new(r#"<\s*/\s*small\s*>"#).unwrap().replace_all(&output, "").to_string();

  // Remove any other HTML tags
  output = Regex::new(r#"<[^>]+>"#).unwrap().replace_all(&output, "").to_string();

  // Replace all instances of <a href="url">text</a> with [text](url)
  output = format_href_to_discord(&output);

  output
}

async fn fetch_feed(url: &str) -> Result<Response, Error> {
  let http = HttpClient::new();
  let res = match http.get(url, "RSS-Monitor").await {
    Ok(res) => res,
    Err(y) => return Err(y.into())
  };

  Ok(res)
}

async fn save_to_redis(key: &str, value: &str) -> Result<(), Error> {
  let redis = get_redis().await;
  redis.set(key, value).await.unwrap();
  if let Err(y) = redis.expire(key, REDIS_EXPIRY_SECS).await {
    task_err("RSS", format!("[RedisExpiry]: {}", y).as_str());
  }
  Ok(())
}

async fn esxi_embed() -> Result<Option<CreateEmbed>, Error> {
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

async fn gportal_embed() -> Result<Option<CreateEmbed>, Error> {
  let redis = get_redis().await;
  let rkey = "RSS_GPortal";
  let rkey_content = format!("{}_Content", rkey);
  let url = "https://status.g-portal.com/history.atom";

  let res = fetch_feed(url).await?;
  let data = res.text().await?;
  let cursor = Cursor::new(data);

  let feed = parse(cursor).unwrap();
  let incident_page = feed.links[0].clone().href;
  let article = feed.entries[0].clone();

  fn get_incident_id(input: &str) -> Option<String> {
    let re = Regex::new(r#"/incidents/([a-zA-Z0-9]+)$"#).unwrap();

    if let Some(caps) = re.captures(input) {
      Some(caps[1].to_string())
    } else {
      None
    }
  }

  let cached_incident = redis.get(&rkey).await.unwrap().unwrap_or_default();
  let new_content = format_html_to_discord(article.content.unwrap().body.unwrap());

  if cached_incident.is_empty() {
    redis.set(&rkey, &get_incident_id(&article.links[0].href).unwrap()).await.unwrap();
    redis.set(&rkey_content, &new_content).await.unwrap();
    if let Err(y) = redis.expire(&rkey, REDIS_EXPIRY_SECS).await {
      task_err("RSS", format!("[RedisExpiry]: {}", y).as_str());
    }
    return Ok(None);
  }

  if let Some(incident) = get_incident_id(&article.links[0].href) {
    if incident == cached_incident {
      let cached_content: String = redis.get(&format!("{}_content", rkey)).await.unwrap().unwrap_or_default();
      if cached_content == new_content {
        return Ok(None);
      } else {
        redis.set(&rkey_content, &new_content).await.unwrap();
        redis.expire(&rkey_content, 21600).await.unwrap();
        return Ok(Some(CreateEmbed::new()
          .color(0xC23EE8)
          .title(article.title.unwrap().content)
          .url(incident_page)
          .description(new_content)
          .timestamp(Timestamp::from(article.updated.unwrap()))
        ));
      }
    } else {
      save_to_redis(&rkey, &incident).await?;
      redis.set(&rkey_content, &new_content).await.unwrap();
      return Ok(Some(CreateEmbed::new()
        .color(0xC23EE8)
        .title(article.title.unwrap().content)
        .url(incident_page)
        .description(new_content)
        .timestamp(Timestamp::from(article.updated.unwrap()))
      ));
    }
  } else {
    task_err("RSS:GPortal", &format!("Incident ID does not match the expected RegEx pattern! ({})", &article.links[0].href));
    Ok(None)
  }
}

async fn rust_message() -> Result<Option<String>, Error> {
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

  let cached_blog = redis.get(&rkey).await.unwrap().unwrap_or_default();

  if cached_blog.is_empty() {
    redis.set(&rkey, get_blog_title(article.id).unwrap().as_str()).await.unwrap();
    if let Err(y) = redis.expire(&rkey, REDIS_EXPIRY_SECS).await {
      task_err("RSS", format!("[RedisExpiry]: {}", y).as_str());
    }
    return Ok(None);
  }

  if let Some(blog) = get_blog_title(article.id) {
    if blog == cached_blog {
      return Ok(None);
    } else {
      save_to_redis(&rkey, &blog).await?;
      Ok(Some(format!("Rust Team has put out a new article!\n**[{}](<{}>)**", article.links[0].title.clone().unwrap(), article.links[0].href)))
    }
  } else {
    task_err("RSS:RustBlog", &format!("Article URL does not match the expected RegEx pattern! ({})", article_id));
    Ok(None)
  }
}

pub async fn rss(ctx: Arc<Context>) -> Result<(), Error> {
  let task_name = "RSS";
  let mut interval = interval(Duration::from_secs(900));
  task_info(&task_name, "Task loaded!");

  loop {
    interval.tick().await;
    let mut log_msgs: Vec<String> = Vec::new();

    match esxi_embed().await {
      Ok(Some(embed)) => {
        ChannelId::new(BINARY_PROPERTIES.rss_channel).send_message(&ctx.http, CreateMessage::new().add_embed(embed)).await.unwrap();
      },
      Ok(None) => {
        log_msgs.push("**[RSS:ESXi]:** Article returned no new content.".to_string());
      },
      Err(y) => {
        log_msgs.push(format!("**[RSS:ESXi:Error]:** Feed failed with the following error:```\n{}\n```", y));
        task_err(&task_name, &y.to_string())
      }
    }

    match gportal_embed().await {
      Ok(Some(embed)) => {
        let redis = get_redis().await;
        let rkey = "RSS_GPortal_MsgID";
        let channel = ChannelId::new(BINARY_PROPERTIES.rss_channel);

        // Check if the message ID is in Redis
        if let Ok(Some(msg_id_key)) = redis.get(&rkey).await {
          if let Ok(msg_id) = msg_id_key.parse::<u64>() {
            // Attempt to edit the message
            if let Ok(mut message) = channel.message(&ctx.http, msg_id).await {
              message.edit(&ctx.http, EditMessage::new().embed(embed)).await.unwrap();
            }
          } else {
            // If the message is not found or invalid ID, send a new message instead
            let message = channel.send_message(&ctx.http, CreateMessage::new()
              .content("*Uh-oh! G-Portal is having issues!*").add_embed(embed)
            ).await.unwrap();
            redis.set(&rkey, &message.id.to_string()).await.unwrap();
            redis.expire(&rkey, 36000).await.unwrap();
          }
        }
      },
      Ok(None) => {
        log_msgs.push("**[RSS:GPortal]:** Article returned no new content.".to_string());
      },
      Err(y) => {
        log_msgs.push(format!("**[RSS:GPortal:Error]:** Feed failed with the following error:```\n{}\n```", y));
        task_err(&task_name, &y.to_string())
      }
    }

    match rust_message().await {
      Ok(Some(content)) => {
        ChannelId::new(BINARY_PROPERTIES.rss_channel).send_message(&ctx.http, CreateMessage::new().content(content)).await.unwrap();
      },
      Ok(None) => {
        log_msgs.push("**[RSS:RustBlog]:** Article returned no new content.".to_string());
      },
      Err(y) => {
        log_msgs.push(format!("**[RSS:RustBlog:Error]:** Feed failed with the following error:```\n{}\n```", y));
        task_err(&task_name, &y.to_string())
      }
    }

    if !log_msgs.is_empty() {
      ChannelId::new(BINARY_PROPERTIES.kon_logs).send_message(
        &ctx.http, CreateMessage::new().content(log_msgs.join("\n"))
      ).await.unwrap();
    }
  }
}
