use crate::Error;

pub mod ilo;
pub mod midi;
pub mod ping;
pub mod status;
pub mod uptime;

/// Deploy the commands globally or in a guild
#[poise::command(
  prefix_command,
  owners_only,
  guild_only
)]
pub async fn deploy(ctx: poise::Context<'_, (), Error>) -> Result<(), Error> {
  poise::builtins::register_application_commands_buttons(ctx).await?;
  Ok(())
}
