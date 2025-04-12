use {
  futures::future,
  kon_libs::{
    BINARY_PROPERTIES,
    HttpClient,
    KonResult
  },
  kon_tokens::token_path,
  poise::{
    CreateReply,
    serenity_prelude::builder::CreateEmbed
  },
  serde_json::Value,
  std::{
    collections::HashMap,
    sync::OnceLock,
    time::Duration
  }
};

type IdNameHashmap = HashMap<&'static str, &'static str>;

const HTTP_TIMEOUT: Duration = Duration::from_secs(5);

fn id_name_map() -> &'static IdNameHashmap {
  static ID_NAME_MAP: OnceLock<IdNameHashmap> = OnceLock::new();
  ID_NAME_MAP.get_or_init(|| [("wotbsg", "ASIA"), ("wowssg", "ASIA"), ("wowseu", "EU")].iter().cloned().collect())
}

async fn pms_serverstatus(
  http: &HttpClient,
  url: String
) -> Result<Vec<(String, Vec<Value>)>, String> {
  let req = match tokio::time::timeout(HTTP_TIMEOUT, http.get(url.as_str(), "PMS-Status")).await {
    Ok(result) => match result {
      Ok(req) => req,
      Err(e) => return Err(format!("Failed to connect: {e}"))
    },
    Err(_) => return Err("Request timed out".to_string())
  };

  let response = match tokio::time::timeout(HTTP_TIMEOUT, req.json::<HashMap<String, Value>>()).await {
    Ok(result) => match result {
      Ok(data) => data,
      Err(e) => return Err(format!("Failed to parse response: {e}"))
    },
    Err(_) => return Err("Response parsing timed out".to_string())
  };

  let data = match response.get("data").and_then(|d| d.as_array()) {
    Some(d) => d,
    None => return Err("Invalid response format".to_string())
  };

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

  for (title, mapped_servers) in servers {
    for server in mapped_servers {
      let name = server["name"].as_str().unwrap();
      let id = server["id"].as_str().unwrap().split(":").next().unwrap_or("");
      let status = match server["availability"].as_str().unwrap() {
        "1" => "Online",
        "-1" => "Offline",
        _ => "Unknown"
      };
      let name = id_name_map().get(id).unwrap_or(&name);
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
      .map(|(name, status)| format!("**{name}:** {status}"))
      .collect::<Vec<String>>()
      .join("\n");
    statuses.push((title, servers_str, true));
  }
  statuses
}

/// Query the server statuses from Wargaming
#[poise::command(slash_command, install_context = "Guild|User", interaction_context = "Guild|BotDm|PrivateChannel")]
pub async fn wargaming(ctx: super::PoiseCtx<'_>) -> KonResult<()> {
  ctx.defer().await?;

  static REGIONS: [(&str, &str); 4] = [
    ("asia", "Asia (WoT)"),
    ("eu", "Europe (WoT)"),
    ("wgcb", "Console (WoTX)"),
    ("lg", "Europe (WoWL)")
  ];

  let pms_base = token_path().await.wg_pms;
  let mut embed = CreateEmbed::new().color(BINARY_PROPERTIES.embed_color);

  let http = HttpClient::new();
  let mut futures = Vec::with_capacity(REGIONS.len());
  let mut region_names = Vec::with_capacity(REGIONS.len());

  for (region, name) in &REGIONS {
    let url = if *region == "asia" {
      pms_base.clone()
    } else {
      pms_base.replace("asia", region)
    };

    futures.push(pms_serverstatus(&http, url));
    region_names.push(name);
  }

  let results = future::join_all(futures).await;

  let mut pms_servers = Vec::new();
  let mut errors = Vec::new();

  for (i, result) in results.into_iter().enumerate() {
    match result {
      Ok(servers) => pms_servers.extend(servers),
      Err(e) => errors.push(format!("**{}:** {e}", region_names[i]))
    }
  }

  let status_fields = process_pms_statuses(pms_servers);
  embed = embed.title("Wargaming Server Status");

  if !errors.is_empty() {
    embed = embed.description(format!("No response from certain servers:\n{}", errors.join("\n")));
  }

  ctx.send(CreateReply::default().embed(embed.fields(status_fields))).await?;

  Ok(())
}
