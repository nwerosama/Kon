use crate::{
  Error,
  EMBED_COLOR,
  models::gameservers::Gameservers
};

use serenity::{
  all::Mentionable,
  builder::CreateActionRow,
  builder::CreateEmbed
};
use poise::{
  CreateReply,
  serenity_prelude,
  serenity_prelude::ButtonStyle
};

/// Manage the game servers for this guild
#[poise::command(slash_command, subcommands("add", "remove", "update", "list"), subcommand_required, guild_only)]
pub async fn gameserver(_: poise::Context<'_, (), Error>) -> Result<(), Error> {
  Ok(())
}

/// Add a game server to the database
#[poise::command(slash_command)]
pub async fn add(
  ctx: poise::Context<'_, (), Error>,
  #[description = "Server name as shown in-game or friendly name"] server_name: String,
  #[description = "Which game is this server running?"] game_name: String,
  #[channel_types("Text")] #[description = "Which channel should this server be restricted to?"] guild_channel: serenity_prelude::GuildChannel,
  #[description = "IP address/domain of the server (Include the port if it has one, e.g 127.0.0.1:8080)"] ip_address: String
) -> Result<(), Error> {
  let unsupported_games_list = [
    "ATS",
    "ETS2",
    "Euro Truck Simulator 2",
    "American Truck Simulator",
  ];
  if unsupported_games_list.contains(&game_name.as_str()) {
    ctx.send(CreateReply::default()
      .ephemeral(true)
      .content(format!("Sorry, `{}` is not supported yet due to database design.", game_name))
    ).await?;
  
    return Ok(());
  }

  let action_row = CreateActionRow::Buttons(vec![
    serenity_prelude::CreateButton::new("confirm")
      .style(ButtonStyle::Success)
      .label("Yes"),
    serenity_prelude::CreateButton::new("cancel")
      .style(ButtonStyle::Danger)
      .label("No")
  ]);

  let reply = CreateReply::default()
    .embed(CreateEmbed::new()
      .title("Does this look correct?")
      .description(format!("
        **Server name:** `{}`
        **Game name:** `{}`
        **Channel:** {}
        **IP Address:** `{}`
      ", server_name, game_name, guild_channel.mention(), ip_address))
      .color(EMBED_COLOR)
    )
    .components(vec![action_row]);

  ctx.send(reply).await?;

  while let Some(collector) = serenity_prelude::ComponentInteractionCollector::new(ctx)
    .channel_id(ctx.channel_id())
    .guild_id(ctx.guild_id().unwrap())
    .author_id(ctx.author().id)
    .timeout(std::time::Duration::from_secs(30))
    .await
  {
    if collector.data.custom_id == "confirm" {
      let result = Gameservers::add_server(
        ctx.guild_id().unwrap().into(),
        server_name.as_str(),
        game_name.as_str(),
        guild_channel.id.into(),
        ip_address.as_str()
      ).await;

      let mut msg = collector.message.clone();

      match result {
        Ok(_) => {
          msg.edit(
            ctx,
            serenity_prelude::EditMessage::new()
              .content("*Confirmed, added the server to database*")
              .components(Vec::new())
          ).await?;
        },
        Err(y) => {
          msg.edit(
            ctx,
            serenity_prelude::EditMessage::new()
              .content(format!("*Error adding server to database: {:?}*", y))
              .components(Vec::new())
          ).await?;
        }
      }
    } else if collector.data.custom_id == "cancel" {
      let mut msg = collector.message.clone();

      msg.edit(
        ctx,
        serenity_prelude::EditMessage::new()
          .content("*Command cancelled*")
          .components(Vec::new())
      ).await?;
    }
  }

  Ok(())
}

/// Remove a game server from the database
#[poise::command(slash_command)]
pub async fn remove(ctx: poise::Context<'_, (), Error>) -> Result<(), Error> {
  ctx.send(CreateReply::default().content("Yet to be implemented.")).await?;

  Ok(())
}

/// Update a game server in the database
#[poise::command(slash_command)]
pub async fn update(ctx: poise::Context<'_, (), Error>) -> Result<(), Error> {
  ctx.send(CreateReply::default().content("Yet to be implemented.")).await?;

  Ok(())
}

/// List all the available game servers for this guild
#[poise::command(slash_command)]
pub async fn list(ctx: poise::Context<'_, (), Error>) -> Result<(), Error> {
  let servers = Gameservers::list_servers(ctx.guild_id().unwrap().into()).await?;

  let mut embed_fields = Vec::new();
  for server in servers {
    embed_fields.push(
      (server.server_name, format!("Game: `{}`\nIP: `{}`", server.game_name, server.ip_address), true)
    );
  }

  ctx.send(CreateReply::default()
    .embed(CreateEmbed::new()
      .title("List of registered gameservers")
      .fields(embed_fields)
      .color(EMBED_COLOR)
    )
  ).await?;

  Ok(())
}
