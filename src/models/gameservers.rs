use crate::controllers::database::DatabaseController;

pub struct Gameservers {
  pub server_name: String,
  pub game_name: String,
  pub guild_owner: i64,
  pub ip_address: String
}

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
        ip_address: row.get("ip_address")
      });
    }

    Ok(servers)
  }

  pub async fn add_server(
    guild_id: u64,
    server_name: &str,
    game_name: &str,
    ip_address: &str
  ) -> Result<(), tokio_postgres::Error> {
    let client = DatabaseController::new().await?.client;
    client.execute("
      INSERT INTO gameservers (server_name, game_name, guild_owner, ip_address)
      VALUES ($1, $2, $3, $4)
    ", &[&server_name, &game_name, &(guild_id as i64), &ip_address]).await?;

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
    ip_address: &str
  ) -> Result<(), tokio_postgres::Error> {
    let client = DatabaseController::new().await?.client;
    client.execute("
      UPDATE gameservers
      SET game_name = $1, ip_address = $2
      WHERE guild_owner = $3 AND server_name = $4
    ", &[&game_name, &ip_address, &(guild_id as i64), &server_name]).await?;

    Ok(())
  }

  pub async fn get_server_names(guild_id: u64) -> Result<Vec<String>, tokio_postgres::Error> {
    let client = DatabaseController::new().await?.client;
    let rows = client.query("
      SELECT server_name FROM gameservers
      WHERE guild_owner = $1
    ", &[&(guild_id as i64)]).await?;
  
    let mut servers = Vec::new();
    for row in rows {
      servers.push(row.get("server_name"));
    }
  
    Ok(servers)
  }

  pub async fn get_server_data(guild_id: u64, server_name: &str) -> Result<Vec<String>, tokio_postgres::Error> {
    let client = DatabaseController::new().await?.client;
    let rows = client.query("
      SELECT * FROM gameservers
      WHERE guild_owner = $1 AND server_name = $2
    ", &[&(guild_id as i64), &server_name]).await?;

    let mut server = Vec::new();
    for row in rows {
      server.push(row.get("server_name"));
      server.push(row.get("game_name"));
      server.push(row.get("ip_address"))
    }

    Ok(server)
  }
}
