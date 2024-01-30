use crate::{
  models::mpservers::MPServers,
  EMBED_COLOR,
  Error
};

use gamedig::protocols::{
  valve::{
    Engine, GatheringSettings, Response
  },
  types::TimeoutSettings,
  valve
};
use std::{
  str::FromStr,
  net::SocketAddr,
  time::Duration,
  collections::HashMap,
  env::var
};
use reqwest::{
  Client,
  header::USER_AGENT
};
use poise::CreateReply;
use serenity::builder::CreateEmbed;
use once_cell::sync::Lazy;
use cargo_toml::Manifest;
use serde_json::Value;
use tokio::join;

static PMS_BASE: Lazy<String> = Lazy::new(||
  var("WG_PMS").expect("Expected a \"WG_PMS\" in the envvar but none was found")
);

fn query_server() -> Result<Response, Error> {
  let server_ip = var("ATS_SERVER_IP").expect("Expected a \"ATS_SERVER_IP\" in the envvar but none was found");
  let addr = SocketAddr::from_str(&server_ip).unwrap();
  let engine = Engine::Source(None);
  let gather_settings = GatheringSettings {
    players: true,
    rules: false,
    check_app_id: false
  };

  let read_timeout = Duration::from_secs(2);
  let write_timeout = Duration::from_secs(2);
  let retries = 1;
  let timeout_settings = TimeoutSettings::new(
    Some(read_timeout),
    Some(write_timeout),
    retries
  ).unwrap();

  let response = valve::query(
    &addr,
    engine,
    Some(gather_settings),
    Some(timeout_settings)
  );

  Ok(response?)
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

/// Query the server statuses
#[poise::command(slash_command, subcommands("ats", "wg", "fs"), subcommand_required)]
pub async fn status(_: poise::Context<'_, (), Error>) -> Result<(), Error> {
  Ok(())
}

/// Retrieve the server status from ATS
#[poise::command(slash_command)]
pub async fn ats(ctx: poise::Context<'_, (), Error>) -> Result<(), Error> {
  let embed = CreateEmbed::new().color(EMBED_COLOR);
  match query_server() {
    Ok(response) => {
      ctx.send(CreateReply::default()
        .embed(embed
          .title("American Truck Simulator Server Status")
          .fields(vec![
            ("Name", format!("{}", response.info.name), true),
            ("Players", format!("{}/{}", response.info.players_online, response.info.players_maximum), true)
          ])
      )).await?;
    }
    Err(why) => println!("Error querying the server: {:?}", why)
  }

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

/// Retrieve the data from Farming Simulator 22 server
#[poise::command(slash_command, guild_only)]
pub async fn fs(ctx: poise::Context<'_, (), Error>) -> Result<(), Error> {
  // let embed = CreateEmbed::new().color(EMBED_COLOR);
  let server = MPServers::get_server_ip(ctx.guild_id().unwrap().into(), "testserver").await?;
  let ip = server.0;
  let md5 = server.1;

  ctx.send(CreateReply::default().content(format!("IP: {}\nMD5: {}", ip, md5))).await?;

  // ctx.send(CreateReply::default().content("This command is not yet implemented")).await?;

  Ok(())
}
