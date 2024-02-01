use crate::{
  Error,
  EMBED_COLOR,
  models::gameservers::Gameservers,
  commands::gameserver::ac_server_name
};

use gamedig::protocols::{
  valve::{
    Engine,
    Response,
    GatheringSettings
  },
  valve,
  minecraft,
  minecraft::RequestSettings,
  types::TimeoutSettings
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
use tokio::{
  net::lookup_host,
  join
};
use poise::CreateReply;
use serenity::builder::CreateEmbed;
use once_cell::sync::Lazy;
use cargo_toml::Manifest;
use serde_json::Value;

static PMS_BASE: Lazy<String> = Lazy::new(||
  var("WG_PMS").expect("Expected a \"WG_PMS\" in the envvar but none was found")
);

fn query_ats_server() -> Result<Response, Error> {
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

async fn query_gameserver(ip_address: &str) -> Result<minecraft::JavaResponse, Box<dyn std::error::Error + Send + Sync>> {
  println!("Querying {}", ip_address);

  let full_address = if ip_address.contains(':') {
    String::from(ip_address)
  } else {
    format!("{}:25565", ip_address)
  };

  let addr = match SocketAddr::from_str(&full_address) {
    Ok(addr) => addr,
    Err(_) => {
      let mut addrs = lookup_host(&full_address).await?;
      addrs.next().ok_or("Address lookup failed")?
    }
  };

  let response = minecraft::query_java(&addr, None, Some(RequestSettings {
    hostname: addr.to_string(),
    protocol_version: -1
  }));
  println!("{:?}", response);

  match response {
    Ok(response) => Ok(response),
    Err(why) => Err(Box::new(why))
  }
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
#[poise::command(slash_command, subcommands("ats", "wg", "mc"), subcommand_required)]
pub async fn status(_: poise::Context<'_, (), Error>) -> Result<(), Error> {
  Ok(())
}

/// Retrieve the server status from ATS
#[poise::command(slash_command)]
pub async fn ats(ctx: poise::Context<'_, (), Error>) -> Result<(), Error> {
  let embed = CreateEmbed::new().color(EMBED_COLOR);
  match query_ats_server() {
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

/// Retrieve the server data from given Minecraft Java server
#[poise::command(slash_command, guild_only)]
pub async fn mc(
  ctx: poise::Context<'_, (), Error>,
  #[description = "Server name"] #[autocomplete = "ac_server_name"] server_name: String
) -> Result<(), Error> {
  let server = Gameservers::get_server_data(ctx.guild_id().unwrap().into(), &server_name).await;

  match server {
    Ok(data) => {
      let name = &data[0];
      let game = &data[1];
      let ip = &data[2];

      let query_result = query_gameserver(ip).await?;
      ctx.send(CreateReply::default()
        .embed(CreateEmbed::new()
          .title(format!("{} Server Status", name))
          .fields(vec![
            ("Game", format!("{}", game), true),
            ("Players", format!("{}/{}", query_result.players_online, query_result.players_maximum), true),
            ("Version", format!("{}", query_result.game_version), true)
          ])
          .color(EMBED_COLOR)
        )
      ).await?;
      // ctx.send(CreateReply::default().content("aaa")).await?;
    },
    Err(why) => {
      ctx.send(CreateReply::default().content(format!("Error retrieving the server data: {:?}", why))).await?;
    }
  }
  
  Ok(())
}
