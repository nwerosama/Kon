use {
  kon_tokens::token_path,
  poise::{
    CreateReply,
    serenity_prelude::builder::CreateEmbed
  },
  serde_json::Value,
  std::collections::HashMap,
  tokio::join
};

use kon_libs::{
  BINARY_PROPERTIES,
  HttpClient,
  KonResult
};

async fn pms_serverstatus(url: &str) -> KonResult<Vec<(String, Vec<Value>)>> {
  let client = HttpClient::new();
  let req = client.get(url, "PMS-Status").await?;

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
  let id_name_map: HashMap<&str, &str> = [("wotbsg", "ASIA"), ("wowssg", "WoWS (ASIA)"), ("wowseu", "WoWS (EU)")]
    .iter()
    .cloned()
    .collect();

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
      server_map
        .entry(title.clone())
        .or_default()
        .push((name.to_owned().to_string(), status.to_owned()));
    }
  }

  let mut statuses = Vec::new();
  for (title, servers) in server_map {
    let servers_str = servers
      .iter()
      .map(|(name, status)| format!("{name}: {status}"))
      .collect::<Vec<String>>()
      .join("\n");
    statuses.push((title, servers_str, true));
  }
  statuses
}

/// Query the server statuses
#[poise::command(
  slash_command,
  install_context = "Guild|User",
  interaction_context = "Guild|BotDm|PrivateChannel",
  subcommands("wg")
)]
pub async fn status(_: super::PoiseCtx<'_>) -> KonResult<()> { Ok(()) }

/// Retrieve the server statuses from Wargaming
#[poise::command(slash_command)]
pub async fn wg(ctx: super::PoiseCtx<'_>) -> KonResult<()> {
  let pms_asia = token_path().await.wg_pms;
  let pms_eu = pms_asia.replace("asia", "eu");
  let embed = CreateEmbed::new().color(BINARY_PROPERTIES.embed_color);

  let (servers_asia, servers_eu) = join!(pms_serverstatus(&pms_asia), pms_serverstatus(&pms_eu));
  let joined_pms_servers = [servers_eu.unwrap(), servers_asia.unwrap()].concat();
  let pms_servers = process_pms_statuses(joined_pms_servers.to_vec());

  ctx
    .send(CreateReply::default().embed(embed.title("Wargaming Server Status").fields(pms_servers)))
    .await?;

  Ok(())
}
