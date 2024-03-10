use crate::{
  Error,
  EMBED_COLOR,
  models::gameservers::Gameservers,
  commands::gameserver::ac_server_name
};

use std::{
  collections::HashMap,
  env::var
};
use reqwest::{
  Client,
  header::USER_AGENT
};
use tokio::join;
use poise::CreateReply;
use serenity::builder::CreateEmbed;
use once_cell::sync::Lazy;
use cargo_toml::Manifest;
use serde::Deserialize;
use serde_json::Value;

static PMS_BASE: Lazy<String> = Lazy::new(||
  var("WG_PMS").expect("Expected a \"WG_PMS\" in the envvar but none was found")
);

#[derive(Deserialize)]
struct MinecraftQueryData {
  motd: Option<MinecraftMotd>,
  players: Option<MinecraftPlayers>,
  version: Option<String>,
  online: bool
}

#[derive(Deserialize)]
struct MinecraftMotd {
  clean: Vec<String>
}

#[derive(Deserialize, Clone, Copy)]
struct MinecraftPlayers {
  online: i32,
  max: i32
}

async fn pms_serverstatus(url: &str) -> Result<Vec<Value>, Error> {
  let bot_version = Manifest::from_path("Cargo.toml").unwrap().package.unwrap().version.unwrap();

  let client = Client::new();
  let req = client.get(url)
    .header(USER_AGENT, format!("Kon/{}/Rust", bot_version))
    .send()
    .await?;
  let response = req.json::<HashMap<String, Value>>().await?;
  let servers = response["data"].as_array().unwrap()[0]["servers_statuses"]["data"].as_array().unwrap().clone();

  Ok(servers)
}

async fn gs_query_minecraft(server_ip: &str) -> Result<MinecraftQueryData, Error> {
  let bot_version = Manifest::from_path("Cargo.toml").unwrap().package.unwrap().version.unwrap();

  let client = Client::new();
  let req = client.get(format!("https://api.mcsrvstat.us/2/{}", server_ip))
    .header(USER_AGENT, format!("Kon/{}/Rust", bot_version))
    .send()
    .await?;

  if req.status().is_success() {
    let data: MinecraftQueryData = req.json().await?;
    Ok(data)
  } else if req.status().is_server_error() {
    Err(Error::from("Webserver returned a 5xx error.")) 
  } else {
    Err(Error::from("Failed to query the server."))
  }
}

/// Query the server statuses
#[poise::command(slash_command, subcommands("wg", "gs"), subcommand_required)]
pub async fn status(_: poise::Context<'_, (), Error>) -> Result<(), Error> {
  Ok(())
}

/// Retrieve the server statuses from Wargaming
#[poise::command(slash_command)]
pub async fn wg(ctx: poise::Context<'_, (), Error>) -> Result<(), Error> {
  let pms_asia = &PMS_BASE;
  let pms_eu = PMS_BASE.replace("asia", "eu");
  let embed = CreateEmbed::new().color(EMBED_COLOR);

  let (servers_asia, servers_eu) = join!(pms_serverstatus(&pms_asia), pms_serverstatus(&pms_eu));

  let mut embed_fields = Vec::new();
  for server in servers_eu.unwrap() {
    let name = server["name"].as_str().unwrap().to_owned();
    let status = match server["availability"].as_str().unwrap() {
      "1" => "Online",
      "-1" => "Offline",
      _ => "Unknown"
    };
    embed_fields.push((name, status, true));
  }

  for server in servers_asia.unwrap() {
    let name = server["name"].as_str().unwrap().to_owned();
    let status = match server["availability"].as_str().unwrap() {
      "1" => "Online",
      "-1" => "Offline",
      _ => "Unknown"
    };
    embed_fields.push((name, status, true));
  }

  ctx.send(CreateReply::default().embed(embed.title("World of Tanks Server Status").fields(embed_fields))).await?;

  Ok(())
}

/// Retrieve the given server data from gameservers DB
#[poise::command(slash_command, guild_only)]
pub async fn gs(
  ctx: poise::Context<'_, (), Error>,
  #[description = "Server name"] #[autocomplete = "ac_server_name"] server_name: String
) -> Result<(), Error> {
  let server_data = Gameservers::get_server_data(ctx.guild_id().unwrap().into(), &server_name).await?;

  // Extract values from a Vec above
  let game_name = &server_data[1];
  let ip_address = &server_data[2];

  match game_name.as_str() {
    "Minecraft" => {
      let result = gs_query_minecraft(ip_address).await?;
      let embed = CreateEmbed::new().color(EMBED_COLOR);

      if result.online {
        let mut embed_fields = Vec::new();
        embed_fields.push(("Server IP".to_owned(), ip_address.to_owned(), true));
        embed_fields.push((format!("\u{200b}"), format!("\u{200b}"), true));
        embed_fields.push(("MOTD".to_owned(), format!("{}", result.motd.unwrap().clean[0]), true));
        embed_fields.push(("Players".to_owned(), format!("**{}**/**{}**", result.players.unwrap().online, result.players.clone().unwrap().max), true));
        embed_fields.push(("Version".to_owned(), result.version.unwrap(), true));

        ctx.send(CreateReply::default()
          .embed(embed
            .title(server_name)
            .fields(embed_fields)
          )
        ).await?;
      } else {
        ctx.send(CreateReply::default()
          .content(format!("**{}** (`{}`) is currently offline or unreachable.", server_name, ip_address))
        ).await?;
      }
    },
    _ => {}
  }

  Ok(())
}
