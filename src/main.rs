// https://cdn.toast-server.net/RustFSHiearchy.png
// Using the new filesystem hierarchy

#[cfg(feature = "rss")]
use {
  kon_tasks::{
    rss,
    run_task
  },
  std::{
    sync::Arc,
    thread::current
  }
};

use {
  kon_cmds::register_cmds,
  kon_libs::{
    BINARY_PROPERTIES,
    BOT_VERSION,
    GIT_COMMIT_BRANCH,
    GIT_COMMIT_HASH,
    KonData,
    KonResult,
    PoiseFwCtx,
    mention_dev
  },
  kon_tokens::token_path,
  poise::serenity_prelude::{
    ChannelId,
    ClientBuilder,
    Context,
    FullEvent,
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
) -> KonResult<KonData> {
  #[cfg(not(feature = "production"))]
  {
    println!("Event[Ready][Notice]: Detected a non-production environment!");
    let gateway = ctx.http.get_bot_gateway().await?;
    let session = gateway.session_start_limit;
    println!("Event[Ready][Notice]: Session limit: {}/{}", session.remaining, session.total);
  }

  println!("Event[Ready]: Build version: {} ({GIT_COMMIT_HASH}:{GIT_COMMIT_BRANCH})", *BOT_VERSION);
  println!("Event[Ready]: Connected to API as {}", ready.user.name);

  let message = CreateMessage::new();
  let ready_embed = CreateEmbed::new()
    .color(BINARY_PROPERTIES.embed_color)
    .thumbnail(ready.user.avatar_url().unwrap_or_default())
    .author(CreateEmbedAuthor::new(format!("{} is ready!", ready.user.name)));

  ChannelId::new(BINARY_PROPERTIES.ready_notify)
    .send_message(&ctx.http, message.add_embed(ready_embed))
    .await?;

  Ok(KonData {})
}

async fn event_processor(
  framework: PoiseFwCtx<'_>,
  event: &FullEvent
) -> KonResult<()> {
  #[cfg(feature = "rss")]
  if let FullEvent::Ready { .. } = event {
    let thread_id = format!("{:?}", current().id());
    let thread_num: String = thread_id.chars().filter(|c| c.is_ascii_digit()).collect();
    println!("Event[Ready]: Task Scheduler operating on thread {thread_num}");

    let ctx = Arc::new(framework.serenity_context.clone());
    run_task(ctx.clone(), rss).await;
  }

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
          println!("Discord[{get_guild_name}]: {} ran /{}", ctx.author().name, ctx.command().qualified_name);
        })
      },
      on_error: |error| {
        Box::pin(async move {
          match error {
            poise::FrameworkError::Command { error, ctx, .. } => {
              println!("PoiseCommandError({}): {error}", ctx.command().qualified_name);
              ctx
                .reply(format!(
                  "Encountered an error during command execution, ask {} to check console for more details!",
                  mention_dev(ctx).unwrap_or_default()
                ))
                .await
                .expect("Error sending message");
            },
            poise::FrameworkError::EventHandler { error, event, .. } => println!("PoiseEventHandlerError({}): {error}", event.snake_case_name()),
            poise::FrameworkError::UnknownInteraction { interaction, .. } => println!(
              "PoiseUnknownInteractionError: {} tried to execute an unknown interaction ({})",
              interaction.user.name, interaction.data.name
            ),
            other => println!("PoiseOtherError: {other}")
          }
        })
      },
      initialize_owners: true,
      event_handler: |framework, event| Box::pin(event_processor(framework, event)),
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

  if let Err(why) = client.start().await {
    println!("Error starting client: {why:#?}");
  }
}
