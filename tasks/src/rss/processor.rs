use kon_libs::{
  BINARY_PROPERTIES,
  KonResult
};

use super::{
  TASK_NAME,
  esxi::esxi_embed,
  get_redis,
  github::github_embed,
  gportal::gportal_embed,
  rust::rust_message,
  task_err
};

use {
  poise::serenity_prelude::{
    ChannelId,
    Context,
    CreateEmbed,
    CreateMessage,
    EditMessage
  },
  regex::Regex,
  tokio::time::{
    Duration,
    sleep
  }
};

//  This is for building up the embed with the feed data
/* std::fs::File::create("rss_name.log").unwrap();
std::fs::write("rss_name.log", format!("{:#?}", feed))?; */

// todo; have a reusable function for feeding RSS data and building the embed out of it.
//       see github.rs / esxi.rs / gportal.rs for references of this idea.

async fn process_embed(
  ctx: &Context,
  embed: Option<CreateEmbed>,
  redis_key: &str,
  content_key: &str
) -> KonResult<()> {
  if let Some(embed) = embed {
    let redis = get_redis().await;
    let channel = ChannelId::new(BINARY_PROPERTIES.rss_channel);

    let msg_id_key: Option<String> = redis.get(redis_key).await?;
    let cached_content: Option<String> = redis.get(content_key).await.unwrap_or(None);

    if let Some(msg_id_key) = msg_id_key {
      if let Ok(msg_id) = msg_id_key.parse::<u64>() {
        if let Ok(mut message) = channel.message(&ctx.http, msg_id).await {
          let new_description = message.embeds[0].description.clone().unwrap();

          if cached_content.as_deref() != Some(&new_description) {
            message.edit(&ctx.http, EditMessage::new().embed(embed)).await?;
          }

          sleep(Duration::from_secs(15)).await;

          if Regex::new(r"(?i)\bresolved\b").unwrap().is_match(&new_description) {
            redis.del(redis_key).await?;
          }
        }
      }
    } else {
      let message = channel.send_message(&ctx.http, CreateMessage::new().add_embed(embed)).await?;
      redis.set(redis_key, &message.id.to_string()).await?;
      redis.expire(redis_key, 36000).await?;
    }
  }

  Ok(())
}

pub async fn feed_processor(ctx: &Context) {
  let mut log_msgs: Vec<String> = Vec::new();

  match esxi_embed().await {
    Ok(Some(embed)) => {
      ChannelId::new(BINARY_PROPERTIES.rss_channel)
        .send_message(&ctx.http, CreateMessage::new().add_embed(embed))
        .await
        .unwrap();
    },
    Ok(None) => (),
    Err(y) => {
      log_msgs.push(format!(
        "**[{TASK_NAME}:ESXi:Error]:** Feed failed with the following error:```\n{}\n```",
        y
      ));
      task_err(TASK_NAME, &y.to_string())
    }
  }

  match gportal_embed().await {
    Ok(Some(embed)) => process_embed(ctx, Some(embed), "RSS_GPortal_MsgID", "RSS_GPortal_Content").await.unwrap(),
    Ok(None) => (),
    Err(y) => {
      log_msgs.push(format!(
        "**[{TASK_NAME}:GPortal:Error]:** Feed failed with the following error:```\n{}\n```",
        y
      ));
      task_err(TASK_NAME, &y.to_string())
    }
  }

  match github_embed().await {
    Ok(Some(embed)) => process_embed(ctx, Some(embed), "RSS_GitHub_MsgID", "RSS_GitHub_Content").await.unwrap(),
    Ok(None) => (),
    Err(y) => {
      log_msgs.push(format!(
        "**[{TASK_NAME}:GitHub:Error]:** Feed failed with the following error:```\n{}\n```",
        y
      ));
      task_err(TASK_NAME, &y.to_string())
    }
  }

  match rust_message().await {
    Ok(Some(content)) => {
      ChannelId::new(BINARY_PROPERTIES.rss_channel)
        .send_message(&ctx.http, CreateMessage::new().content(content))
        .await
        .unwrap();
    },
    Ok(None) => (),
    Err(y) => {
      log_msgs.push(format!(
        "**[{TASK_NAME}:RustBlog:Error]:** Feed failed with the following error:```\n{}\n```",
        y
      ));
      task_err(TASK_NAME, &y.to_string())
    }
  }

  if !log_msgs.is_empty() {
    ChannelId::new(BINARY_PROPERTIES.kon_logs)
      .send_message(&ctx.http, CreateMessage::new().content(log_msgs.join("\n")))
      .await
      .unwrap();
  }
}
