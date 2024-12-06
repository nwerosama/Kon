use kon_libs::{
  KonData,
  KonError,
  KonResult,
  PoiseCtx
};

use poise::Command;

mod ilo;
mod midi;
mod status;
mod uptime;

use {
  ilo::ilo,
  midi::midi_to_wav,
  status::status,
  uptime::uptime
};

macro_rules! commands {
  ($($cmd:ident),*) => {
    vec![$($cmd()),*]
  }
}

pub fn register_cmds() -> Vec<Command<KonData, KonError>> { commands!(deploy, ping, midi_to_wav, status, ilo, uptime) }

/// Deploy the commands globally or in a guild
#[poise::command(prefix_command, owners_only, guild_only)]
pub async fn deploy(ctx: PoiseCtx<'_>) -> KonResult<()> {
  poise::builtins::register_application_commands_buttons(ctx).await?;
  Ok(())
}

/// Check if the bot is alive
#[poise::command(slash_command)]
pub async fn ping(ctx: PoiseCtx<'_>) -> KonResult<()> {
  ctx.reply(format!("Powong! `{:.2?}`", ctx.ping().await)).await?;
  Ok(())
}
