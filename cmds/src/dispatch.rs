mod ilo;
mod status;
mod translate;
mod uptime;

use kon_libs::{
  KonData,
  KonError,
  KonResult,
  PoiseCtx
};

use {
  ilo::ilo,
  status::status,
  translate::translate,
  uptime::uptime
};

macro_rules! commands {
  ($($cmd:ident),*) => {
    vec![$($cmd()),*]
  }
}

pub fn register_cmds() -> Vec<poise::Command<KonData, KonError>> { commands!(deploy, ping, ilo, status, translate, uptime) }

/// Deploy the commands globally or in a guild
#[poise::command(prefix_command, owners_only, guild_only)]
pub async fn deploy(ctx: PoiseCtx<'_>) -> KonResult<()> {
  poise::builtins::register_application_commands_buttons(ctx).await?;
  Ok(())
}

/// Check if the bot is alive
#[poise::command(slash_command, install_context = "Guild|User", interaction_context = "Guild|BotDm|PrivateChannel")]
pub async fn ping(ctx: PoiseCtx<'_>) -> KonResult<()> {
  ctx.reply(format!("Powong! **{:.0?}**", ctx.ping().await)).await?;
  Ok(())
}
