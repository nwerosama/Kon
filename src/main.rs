mod commands;
mod controllers;
mod internals;
// https://cdn.toast-server.net/RustFSHiearchy.png
// Using the new filesystem hierarchy

use crate::{
  internals::{
    utils::{
      token_path,
      mention_dev
    },
    config::BINARY_PROPERTIES
  },
  // controllers::database::DatabaseController
};

use std::{
  thread::current,
  sync::Arc
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
  Command,
  GatewayIntents
};

type Error = Box<dyn std::error::Error + Send + Sync>;

async fn on_ready(
  ctx: &Context,
  ready: &Ready,
  framework: &poise::Framework<(), Error>
) -> Result<(), Error> {
  #[cfg(not(feature = "production"))]
  {
    println!("Event[Ready][Notice]: Detected a non-production environment!");
    let gateway = ctx.http.get_bot_gateway().await?;
    let session = gateway.session_start_limit;
    println!("Event[Ready][Notice]: Session limit: {}/{}", session.remaining, session.total);
  }

  println!("Event[Ready]: Connected to API as {}", ready.user.name);

  let message = CreateMessage::new();
  let ready_embed = CreateEmbed::new()
    .color(BINARY_PROPERTIES.embed_color)
    .thumbnail(ready.user.avatar_url().unwrap_or_default())
    .author(CreateEmbedAuthor::new(format!("{} is ready!", ready.user.name)));

  ChannelId::new(BINARY_PROPERTIES.ready_notify).send_message(&ctx.http, message.add_embed(ready_embed)).await?;

  if BINARY_PROPERTIES.deploy_commands {
    let builder = poise::builtins::create_application_commands(&framework.options().commands);
    let commands = Command::set_global_commands(&ctx.http, builder).await;
    let mut commands_deployed = std::collections::HashSet::new();

    match commands {
      Ok(cmdmap) => for command in cmdmap.iter() {
        commands_deployed.insert(command.name.clone());
      },
      Err(y) => eprintln!("Error registering commands: {:?}", y)
    }

    if commands_deployed.len() > 0 {
      println!("Event[Ready]: Deployed the commands globally:\n- {}", commands_deployed.into_iter().collect::<Vec<_>>().join("\n- "));
    }
  }

  Ok(())
}

async fn event_processor(
  ctx: &Context,
  event: &FullEvent,
  framework: poise::FrameworkContext<'_, (), Error>
) -> Result<(), Error> {
  match event {
    FullEvent::Ratelimit { data } => {
      println!("Event[Ratelimit]: {:#?}", data);
    }
    FullEvent::Message { new_message } => {
      if new_message.author.bot || !new_message.guild_id.is_none() {
        return Ok(());
      }

      if new_message.content.to_lowercase().starts_with("deploy") && new_message.author.id == BINARY_PROPERTIES.developers[0] {
        let builder = poise::builtins::create_application_commands(&framework.options().commands);
        let commands = Command::set_global_commands(&ctx.http, builder).await;
        let mut commands_deployed = std::collections::HashSet::new();
      
        match commands {
          Ok(cmdmap) => for command in cmdmap.iter() {
            commands_deployed.insert(command.name.clone());
          },
          Err(y) => {
            eprintln!("Error registering commands: {:?}", y);
            new_message.reply(&ctx.http, "Deployment failed, check console for more details!").await?;
          }
        }
      
        if commands_deployed.len() > 0 {
          new_message.reply(&ctx.http, format!(
            "Deployed the commands globally:\n- {}",
            commands_deployed.into_iter().collect::<Vec<_>>().join("\n- ")
          )).await?;
        }
      }
    }
    FullEvent::Ready { .. } => {
      let thread_id = format!("{:?}", current().id());
      let thread_num: String = thread_id.chars().filter(|c| c.is_digit(10)).collect();
      println!("Event[Ready]: Task Scheduler operating on thread {}", thread_num);

      let ctx = Arc::new(ctx.clone());

      tokio::spawn(async move {
        match internals::tasks::rss::rss(ctx).await {
          Ok(_) => {},
          Err(y) => {
            eprintln!("TaskScheduler[Main:RSS:Error]: Task execution failed: {}", y);
            if let Some(source) = y.source() {
              eprintln!("TaskScheduler[Main:RSS:Error]: Task execution failed caused by: {:#?}", source);
            }
          }
        }
      });
    }
    _ => {}
  }

  Ok(())
}

#[tokio::main]
async fn main() {
  // DatabaseController::new().await.expect("Error initializing database");

  let framework = poise::Framework::builder()
    .options(poise::FrameworkOptions {
      commands: vec![
        commands::ilo::ilo(),
        commands::ping::ping(),
        commands::status::status(),
        commands::midi::midi_to_wav(),
        commands::uptime::uptime()
      ],
      pre_command: |ctx| Box::pin(async move {
        let get_guild_name = match ctx.guild() {
          Some(guild) => guild.name.clone(),
          None => String::from("Direct Message")
        };
        println!("Discord[{}] {} ran /{}", get_guild_name, ctx.author().name, ctx.command().qualified_name);
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
    | GatewayIntents::MESSAGE_CONTENT
    | GatewayIntents::DIRECT_MESSAGES
  )
  .framework(framework)
  .await.expect("Error creating client");

  if let Err(why) = client.start().await {
    println!("Error starting client: {:#?}", why);
  }
}
