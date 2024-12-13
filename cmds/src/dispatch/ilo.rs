use {
  kon_libs::{
    BINARY_PROPERTIES,
    KonResult
  },
  kon_tokens::token_path,
  lazy_static::lazy_static,
  poise::{
    CreateReply,
    serenity_prelude::{
      CreateEmbed,
      Timestamp
    }
  },
  reqwest::{
    Client,
    ClientBuilder,
    Error as ReqError
  },
  serde::{
    Deserialize,
    Serialize,
    de::DeserializeOwned
  }
};

const ILO_HOSTNAME: &str = "POMNI";

lazy_static! {
  static ref REQWEST_CLIENT: Client = ClientBuilder::new().danger_accept_invalid_certs(true).build().unwrap();
}

#[derive(Serialize, Deserialize)]
struct Chassis {
  #[serde(rename = "Fans")]
  fans:         Vec<Fan>,
  #[serde(rename = "Temperatures")]
  temperatures: Vec<Temperature>
}

#[derive(Serialize, Deserialize)]
struct Fan {
  #[serde(rename = "CurrentReading")]
  current_reading: i32,
  #[serde(rename = "FanName")]
  fan_name:        String,
  #[serde(rename = "Status")]
  status:          Status
}

#[derive(Serialize, Deserialize)]
struct Temperature {
  #[serde(rename = "CurrentReading")]
  current_reading:          i32,
  #[serde(rename = "Name")]
  name:                     String,
  #[serde(rename = "ReadingCelsius")]
  reading_celsius:          i32,
  #[serde(rename = "Status")]
  status:                   Status,
  #[serde(rename = "Units")]
  units:                    String,
  #[serde(rename = "UpperThresholdCritical")]
  upper_threshold_critical: i32,
  #[serde(rename = "UpperThresholdFatal")]
  upper_threshold_fatal:    i32
}

#[derive(Serialize, Deserialize)]
struct Status {
  #[serde(rename = "Health")]
  health: Option<String>,
  #[serde(rename = "State")]
  state:  String
}

#[derive(Serialize, Deserialize, Debug)]
struct Power {
  #[serde(rename = "PowerCapacityWatts")]
  power_capacity_watts: i32,
  #[serde(rename = "PowerConsumedWatts")]
  power_consumed_watts: i32,
  #[serde(rename = "PowerMetrics")]
  power_metrics:        PowerMetrics
}

#[derive(Serialize, Deserialize, Debug)]
struct PowerMetrics {
  #[serde(rename = "AverageConsumedWatts")]
  average_consumed_watts: i32,
  #[serde(rename = "MaxConsumedWatts")]
  max_consumed_watts:     i32,
  #[serde(rename = "MinConsumedWatts")]
  min_consumed_watts:     i32
}

#[derive(Serialize, Deserialize)]
struct System {
  #[serde(rename = "Memory")]
  memory:            Memory,
  #[serde(rename = "Model")]
  model:             String,
  #[serde(rename = "Oem")]
  oem:               Oem,
  #[serde(rename = "PowerState")]
  power_state:       String,
  #[serde(rename = "ProcessorSummary")]
  processor_summary: ProcessorSummary
}

#[derive(Serialize, Deserialize)]
struct Memory {
  #[serde(rename = "TotalSystemMemoryGB")]
  total_system_memory: i32
}

#[derive(Serialize, Deserialize)]
struct ProcessorSummary {
  #[serde(rename = "Count")]
  count: i32,
  #[serde(rename = "Model")]
  cpu:   String
}

#[derive(Serialize, Deserialize)]
struct Oem {
  #[serde(rename = "Hp")]
  hp: Hp
}

#[derive(Serialize, Deserialize)]
struct Hp {
  #[serde(rename = "PostState")]
  post_state: String
}

#[derive(Serialize, Deserialize)]
struct Event {
  #[serde(rename = "Status")]
  status: Status
}

