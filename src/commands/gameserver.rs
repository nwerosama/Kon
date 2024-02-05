use crate::{
  Error,
  EMBED_COLOR,
  models::gameservers::Gameservers
};

use serenity::{
  futures::{
    stream::iter,
    future::ready,
    Stream,
    StreamExt
  },
  builder::CreateActionRow,
  builder::CreateEmbed,
};
use poise::{
  CreateReply,
  serenity_prelude,
  serenity_prelude::ButtonStyle,
  ChoiceParameter
};

#[derive(Debug, poise::ChoiceParameter)]
enum GameNames {
  #[name = "Minecraft"]
  Minecraft
}

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
  #[description = "Which game is this server running?"] game_name: GameNames,
  #[description = "IP address/domain of the server (Include the port if it has one, e.g 127.0.0.1:8080)"] ip_address: String
) -> Result<(), Error> {
  let action_row = CreateActionRow::Buttons(vec![
    serenity_prelude::CreateButton::new("add-confirm")
      .style(ButtonStyle::Success)
      .label("Yes"),
    serenity_prelude::CreateButton::new("add-cancel")
      .style(ButtonStyle::Danger)
      .label("No")
  ]);

  let reply = CreateReply::default()
    .embed(CreateEmbed::new()
      .title("Does this look correct?")
      .description(format!("
        **Server name:** `{}`
        **Game name:** `{}`
        **IP Address:** `{}`
      ", server_name, game_name.name(), ip_address))
      .color(EMBED_COLOR)
    )
    .components(vec![action_row]);

  ctx.send(reply).await?;

  while let Some(collector) = serenity_prelude::ComponentInteractionCollector::new(ctx)
    .guild_id(ctx.guild_id().unwrap())
    .author_id(ctx.author().id)
    .timeout(std::time::Duration::from_secs(30))
    .await
  {
    if collector.data.custom_id == "add-confirm" {
      let result = Gameservers::add_server(
        ctx.guild_id().unwrap().into(),
        server_name.as_str(),
        game_name.name(),
        ip_address.as_str()
      ).await;

      let mut msg = collector.message.clone();

      match result {
        Ok(_) => {
          msg.edit(
            ctx,
            serenity_prelude::EditMessage::new()
              .content("*Confirmed, added the server to database*")
              .embeds(Vec::new())
              .components(Vec::new())
          ).await?;
        },
        Err(y) => {
          msg.edit(
            ctx,
            serenity_prelude::EditMessage::new()
              .content(format!("*Error adding server to database: {:?}*", y))
              .embeds(Vec::new())
              .components(Vec::new())
          ).await?;
        }
      }
    } else if collector.data.custom_id == "add-cancel" {
      let mut msg = collector.message.clone();

      msg.edit(
        ctx,
        serenity_prelude::EditMessage::new()
          .content("*Command cancelled*")
          .embeds(Vec::new())
          .components(Vec::new())
      ).await?;
    }
  }

  Ok(())
}

/// Remove a game server from the database
#[poise::command(slash_command)]
pub async fn remove(
  ctx: poise::Context<'_, (), Error>,
  #[description = "Server name"] #[autocomplete = "ac_server_name"] server_name: String
) -> Result<(), Error> {
  let reply = CreateReply::default()
    .embed(CreateEmbed::new()
      .title("Are you sure you want to remove this server?")
      .description(format!("**Server name:** `{}`", server_name))
      .color(EMBED_COLOR)
    )
    .components(vec![
      CreateActionRow::Buttons(vec![
        serenity_prelude::CreateButton::new("delete-confirm")
          .style(ButtonStyle::Success)
          .label("Yes"),
        serenity_prelude::CreateButton::new("delete-cancel")
          .style(ButtonStyle::Danger)
          .label("No")
      ])
    ]);

  ctx.send(reply).await?;

  while let Some(collector) = serenity_prelude::ComponentInteractionCollector::new(ctx)
    .guild_id(ctx.guild_id().unwrap())
    .author_id(ctx.author().id)
    .timeout(std::time::Duration::from_secs(30))
    .await
  {
    if collector.data.custom_id == "delete-confirm" {
      let result = Gameservers::remove_server(ctx.guild_id().unwrap().into(), server_name.as_str()).await;

      let mut msg = collector.message.clone();

      match result {
        Ok(_) => {
          msg.edit(
            ctx,
            serenity_prelude::EditMessage::new()
              .content("*Confirmed, removed the server from database*")
              .embeds(Vec::new())
              .components(Vec::new())
          ).await?;
        },
        Err(y) => {
          msg.edit(
            ctx,
            serenity_prelude::EditMessage::new()
              .content(format!("*Error removing server from database: {:?}*", y))
              .embeds(Vec::new())
              .components(Vec::new())
          ).await?;
        }
      }
    } else if collector.data.custom_id == "delete-cancel" {
      let mut msg = collector.message.clone();

      msg.edit(
        ctx,
        serenity_prelude::EditMessage::new()
          .content("*Command cancelled*")
          .embeds(Vec::new())
          .components(Vec::new())
      ).await?;
    }
  }

  Ok(())
}

/// Update a game server in the database
#[poise::command(slash_command)]
pub async fn update(
  ctx: poise::Context<'_, (), Error>,
  #[description = "Server name"] #[autocomplete = "ac_server_name"] server_name: String,
  #[description = "Game name"] game_name: GameNames,
  #[description = "IP address"] ip_address: String
) -> Result<(), Error> {
  let result = Gameservers::update_server(
    ctx.guild_id().unwrap().into(),
    &server_name,
    &game_name.name(),
    &ip_address
  ).await;

  match result {
    Ok(_) => {
      ctx.send(CreateReply::default().content("Updated the server in database.")).await?;
    },
    Err(y) => {
      ctx.send(CreateReply::default().content(format!("Error updating the server in database: {:?}", y))).await?;
    }
  }

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

pub async fn ac_server_name<'a>(
  ctx: poise::Context<'_, (), Error>,
  partial: &'a str
) -> impl Stream<Item = String> + 'a {
  let result = Gameservers::get_server_names(ctx.guild_id().unwrap().into()).await;

  let names = match result {
    Ok(names_vector) => names_vector,
    Err(y) => {
      println!("Error retrieving server names: {:?}", y);
      Vec::new()
    }
  };

  iter(names)
    .filter(move |server_name| ready(server_name.starts_with(partial)))
    .map(|server_name| server_name.to_string())
}
