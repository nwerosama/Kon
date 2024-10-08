mod commands;
mod controllers;
mod internals;
// https://cdn.toast-server.net/RustFSHiearchy.png
// Using the new filesystem hierarchy

use crate::internals::{
  utils::{
    BOT_VERSION,
    token_path,
    mention_dev
  },
  tasks::{
    run_task,
    rss
  },
  config::BINARY_PROPERTIES
};

use std::{
  sync::Arc,
  thread::current
};
use poise::serenity_prelude::{
  builder::{
    CreateMessage,
    CreateEmbed,
    CreateEmbedAuthor
  },
  Ready,
  Context,
  FullEvent,
  ClientBuilder,
  ChannelId,
  GatewayIntents
};

type Error = Box<dyn std::error::Error + Send + Sync>;

#[cfg(feature = "production")]
pub static GIT_COMMIT_HASH: &str = env!("GIT_COMMIT_HASH");
pub static GIT_COMMIT_BRANCH: &str = env!("GIT_COMMIT_BRANCH");

#[cfg(not(feature = "production"))]
pub static GIT_COMMIT_HASH: &str = "devel";

async fn on_ready(
  ctx: &Context,
  ready: &Ready,
  _framework: &poise::Framework<(), Error>
) -> Result<(), Error> {
  #[cfg(not(feature = "production"))]
  {
    println!("Event[Ready][Notice]: Detected a non-production environment!");
    let gateway = ctx.http.get_bot_gateway().await?;
    let session = gateway.session_start_limit;
    println!("Event[Ready][Notice]: Session limit: {}/{}", session.remaining, session.total);
  }

  println!("Event[Ready]: Build version: {} ({}:{})", *BOT_VERSION, GIT_COMMIT_HASH, GIT_COMMIT_BRANCH);
  println!("Event[Ready]: Connected to API as {}", ready.user.name);

  let message = CreateMessage::new();
  let ready_embed = CreateEmbed::new()
    .color(BINARY_PROPERTIES.embed_color)
    .thumbnail(ready.user.avatar_url().unwrap_or_default())
    .author(CreateEmbedAuthor::new(format!("{} is ready!", ready.user.name)));

  ChannelId::new(BINARY_PROPERTIES.ready_notify).send_message(&ctx.http, message.add_embed(ready_embed)).await?;

  Ok(())
}

async fn event_processor(
  ctx: &Context,
  event: &FullEvent,
  _framework: poise::FrameworkContext<'_, (), Error>
) -> Result<(), Error> {
  if let FullEvent::Ready { .. } = event {
    let thread_id = format!("{:?}", current().id());
    let thread_num: String = thread_id.chars().filter(|c| c.is_ascii_digit()).collect();
    println!("Event[Ready]: Task Scheduler operating on thread {}", thread_num);

    let ctx = Arc::new(ctx.clone());
    run_task(ctx.clone(), rss).await;
  }

  Ok(())
}

#[tokio::main]
async fn main() {
  let framework = poise::Framework::builder()
    .options(poise::FrameworkOptions {
      commands: vec![
        commands::deploy(),
        commands::ilo::ilo(),
        commands::ping::ping(),
        commands::status::status(),
        commands::midi::midi_to_wav(),
        commands::uptime::uptime()
      ],
      prefix_options: poise::PrefixFrameworkOptions {
        prefix: Some(String::from("konata")),
        mention_as_prefix: false,
        case_insensitive_commands: true,
        ignore_bots: true,
        ignore_thread_creation: true,
        ..Default::default()
      },
      pre_command: |ctx| Box::pin(async move {
        let get_guild_name = match ctx.guild() {
          Some(guild) => guild.name.clone(),
          None => String::from("Direct Message")
        };
        println!("Discord[{}]: {} ran /{}", get_guild_name, ctx.author().name, ctx.command().qualified_name);
      }),
      on_error: |error| Box::pin(async move {
        match error {
          poise::FrameworkError::Command { error, ctx, .. } => {
            println!("PoiseCommandError({}): {}", ctx.command().qualified_name, error);
            ctx.reply(format!(
              "Encountered an error during command execution, ask {} to check console for more details!",
              mention_dev(ctx).unwrap_or_default()
            )).await.expect("Error sending message");
          },
          poise::FrameworkError::EventHandler { error, event, .. } => println!("PoiseEventHandlerError({}): {}", event.snake_case_name(), error),
          poise::FrameworkError::Setup { error, .. } => println!("PoiseSetupError: {}", error),
          poise::FrameworkError::UnknownInteraction { interaction, .. } => println!(
            "PoiseUnknownInteractionError: {} tried to execute an unknown interaction ({})",
            interaction.user.name,
            interaction.data.name
          ),
          other => println!("PoiseOtherError: {}", other)
        }
      }),
      initialize_owners: true,
      event_handler: |ctx, event, framework, _| Box::pin(event_processor(ctx, event, framework)),
      ..Default::default()
    })
    .setup(|ctx, ready, framework| Box::pin(on_ready(ctx, ready, framework)))
    .build();

  let mut client = ClientBuilder::new(
    token_path().await.main,
    GatewayIntents::GUILDS
    | GatewayIntents::GUILD_MESSAGES
    | GatewayIntents::MESSAGE_CONTENT
  )
  .framework(framework)
  .await.expect("Error creating client");

  if let Err(why) = client.start().await {
    println!("Error starting client: {:#?}", why);
  }
}
