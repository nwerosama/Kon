use crate::{
  Error,
  models::gameservers::Gameservers,
  commands::gameserver::ac_server_name,
  internals::utils::EMBED_COLOR,
  internals::http::HttpClient,
  internals::utils::token_path
};

use std::collections::HashMap;
use tokio::join;
use poise::CreateReply;
use poise::serenity_prelude::builder::CreateEmbed;
use serde::Deserialize;
use serde_json::Value;

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

async fn pms_serverstatus(url: &str) -> Result<Vec<(String, Vec<Value>)>, Error> {
  let client = HttpClient::new();
  let req = client.get(url).await?;

  let response = req.json::<HashMap<String, Value>>().await?;
  let data = response["data"].as_array().unwrap();

  let mut servers = Vec::new();
  for item in data {
    if let Some(title) = item["title"].as_str() {
      if let Some(servers_statuses) = item["servers_statuses"]["data"].as_array() {
        if !servers_statuses.is_empty() {
          servers.push((title.to_owned(), servers_statuses.clone()));
        }
      }
    }
  }

  Ok(servers)
}

fn process_pms_statuses(servers: Vec<(String, Vec<Value>)>) -> Vec<(String, String, bool)> {
  let mut server_map: HashMap<String, Vec<(String, String)>> = HashMap::new();
  let id_name_map: HashMap<&str, &str> = [
    ("wotbsg", "ASIA"),
    ("wowssg", "WoWS (ASIA)"),
    ("wowseu", "WoWS (EU)")
  ].iter().cloned().collect();

  for (title, mapped_servers) in servers {
    for server in mapped_servers {
      let name = server["name"].as_str().unwrap();
      let id = server["id"].as_str().unwrap().split(":").next().unwrap_or("");
      let status = match server["availability"].as_str().unwrap() {
        "1" => "Online",
        "-1" => "Offline",
        _ => "Unknown"
      };
      let name = id_name_map.get(id).unwrap_or(&name);
      server_map.entry(title.clone()).or_insert_with(Vec::new).push((name.to_owned().to_string(), status.to_owned()));
    }
  }

  let mut statuses = Vec::new();
  for (title, servers) in server_map {
    let servers_str = servers.iter().map(|(name, status)| format!("{}: {}", name, status)).collect::<Vec<String>>().join("\n");
    statuses.push((title, servers_str, true));
  }
  statuses
}

async fn gs_query_minecraft(server_ip: &str) -> Result<MinecraftQueryData, Error> {
  let client = HttpClient::new();
  let req = client.get(&format!("https://api.mcsrvstat.us/2/{}", server_ip)).await?;

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
  let pms_asia = token_path().await.wg_pms;
  let pms_eu = pms_asia.replace("asia", "eu");
  let embed = CreateEmbed::new().color(EMBED_COLOR);

  let (servers_asia, servers_eu) = join!(pms_serverstatus(&pms_asia), pms_serverstatus(&pms_eu));
  let joined_pms_servers = [servers_eu.unwrap(), servers_asia.unwrap()].concat();
  let pms_servers = process_pms_statuses(joined_pms_servers.to_vec());

  ctx.send(CreateReply::default().embed(embed.title("Wargaming Server Status").fields(pms_servers))).await?;

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
