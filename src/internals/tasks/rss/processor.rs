use super::{
  task_err,
  TASK_NAME,
  BINARY_PROPERTIES,
  get_redis,
  esxi::esxi_embed,
  github::github_embed,
  gportal::gportal_embed,
  rust::rust_message
};

use regex::Regex;
use tokio::time::{
  Duration,
  sleep
};
use poise::serenity_prelude::{
  Context,
  ChannelId,
  EditMessage,
  CreateMessage
};

  //  This is for building up the embed with the feed data
  /* std::fs::File::create("rss_name.log").unwrap();
  std::fs::write("rss_name.log", format!("{:#?}", feed))?; */

pub async fn feed_processor(ctx: &Context) {
  let mut log_msgs: Vec<String> = Vec::new();

  match esxi_embed().await {
    Ok(Some(embed)) => {
      ChannelId::new(BINARY_PROPERTIES.rss_channel).send_message(&ctx.http, CreateMessage::new().add_embed(embed)).await.unwrap();
    },
    Ok(None) => (),
    Err(y) => {
      log_msgs.push(format!("**[{TASK_NAME}:ESXi:Error]:** Feed failed with the following error:```\n{}\n```", y));
      task_err(&TASK_NAME, &y.to_string())
    }
  }

  match gportal_embed().await {
    Ok(Some(embed)) => {
      let redis = get_redis().await;
      let rkey = "RSS_GPortal_MsgID";
      let channel = ChannelId::new(BINARY_PROPERTIES.rss_channel);

      // Check if the message ID is in Redis
      match redis.get(&rkey).await {
        Ok(Some(msg_id_key)) => {
          // Fetch the cached content
          let cached_content: Option<String> = redis.get("RSS_GPortal_Content").await.unwrap_or(None);

          if let Ok(msg_id) = msg_id_key.parse::<u64>() {
            // Attempt to edit the message
            if let Ok(mut message) = channel.message(&ctx.http, msg_id).await {
              let new_desc = message.embeds[0].description.clone().unwrap();

              if cached_content.as_deref() != Some(&new_desc) {
                message.edit(&ctx.http, EditMessage::new().embed(embed)).await.unwrap();
              }

              sleep(Duration::from_secs(25)).await;

              if Regex::new(r"(?i)\bresolved\b").unwrap().is_match(&new_desc) {
                message.reply(&ctx.http, "This incident has been marked as resolved!").await.unwrap();
                redis.del(&rkey).await.unwrap();
              }
            }
          }
        },
        Ok(None) | Err(_) => {
          // If the message is invalid ID, send a new message instead
          let message = channel.send_message(&ctx.http, CreateMessage::new()
            .content("*Uh-oh! G-Portal is having issues!*").add_embed(embed)
          ).await.unwrap();
          redis.set(&rkey, &message.id.to_string()).await.unwrap();
          redis.expire(&rkey, 36000).await.unwrap();
        }
      }
    },
    Ok(None) => (),
    Err(y) => {
      log_msgs.push(format!("**[{TASK_NAME}:GPortal:Error]:** Feed failed with the following error:```\n{}\n```", y));
      task_err(&TASK_NAME, &y.to_string())
    }
  }

  match github_embed().await {
    Ok(Some(embed)) => {
      let redis = get_redis().await;
      let rkey = "RSS_GitHub_MsgID";
      let channel = ChannelId::new(BINARY_PROPERTIES.rss_channel);

      // Check if the message ID is in Redis
      match redis.get(&rkey).await {
        Ok(Some(msg_id_key)) => {
          // Fetch the cached content
          let cached_content: Option<String> = redis.get("RSS_GitHub_Content").await.unwrap_or(None);

          if let Ok(msg_id) = msg_id_key.parse::<u64>() {
            // Attempt to edit the message
            if let Ok(mut message) = channel.message(&ctx.http, msg_id).await {
              let new_desc = message.embeds[0].description.clone().unwrap();

              if cached_content.as_deref() != Some(&new_desc) {
                message.edit(&ctx.http, EditMessage::new().embed(embed)).await.unwrap();
              }

              sleep(Duration::from_secs(25)).await;

              if Regex::new(r"(?i)\bresolved\b").unwrap().is_match(&new_desc) {
                message.reply(&ctx.http, "This incident has been marked as resolved!").await.unwrap();
                redis.del(&rkey).await.unwrap();
              }
            }
          }
        },
        Ok(None) | Err(_) => {
          // If the message is not found, send a new message instead
          let message = channel.send_message(&ctx.http, CreateMessage::new().add_embed(embed)).await.unwrap();
          redis.set(&rkey, &message.id.to_string()).await.unwrap();
          redis.expire(&rkey, 36000).await.unwrap();
        }
      }
    },
    Ok(None) => (),
    Err(y) => {
      log_msgs.push(format!("**[{TASK_NAME}:GitHub:Error]:** Feed failed with the following error:```\n{}\n```", y));
      task_err(&TASK_NAME, &y.to_string())
    }
  }

  match rust_message().await {
    Ok(Some(content)) => {
      ChannelId::new(BINARY_PROPERTIES.rss_channel).send_message(&ctx.http, CreateMessage::new().content(content)).await.unwrap();
    },
    Ok(None) => (),
    Err(y) => {
      log_msgs.push(format!("**[{TASK_NAME}:RustBlog:Error]:** Feed failed with the following error:```\n{}\n```", y));
      task_err(&TASK_NAME, &y.to_string())
    }
  }

  if !log_msgs.is_empty() {
    ChannelId::new(BINARY_PROPERTIES.kon_logs).send_message(
      &ctx.http, CreateMessage::new().content(log_msgs.join("\n"))
    ).await.unwrap();
  }
}
