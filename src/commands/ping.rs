use crate::Error;

/// Check if the bot is alive
#[poise::command(slash_command)]
pub async fn ping(ctx: super::PoiseCtx<'_>) -> Result<(), Error> {
  ctx.reply(format!("Powong! `{:?}`", ctx.ping().await)).await?;
  Ok(())
}
