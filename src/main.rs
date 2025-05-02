mod errors;
mod shutdown;
// https://cdn.toast-server.net/RustFSHiearachy.png
// Using the new filesystem hiearachy

use {
  kon_cmds::register_cmds,
  kon_libs::{
    BINARY_PROPERTIES,
    BOT_VERSION,
    GIT_COMMIT_BRANCH,
    GIT_COMMIT_HASH,
    KonResult
  },
  kon_tokens::token_path,
  poise::serenity_prelude::{
    ChannelId,
    ClientBuilder,
    Context,
    GatewayIntents,
    Ready,
    builder::{
      CreateEmbed,
      CreateEmbedAuthor,
      CreateMessage
    }
  },
  std::borrow::Cow
};

async fn on_ready(
  ctx: &Context,
  ready: &Ready
) -> KonResult<()> {
  #[cfg(not(feature = "production"))]
  {
    println!("Event[Ready][Notice] Detected a development environment!");
    let gateway = ctx.http.get_bot_gateway().await?;
    let session = gateway.session_start_limit;
    println!("Event[Ready][Notice] Session limit: {}/{}", session.remaining, session.total);
  }

  println!("Event[Ready] Build version: v{} ({GIT_COMMIT_HASH}:{GIT_COMMIT_BRANCH})", *BOT_VERSION);
  println!("Event[Ready] Connected to API as {}", ready.user.name);

  let message = CreateMessage::new();
  let ready_embed = CreateEmbed::new()
    .color(BINARY_PROPERTIES.embed_color)
    .thumbnail(ready.user.avatar_url().unwrap_or_default())
    .author(CreateEmbedAuthor::new(format!("{} is ready!", ready.user.name)));

  ChannelId::new(BINARY_PROPERTIES.ready_notify)
    .send_message(&ctx.http, message.add_embed(ready_embed))
    .await?;

  Ok(())
}

#[tokio::main]
async fn main() {
  let prefix = if BINARY_PROPERTIES.env.contains("dev") {
    Some(Cow::Borrowed("kon!"))
  } else {
    Some(Cow::Borrowed("k!"))
  };

  let framework = poise::Framework::builder()
    .options(poise::FrameworkOptions {
      commands: register_cmds(),
      prefix_options: poise::PrefixFrameworkOptions {
        prefix,
        mention_as_prefix: false,
        case_insensitive_commands: true,
        ignore_bots: true,
        ignore_thread_creation: true,
        ..Default::default()
      },
      pre_command: |ctx| {
        Box::pin(async move {
          let get_guild_name = match ctx.guild() {
            Some(guild) => guild.name.clone(),
            None => String::from("DM/User-App")
          };
          println!("Discord[{get_guild_name}] {} ran /{}", ctx.author().name, ctx.command().qualified_name);
        })
      },
      on_error: |error| Box::pin(async move { errors::fw_errors(error).await }),
      initialize_owners: true,
      ..Default::default()
    })
    .setup(|ctx, ready, _| Box::pin(on_ready(ctx, ready)))
    .build();

  let mut client = ClientBuilder::new(
    token_path().await.main,
    GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT
  )
  .framework(framework)
  .await
  .expect("Error creating client");

  let shard_manager = client.shard_manager.clone();

  tokio::spawn(async move {
    shutdown::gracefully_shutdown().await;
    shard_manager.shutdown_all().await;
  });

  if let Err(why) = client.start().await {
    println!("Error starting client: {why:#?}");
  }
}
