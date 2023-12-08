use crate::Error;

use reqwest::get;
use std::collections::HashMap;
use serde_json::Value;

lazy_static::lazy_static! {
  static ref PMS_BASE: String = std::env::var("WG_PMS").expect("Expected a \"WG_PMS\" in the envvar but none was found");
}

/// Retrieve the server data from Wargaming
#[poise::command(slash_command, subcommands("asia", "europe"))]
pub async fn status(_ctx: poise::Context<'_, (), Error>) -> Result<(), Error> {
  Ok(())
}

/// Check the server status for Asia server
#[poise::command(slash_command)]
pub async fn asia(ctx: poise::Context<'_, (), Error>) -> Result<(), Error> {
  let asia_pms = &PMS_BASE;
  let servers = pms_serverstatus(&asia_pms).await?;

  let mut fields = Vec::new();

  for server in servers {
    let name = server["name"].as_str().unwrap().to_owned();
    let status = match server["availability"].as_str().unwrap() {
      "1" => "Online",
      "-1" => "Offline",
      _ => "Unknown"
    };
    fields.push((name, status, true));
  }

  ctx.send(|m|
    m.embed(|e| {
      e.color(crate::COLOR)
        .title("World of Tanks Asia Status")
        .fields(fields)
    })
  ).await?;

  Ok(())
}

/// Check the server status for EU servers
#[poise::command(slash_command)]
pub async fn europe(ctx: poise::Context<'_, (), Error>) -> Result<(), Error> {
  let eu_pms = PMS_BASE.replace("asia", "eu");
  let servers = pms_serverstatus(&eu_pms).await?;

  let mut fields = Vec::new();

  for server in servers {
    let name = server["name"].as_str().unwrap().to_owned();
    let status = match server["availability"].as_str().unwrap() {
      "1" => "Online",
      "-1" => "Offline",
      _ => "Unknown"
    };
    fields.push((name, status, true));
  }

  ctx.send(|m|
    m.embed(|e| {
      e.color(crate::COLOR)
        .title("World of Tanks Europe Status")
        .fields(fields)
    })
  ).await?;

  Ok(())
}

async fn pms_serverstatus(url: &str) -> Result<Vec<Value>, Error> {
  let response = get(url).await?.json::<HashMap<String, Value>>().await?;
  let servers = response["data"].as_array().unwrap()[0]["servers_statuses"]["data"].as_array().unwrap().clone();

  Ok(servers)
}
