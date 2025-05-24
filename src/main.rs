mod errors;
mod events;
mod shutdown;
// https://cdn.toast-server.net/RustFSHiearachy.png
// Using the new filesystem hiearachy

use {
  kon_cmds::register_cmds,
  kon_libs::BINARY_PROPERTIES,
  kon_tokens::discord_token,
  poise::serenity_prelude::{
    ClientBuilder,
    GatewayIntents
  },
  std::borrow::Cow
};

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
          let guild_name: Cow<'_, str> = match ctx.guild() {
            Some(guild) => Cow::Owned(guild.name.clone().into()),
            None => Cow::Borrowed("Unknown Guild")
          };
          let prefix = match ctx.command().prefix_action {
            Some(_) => ctx.framework().options.prefix_options.prefix.as_ref().unwrap(),
            None => "/"
          };

          println!("Discord[{guild_name}] {} ran {prefix}{}", ctx.author().name, ctx.command().qualified_name);
        })
      },
      on_error: |error| Box::pin(async move { errors::fw_errors(error).await }),
      initialize_owners: true,
      ..Default::default()
    })
    .build();

  let mut client = ClientBuilder::new(
    discord_token().await,
    GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT
  )
  .framework(framework)
  .event_handler(events::DiscordEvents)
  .await
  .expect("Error creating client");

  let shutdown_trig = client.shard_manager.get_shutdown_trigger();

  tokio::spawn(async move {
    shutdown::gracefully_shutdown().await;
    shutdown_trig();
  });

  if let Err(why) = client.start().await {
    println!("Error starting client: {why:#?}");
  }
}
