use crate::controllers::database::DatabaseController;

pub struct Gameservers {
  pub server_name: String,
  pub game_name: String,
  pub guild_owner: i64,
  pub guild_channel: i64,
  pub ip_address: String
}

#[allow(dead_code)]
impl Gameservers {
  pub async fn list_servers(guild_id: u64) -> Result<Vec<Self>, tokio_postgres::Error> {
    let client = DatabaseController::new().await?.client;
    let rows = client.query("
      SELECT * FROM gameservers
      WHERE guild_owner = $1
    ", &[&(guild_id as i64)]).await?;

    let mut servers = Vec::new();
    for row in rows {
      servers.push(Self {
        server_name: row.get("server_name"),
        game_name: row.get("game_name"),
        guild_owner: row.get("guild_owner"),
        guild_channel: row.get("guild_channel"),
        ip_address: row.get("ip_address")
      });
    }

    Ok(servers)
  }

  pub async fn add_server(
    guild_id: u64,
    server_name: &str,
    game_name: &str,
    guild_channel: u64,
    ip_address: &str
  ) -> Result<(), tokio_postgres::Error> {
    let client = DatabaseController::new().await?.client;
    client.execute("
      INSERT INTO gameservers (server_name, game_name, guild_owner, guild_channel, ip_address)
      VALUES ($1, $2, $3, $4, $5)
    ", &[&server_name, &game_name, &(guild_id as i64), &(guild_channel as i64), &ip_address]).await?;

    Ok(())
  }

  pub async fn remove_server(guild_id: u64, server_name: &str) -> Result<(), tokio_postgres::Error> {
    let client = DatabaseController::new().await?.client;
    client.execute("
      DELETE FROM gameservers
      WHERE guild_owner = $1 AND server_name = $2
    ", &[&(guild_id as i64), &server_name]).await?;

    Ok(())
  }

  pub async fn update_server(
    guild_id: u64,
    server_name: &str,
    game_name: &str,
    guild_channel: u64,
    ip_address: &str
  ) -> Result<(), tokio_postgres::Error> {
    let client = DatabaseController::new().await?.client;
    client.execute("
      UPDATE gameservers
      SET game_name = $1, guild_channel = $2, ip_address = $3
      WHERE guild_owner = $4 AND server_name = $5
    ", &[&game_name, &(guild_channel as i64), &ip_address, &(guild_id as i64), &server_name]).await?;

    Ok(())
  }

  pub async fn update_name(guild_id: u64, server_name: &str, new_name: &str) -> Result<(), tokio_postgres::Error> {
    let client = DatabaseController::new().await?.client;
    client.execute("
      UPDATE gameservers
      SET server_name = $1
      WHERE guild_owner = $2 AND server_name = $3
    ", &[&new_name, &(guild_id as i64), &server_name]).await?;

    Ok(())
  }
}