#[derive(Serialize, Deserialize)]
/// HP calls this Integrated Management Log
struct Iml {
  #[serde(rename = "Items")]
  items: Vec<ImlEntry>
}

#[derive(Serialize, Deserialize)]
struct ImlEntry {
  #[serde(rename = "Created")]
  created:  String,
  #[serde(rename = "Message")]
  message:  String,
  #[serde(rename = "Severity")]
  severity: String
}

enum RedfishEndpoint {
  Thermal,
  Power,
  System,
  EventService,
  LogServices
}

impl RedfishEndpoint {
  fn url(&self) -> String {
    match self {
      RedfishEndpoint::Thermal => "Chassis/1/Thermal".to_string(),
      RedfishEndpoint::Power => "Chassis/1/Power".to_string(),
      RedfishEndpoint::System => "Systems/1".to_string(),
      RedfishEndpoint::EventService => "EventService".to_string(),
      RedfishEndpoint::LogServices => "Systems/1/LogServices/IML/Entries".to_string()
    }
  }
}

async fn ilo_data<T: DeserializeOwned>(endpoint: RedfishEndpoint) -> Result<T, ReqError> {
  let client = &*REQWEST_CLIENT;
  let token = token_path().await;
  let redfish_url = format!("https://{}/redfish/v1/{}", token.ilo_ip, endpoint.url());

  let res = client.get(redfish_url).basic_auth(token.ilo_user, Some(token.ilo_pw)).send().await?;

  res.json::<T>().await
}

fn embed_builder(
  title: &str,
  description: Option<String>,
  fields: Option<Vec<(String, String, bool)>>
) -> CreateEmbed {
  let mut embed = CreateEmbed::new()
    .color(BINARY_PROPERTIES.embed_color)
    .timestamp(Timestamp::now())
    .title(format!("{ILO_HOSTNAME} - {title}"));

  if let Some(d) = description {
    embed = embed.description(d);
  }

  if let Some(f) = fields {
    for (name, value, inline) in f {
      embed = embed.field(name, value, inline);
    }
  }

  embed
}

fn fmt_dt(input: &str) -> Option<String> {
  let parts: Vec<&str> = input.split('T').collect();
  if parts.len() != 2 {
    return None;
  }

  let date_parts: Vec<&str> = parts[0].split('-').collect();
  if date_parts.len() != 3 {
    return None;
  }

  let date = format!("{}/{}/{}", date_parts[2], date_parts[1], date_parts[0]);
  let time = parts[1].trim_end_matches('Z');

  Some(format!("{date} {time}"))
}

