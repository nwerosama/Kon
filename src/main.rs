mod commands;
mod controllers;
mod models;
mod internals;

use std::{
  env::var,
  error
};
use poise::serenity_prelude::{
  builder::{
    CreateMessage,
    CreateEmbed,
    CreateEmbedAuthor
  },
  Context,
  Ready,
  ClientBuilder,
  ChannelId,
  Command,
  GatewayIntents
};

type Error = Box<dyn error::Error + Send + Sync>;

static BOT_READY_NOTIFY: u64 = 865673694184996888;

async fn on_ready(
  ctx: &Context,
  ready: &Ready,
  framework: &poise::Framework<(), Error>
) -> Result<(), Error> {
  println!("Connected to API as {}", ready.user.name);

  let message = CreateMessage::new();
  let ready_embed = CreateEmbed::new()
    .color(internals::utils::EMBED_COLOR)
    .thumbnail(ready.user.avatar_url().unwrap_or_default())
    .author(CreateEmbedAuthor::new(format!("{} is ready!", ready.user.name)).clone());

  ChannelId::new(BOT_READY_NOTIFY).send_message(&ctx.http, message.add_embed(ready_embed)).await?;

  let register_commands = var("REGISTER_CMDS").unwrap_or_else(|_| String::from("true")).parse::<bool>().unwrap_or(true);

  if register_commands {
    let builder = poise::builtins::create_application_commands(&framework.options().commands);
    let commands = Command::set_global_commands(&ctx.http, builder).await;

    match commands {
      Ok(cmdmap) => for command in cmdmap.iter() {
        println!("Registered command globally: {}", command.name);
      },
      Err(why) => println!("Error registering commands: {:?}", why)
    }
  }

  Ok(())
}

#[tokio::main]
async fn main() {
  let token = var("DISCORD_TOKEN").expect("Expected a \"DISCORD_TOKEN\" in the envvar but none was found");
  let db = controllers::database::DatabaseController::new().await.expect("Failed to connect to database");

  let framework = poise::Framework::builder()
    .options(poise::FrameworkOptions {
      commands: vec![
        commands::ping::ping(),
        commands::uptime::uptime(),
        commands::status::status(),
        commands::gameserver::gameserver()
      ],
      pre_command: |ctx| Box::pin(async move {
        let get_guild_name = match ctx.guild() {
          Some(guild) => guild.name.clone(),
          None => String::from("DM")
        };
        println!("[{}] {} ran /{}", get_guild_name, ctx.author().name, ctx.command().qualified_name)
      }),
      on_error: |error| Box::pin(async move {
        match error {
          poise::FrameworkError::Command { error, ctx, .. } => {
            println!("PoiseCommandError({}): {}", ctx.command().qualified_name, error);
          }
          other => println!("PoiseOtherError: {:?}", other)
        }
      }),
      initialize_owners: true,
      ..Default::default()
    })
    .setup(|ctx, ready, framework| Box::pin(on_ready(ctx, ready, framework)))
    .build();

  let mut client = ClientBuilder::new(token, GatewayIntents::GUILDS)
    .framework(framework)
    .await.expect("Error creating client");

  {
    let mut data = client.data.write().await;
    data.insert::<controllers::database::DatabaseController>(db);
  }

  if let Err(why) = client.start().await {
    println!("Client error: {:?}", why);
  }
}
