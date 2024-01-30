use crate::controllers::database::DatabaseController;

// #[derive(Debug)]
pub struct MPServers {
  pub server_name: String,
  pub guild_owner: i64,
  pub is_active: bool,
  pub ip_address: String,
  pub md5_code: String
}

#[allow(dead_code)]
impl MPServers {
  pub async fn get_servers(guild_id: u64) -> Result<Self, tokio_postgres::Error> {
    let client = DatabaseController::new().await?.client;
    let row = client.query_one("
      SELECT * FROM mpservers
      WHERE guild_owner = $1
    ", &[&(guild_id as i64)]).await?;

    Ok(Self {
      server_name: row.get("server_name"),
      guild_owner: row.get("guild_owner"),
      is_active: row.get("is_active"),
      ip_address: row.get("ip_address"),
      md5_code: row.get("md5_code")
    })
  }

  pub async fn get_server_ip(guild_id: u64, server_name: &str) -> Result<(String, String), tokio_postgres::Error> {
    let client = DatabaseController::new().await?.client;
    let row = client.query_one("
      SELECT ip_address, md5_code FROM mpservers
      WHERE guild_owner = $1 AND server_name = $2
    ", &[&(guild_id as i64), &server_name]).await?;

    Ok((row.get("ip_address"), row.get("md5_code")))
  }
}
