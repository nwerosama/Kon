use {
  kon_libs::{
    BINARY_PROPERTIES,
    KonResult
  },
  kon_tokens::token_path,
  poise::{
    CreateReply,
    serenity_prelude::{
      CreateEmbed,
      Timestamp
    }
  },
  reqwest::{
    ClientBuilder,
    Error as ReqError
  },
  serde::{
    Deserialize,
    Serialize
  }
};

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

const ILO_HOSTNAME: &str = "POMNI";

enum RedfishEndpoint {
  Thermal,
  Power,
  System,
  EventService
}

impl RedfishEndpoint {
  fn url(&self) -> String {
    match self {
      RedfishEndpoint::Thermal => "Chassis/1/Thermal".to_string(),
      RedfishEndpoint::Power => "Chassis/1/Power".to_string(),
      RedfishEndpoint::System => "Systems/1".to_string(),
      RedfishEndpoint::EventService => "EventService".to_string()
    }
  }
}

async fn ilo_data(endpoint: RedfishEndpoint) -> Result<Box<dyn std::any::Any + Send>, ReqError> {
  let client = ClientBuilder::new().danger_accept_invalid_certs(true).build().unwrap();
  let res = client
    .get(format!("https://{}/redfish/v1/{}", token_path().await.ilo_ip, endpoint.url()))
    .basic_auth(token_path().await.ilo_user, Some(token_path().await.ilo_pw))
    .send()
    .await
    .unwrap();

  match endpoint {
    RedfishEndpoint::Thermal => {
      let body: Chassis = res.json().await.unwrap();
      Ok(Box::new(body))
    },
    RedfishEndpoint::Power => {
      let body: Power = res.json().await.unwrap();
      Ok(Box::new(body))
    },
    RedfishEndpoint::System => {
      let body: System = res.json().await.unwrap();
      Ok(Box::new(body))
    },
    RedfishEndpoint::EventService => {
      let body: Event = res.json().await.unwrap();
      Ok(Box::new(body))
    }
  }
}

/// Retrieve data from the HP iLO4 interface
#[poise::command(
  slash_command,
  install_context = "Guild|User",
  interaction_context = "Guild|BotDm|PrivateChannel",
  subcommands("temperature", "power", "system")
)]
pub async fn ilo(_: super::PoiseCtx<'_>) -> KonResult<()> { Ok(()) }

/// Retrieve the server's temperature data
#[poise::command(slash_command)]
pub async fn temperature(ctx: super::PoiseCtx<'_>) -> KonResult<()> {
  ctx.defer().await?;
  let ilo = ilo_data(RedfishEndpoint::Thermal).await.unwrap();
  let data = ilo.downcast_ref::<Chassis>().unwrap();
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

    tempdata.push_str(&format!("**{}:** `{}°C`\n", name, temp.reading_celsius));
  }
  for fan in &data.fans {
    if fan.current_reading == 0 {
      continue;
    }

    fandata.push_str(&format!("**{}:** `{}%`\n", fan.fan_name, fan.current_reading));
  }

  ctx
    .send(
      CreateReply::default().embed(
        CreateEmbed::new()
          .color(BINARY_PROPERTIES.embed_color)
          .timestamp(Timestamp::now())
          .title(format!("{} - Temperatures", ILO_HOSTNAME))
          .fields(vec![("Temperatures", tempdata, false), ("Fans", fandata, false)])
      )
    )
    .await?;

  Ok(())
}

/// Retrieve the server's power data
#[poise::command(slash_command)]
pub async fn power(ctx: super::PoiseCtx<'_>) -> KonResult<()> {
  ctx.defer().await?;
  let ilo = ilo_data(RedfishEndpoint::Power).await.unwrap();
  let data = ilo.downcast_ref::<Power>().unwrap();

  let mut powerdata = String::new();

  powerdata.push_str(&format!("**Power Capacity:** `{}w`\n", &data.power_capacity_watts));
  powerdata.push_str(&format!("**Power Consumed:** `{}w`\n", &data.power_consumed_watts));
  powerdata.push_str(&format!("**Average Power:** `{}w`\n", &data.power_metrics.average_consumed_watts));
  powerdata.push_str(&format!("**Max Consumed:** `{}w`\n", &data.power_metrics.max_consumed_watts));
  powerdata.push_str(&format!("**Min Consumed:** `{}w`", &data.power_metrics.min_consumed_watts));

  ctx
    .send(
      CreateReply::default().embed(
        CreateEmbed::new()
          .color(BINARY_PROPERTIES.embed_color)
          .timestamp(Timestamp::now())
          .title(format!("{} - Power", ILO_HOSTNAME))
          .description(powerdata)
      )
    )
    .await?;

  Ok(())
}

/// Retrieve the server's system data
#[poise::command(slash_command)]
pub async fn system(ctx: super::PoiseCtx<'_>) -> KonResult<()> {
  ctx.defer().await?;

  let (ilo_sys, ilo_event) = tokio::join!(ilo_data(RedfishEndpoint::System), ilo_data(RedfishEndpoint::EventService));

  let ilo_sys = ilo_sys.unwrap();
  let ilo_event = ilo_event.unwrap();

  let system_data = ilo_sys.downcast_ref::<System>().unwrap();
  let event_data = ilo_event.downcast_ref::<Event>().unwrap();

  let mut data = String::new();

  let post_state = match system_data.oem.hp.post_state.as_str() {
    "FinishedPost" => "Finished POST",
    "InPost" => "In POST (Booting)",
    "PowerOff" => "Powered off",
    _ => "Unknown State"
  };
  if system_data.oem.hp.post_state != "FinishedPost" {
    println!("iLO:PostState = {}", system_data.oem.hp.post_state);
  }

  data.push_str(&format!(
    "**Health:** `{}`\n",
    event_data.status.health.as_ref().unwrap_or(&"Unknown".to_string())
  ));
  data.push_str(&format!("**POST:** `{}`\n", post_state));
  data.push_str(&format!("**Power:** `{}`\n", &system_data.power_state));
  data.push_str(&format!("**Model:** `{}`", &system_data.model));

  ctx
    .send(
      CreateReply::default().embed(
        CreateEmbed::new()
          .color(BINARY_PROPERTIES.embed_color)
          .timestamp(Timestamp::now())
          .title(format!("{} - System", ILO_HOSTNAME))
          .description(data)
          .fields(vec![
            (
              format!("CPU ({}x)", system_data.processor_summary.count),
              system_data.processor_summary.cpu.trim().to_string(),
              true
            ),
            ("RAM".to_string(), format!("{} GB", system_data.memory.total_system_memory), true),
          ])
      )
    )
    .await?;

  Ok(())
}
