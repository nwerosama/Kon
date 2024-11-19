use crate::Error;

pub mod ilo;
pub mod midi;
pub mod ping;
pub mod status;
pub mod uptime;

type PoiseCtx<'a> = poise::Context<'a, (), Error>;

/// Deploy the commands globally or in a guild
#[poise::command(
  prefix_command,
  owners_only,
  guild_only
)]
pub async fn deploy(ctx: PoiseCtx<'_>) -> Result<(), Error> {
  poise::builtins::register_application_commands_buttons(ctx).await?;
  Ok(())
}
