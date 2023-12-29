use crate::{Error, COLOR};

use gamedig::protocols::{
  valve::{
    Engine, GatheringSettings, Response
  },
  types::TimeoutSettings,
  valve,
};
use std::{
  str::FromStr,
  net::SocketAddr,
  time::Duration,
  env::var
};

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

/// Retrieve the server status from ATS
#[poise::command(slash_command)]
pub async fn ats_status(ctx: poise::Context<'_, (), Error>) -> Result<(), Error> {
  match query_server() {
    Ok(response) => {
      ctx.send(|m| m.embed(|e|
        e.color(COLOR)
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