/// Retrieve data from the HP iLO interface
#[poise::command(
  slash_command,
  install_context = "Guild|User",
  interaction_context = "Guild|BotDm|PrivateChannel",
  subcommands("temperature", "power", "system", "logs")
)]
pub async fn ilo(_: super::PoiseCtx<'_>) -> KonResult<()> { Ok(()) }

/// Retrieve the server's temperature data
#[poise::command(slash_command)]
async fn temperature(ctx: super::PoiseCtx<'_>) -> KonResult<()> {
  ctx.defer().await?;
  let data: Chassis = ilo_data(RedfishEndpoint::Thermal).await?;
  let mut tempdata = String::new();
  let mut fandata = String::new();

  let allowed_sensors = ["01-Inlet Ambient", "04-P1 DIMM 1-6", "14-Chipset Zone"];

  for temp in &data.temperatures {
    if temp.reading_celsius == 0 || !allowed_sensors.contains(&temp.name.as_str()) {
      continue;
    }

    let name = match temp.name.as_str() {
      "01-Inlet Ambient" => "Inlet Ambient",
      "04-P1 DIMM 1-6" => "P1 DIMM 1-6",
      "14-Chipset Zone" => "Chipset Zone",
      _ => "Unknown Sensor"
    };

    tempdata.push_str(&format!("**{name}:** `{}°C`\n", temp.reading_celsius));
  }
  for fan in &data.fans {
    if fan.current_reading == 0 {
      continue;
    }

    fandata.push_str(&format!("**{}:** `{}%`\n", fan.fan_name, fan.current_reading));
  }

  ctx
    .send(CreateReply::default().embed(embed_builder(
      "Temperatures",
      None,
      Some(vec![("Temperatures".to_string(), tempdata, false), ("Fans".to_string(), fandata, false)])
    )))
    .await?;

  Ok(())
}

/// Retrieve the server's power data
#[poise::command(slash_command)]
async fn power(ctx: super::PoiseCtx<'_>) -> KonResult<()> {
  ctx.defer().await?;
  let data: Power = ilo_data(RedfishEndpoint::Power).await?;

  let mut powerdata = String::new();

  powerdata.push_str(&format!("**Power Capacity:** `{}w`\n", &data.power_capacity_watts));
  powerdata.push_str(&format!("**Power Consumed:** `{}w`\n", &data.power_consumed_watts));
  powerdata.push_str(&format!("**Average Power:** `{}w`\n", &data.power_metrics.average_consumed_watts));
  powerdata.push_str(&format!("**Max Consumed:** `{}w`\n", &data.power_metrics.max_consumed_watts));
  powerdata.push_str(&format!("**Min Consumed:** `{}w`", &data.power_metrics.min_consumed_watts));

  ctx
    .send(CreateReply::default().embed(embed_builder("Power", Some(powerdata), None)))
    .await?;

  Ok(())
}

/// Retrieve the server's system data
#[poise::command(slash_command)]
async fn system(ctx: super::PoiseCtx<'_>) -> KonResult<()> {
  ctx.defer().await?;

  let (ilo_sys, ilo_event) = tokio::join!(ilo_data(RedfishEndpoint::System), ilo_data(RedfishEndpoint::EventService));

  let ilo_sys: System = ilo_sys.unwrap();
  let ilo_event: Event = ilo_event.unwrap();

  let mut data = String::new();

  let post_state = match ilo_sys.oem.hp.post_state.as_str() {
    "FinishedPost" => "Finished POST",
    "InPost" => "In POST (Booting)",
    "PowerOff" => "Powered off",
    _ => "Unknown State"
  };
  if ilo_sys.oem.hp.post_state != "FinishedPost" {
    println!("iLO:PostState = {}", ilo_sys.oem.hp.post_state);
  }

  data.push_str(&format!(
    "**Health:** `{}`\n",
    ilo_event.status.health.as_ref().unwrap_or(&"Unknown".to_string())
  ));
  data.push_str(&format!("**POST:** `{post_state}`\n"));
  data.push_str(&format!("**Power:** `{}`\n", &ilo_sys.power_state));
  data.push_str(&format!("**Model:** `{}`", &ilo_sys.model));

  ctx
    .send(CreateReply::default().embed(embed_builder(
      "System",
      Some(data),
      Some(vec![
        (
          format!("CPU ({}x)", ilo_sys.processor_summary.count),
          ilo_sys.processor_summary.cpu.trim().to_string(),
          true
        ),
        ("RAM".to_string(), format!("{} GB", ilo_sys.memory.total_system_memory), true),
      ])
    )))
    .await?;

  Ok(())
}

/// Retrieve the server's IML data
#[poise::command(slash_command)]
async fn logs(ctx: super::PoiseCtx<'_>) -> KonResult<()> {
  ctx.defer().await?;

  let data: Iml = ilo_data(RedfishEndpoint::LogServices).await?;
  let mut log_entries = String::new();

  for entry in data.items.iter().rev().take(5) {
    let dt = fmt_dt(&entry.created).unwrap_or_else(|| "Unknown".to_string());
    log_entries.push_str(&format!("**[{}:{dt}]:** {}\n", entry.severity, entry.message));
  }

  ctx
    .send(CreateReply::default().embed(embed_builder("IML", Some(log_entries), None)))
    .await?;

  Ok(())
}
