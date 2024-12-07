use kon_libs::{
  BINARY_PROPERTIES,
  KonResult
};

use super::{
  RSSFeedBox,
  RSSFeedOutput,
  TASK_NAME,
  get_redis
};

use {
  poise::serenity_prelude::{
    ChannelId,
    Context,
    CreateEmbed,
    CreateMessage,
    EditMessage,
    Http
  },
  regex::Regex,
  std::sync::Arc,
  tokio::time::{
    Duration,
    sleep
  }
};

//  This is for building up the embed with the feed data
/* std::fs::File::create("rss_name.log").unwrap();
std::fs::write("rss_name.log", format!("{:#?}", feed))?; */

async fn process_regular_embed(
  http: &Http,
  embed: CreateEmbed,
  redis_key: &str
) -> KonResult<()> {
  let redis = get_redis().await;
  let channel = ChannelId::new(BINARY_PROPERTIES.rss_channel);

  let msg_id_key: Option<String> = redis.get(redis_key).await?;

  if let Some(msg_id_key) = msg_id_key {
    if let Ok(msg_id) = msg_id_key.parse::<u64>() {
      if let Ok(mut message) = channel.message(http, msg_id).await {
        message.edit(http, EditMessage::new().embed(embed)).await?;
      }
    }
  } else {
    let message = channel.send_message(http, CreateMessage::new().add_embed(embed)).await?;
    redis.set(redis_key, &message.id.to_string()).await?;
    redis.expire(redis_key, 36000).await?;
  }

  Ok(())
}

/// Cache-based embed updater for ongoing outages/incidents
async fn process_incident_embed(
  http: &Http,
  embed: CreateEmbed,
  redis_key: &str,
  content_key: &str
) -> KonResult<()> {
  let redis = get_redis().await;
  let channel = ChannelId::new(BINARY_PROPERTIES.rss_channel);

  let msg_id_key: Option<String> = redis.get(redis_key).await?;
  let cached_content: Option<String> = redis.get(content_key).await.unwrap_or(None);

  if let Some(msg_id_key) = msg_id_key {
    if let Ok(msg_id) = msg_id_key.parse::<u64>() {
      if let Ok(mut message) = channel.message(http, msg_id).await {
        if let Some(existing) = message.embeds.first() {
          let new_description = existing.description.clone().unwrap();

          if cached_content.as_deref() != Some(&new_description) {
            message.edit(http, EditMessage::new().embed(embed)).await?;
          }

          sleep(Duration::from_secs(15)).await;

          if Regex::new(r"(?i)\bresolved\b").unwrap().is_match(&new_description) {
            redis.del(redis_key).await?;
          }
        }
      }
    }
  } else {
    let message = channel.send_message(http, CreateMessage::new().add_embed(embed)).await?;
    redis.set(redis_key, &message.id.to_string()).await?;
    redis.expire(redis_key, 36000).await?;
  }

  Ok(())
}

/// Process the content string
async fn process_msg_content(
  http: &Http,
  content: String,
  redis_key: &str
) -> KonResult<()> {
  let redis = get_redis().await;
  let channel = ChannelId::new(BINARY_PROPERTIES.rss_channel);

  let msg_id_key: Option<String> = redis.get(redis_key).await?;

  if let Some(msg_id_key) = msg_id_key {
    if let Ok(msg_id) = msg_id_key.parse::<u64>() {
      channel.edit_message(http, msg_id, EditMessage::new().content(content)).await?;
    }
  } else {
    let message = channel.send_message(http, CreateMessage::new().content(content)).await?;
    redis.set(redis_key, &message.id.to_string()).await?;
    redis.expire(redis_key, 36000).await?;
  }

  Ok(())
}

pub struct RSSProcessor {
  pub feeds: Vec<RSSFeedBox>
}

impl RSSProcessor {
  pub fn new() -> Self { Self { feeds: Vec::new() } }

  pub fn add_feed(
    &mut self,
    feed: RSSFeedBox
  ) {
    self.feeds.push(feed);
  }

  pub async fn process_all(
    &self,
    ctx: Arc<Context>
  ) -> KonResult<()> {
    let mut discord_msg: Vec<String> = Vec::new();

    for feed in &self.feeds {
      let feed_name = feed.name();
      let redis_key = format!("RSS_{feed_name}_MsgId");
      let error_msg = format!("**[{TASK_NAME}:{feed_name}:Error]:** Feed failed with the following error:```\n{{ error }}\n```");

      match feed.process(ctx.clone()).await {
        Ok(Some(output)) => match output {
          RSSFeedOutput::RegularEmbed(embed) => {
            if let Err(e) = process_regular_embed(&ctx.http, embed, &redis_key).await {
              discord_msg.push(error_msg.replace("{{ error }}", &e.to_string()))
            }
          },
          RSSFeedOutput::IncidentEmbed(embed) => {
            if let Err(e) = process_incident_embed(&ctx.http, embed, &redis_key, &format!("RSS_{feed_name}_Content")).await {
              discord_msg.push(error_msg.replace("{{ error }}", &e.to_string()))
            }
          },
          RSSFeedOutput::Content(content) => {
            if let Err(e) = process_msg_content(&ctx.http, content, &redis_key).await {
              discord_msg.push(error_msg.replace("{{ error }}", &e.to_string()))
            }
          },
        },
        Ok(None) => (),
        Err(e) => discord_msg.push(error_msg.replace("{{ error }}", &e.to_string()))
      }
    }

    if !discord_msg.is_empty() {
      ChannelId::new(BINARY_PROPERTIES.kon_logs)
        .send_message(&ctx.http, CreateMessage::new().content(discord_msg.join("\n")))
        .await?;
    }

    Ok(())
  }
}
