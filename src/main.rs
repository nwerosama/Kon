mod commands;

use poise::serenity_prelude::{self as serenity};

type Error = Box<dyn std::error::Error + Send + Sync>;

pub static COLOR: i32 = 0x5a99c7;

async fn on_ready(
  ctx: &serenity::Context,
  ready: &serenity::Ready,
  framework: &poise::Framework<(), Error>
) -> Result<(), Error> {
  println!("Connected to API as {}", ready.user.name);

  serenity::ChannelId(865673694184996888).send_message(&ctx.http, |m| m.embed(|e|
    e.color(COLOR)
      .thumbnail(ready.user.avatar_url().unwrap_or_default())
      .author(|a|
        a.name(format!("{} is ready!", ready.user.name))
      )
  )).await?;

  let register_commands = std::env::var("REGISTER_CMDS").unwrap_or_else(|_| String::from("true")).parse::<bool>().unwrap_or(true);

  if register_commands {
    let builder = poise::builtins::create_application_commands(&framework.options().commands);
    let commands = serenity::Command::set_global_application_commands(&ctx.http, |commands| {
      *commands = builder.clone();
      commands
    }).await;

    match commands {
      Ok(cmdmap) => {
        for command in cmdmap.iter() {
          println!("Registered command globally: {}", command.name);
        }
      },
      Err(why) => println!("Error registering commands: {:?}", why)
    }
  }

  Ok(())
}

#[tokio::main]
async fn main() {
  let token = std::env::var("DISCORD_TOKEN").expect("Expected a \"DISCORD_TOKEN\" in the envvar but none was found");

  let client = poise::Framework::builder().token(token)
    .intents(serenity::GatewayIntents::MESSAGE_CONTENT | serenity::GatewayIntents::GUILDS)
    .options(poise::FrameworkOptions {
      commands: vec![
        commands::ping::ping(),
        commands::wg_status::wg_status(),
        commands::ats_status::ats_status()
      ],
      pre_command: |ctx| Box::pin(async move {
        let get_guild_name = match ctx.guild() {
          Some(guild) => guild.name.clone(),
          None => String::from("DM")
        };
        println!("[{}] {} ran /{}", get_guild_name, ctx.author().name, ctx.command().qualified_name)
      }),
      ..Default::default()
    })
    .setup(|ctx, ready, framework| Box::pin(on_ready(ctx, ready, framework)))
    .build().await.expect("Error while building the client");

  if let Err(why) = client.start().await {
    println!("Client error: {:?}", why);
  }
}
