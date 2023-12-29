use crate::{Error, COLOR};

use reqwest::{
  Client,
  header::USER_AGENT
};
use std::{
  collections::HashMap,
  env::var
};
use cargo_toml::Manifest;
use serde_json::Value;
use tokio::join;

lazy_static::lazy_static! {
  static ref PMS_BASE: String = var("WG_PMS").expect("Expected a \"WG_PMS\" in the envvar but none was found");
}

/// Retrieve the server statuses from Wargaming
#[poise::command(slash_command)]
pub async fn wg_status(ctx: poise::Context<'_, (), Error>) -> Result<(), Error> {
  let pms_asia = &PMS_BASE;
  let pms_eu = PMS_BASE.replace("asia", "eu");

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

  ctx.send(|m| m.embed(|e|
    e.color(COLOR)
      .title("World of Tanks Server Status")
      .fields(embed_fields)
  )).await?;

  Ok(())
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
