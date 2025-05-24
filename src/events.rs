use {
  kon_libs::{
    BINARY_PROPERTIES,
    BOT_VERSION,
    GIT_COMMIT_BRANCH,
    GIT_COMMIT_HASH
  },
  poise::{
    async_trait,
    serenity_prelude::{
      Context,
      EventHandler,
      FullEvent,
      GenericChannelId,
      builder::{
        CreateEmbed,
        CreateEmbedAuthor,
        CreateMessage
      }
    }
  }
};

pub struct DiscordEvents;

#[async_trait]
impl EventHandler for DiscordEvents {
  async fn dispatch(
    &self,
    ctx: &Context,
    event: &FullEvent
  ) {
    if let FullEvent::Ready { data_about_bot, .. } = event {
      #[cfg(not(feature = "production"))]
      {
        println!("Event[Ready][Notice] Detected a development environment!");
        let gateway = ctx.http.get_bot_gateway().await.unwrap();
        let session = gateway.session_start_limit;
        println!("Event[Ready][Notice] Session limit: {}/{}", session.remaining, session.total);
      }

      println!("Event[Ready] Build version: v{} ({GIT_COMMIT_HASH}:{GIT_COMMIT_BRANCH})", *BOT_VERSION);
      println!("Event[Ready] Connected to API as {}", data_about_bot.user.name);

      let message = CreateMessage::new();
      let ready_embed = CreateEmbed::new()
        .color(BINARY_PROPERTIES.embed_color)
        .thumbnail(data_about_bot.user.avatar_url().unwrap_or_default())
        .author(CreateEmbedAuthor::new(format!("{} is ready!", data_about_bot.user.name)));

      GenericChannelId::new(BINARY_PROPERTIES.ready_notify)
        .send_message(&ctx.http, message.add_embed(ready_embed))
        .await
        .unwrap();
    }
  }
}
