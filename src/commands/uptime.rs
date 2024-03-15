use crate::{
  Error,
  internals::utils::{
    format_duration,
    concat_message,
    BOT_VERSION
  }
};

use sysinfo::System;
use uptime_lib::get;
use std::time::{
  Duration,
  SystemTime,
  UNIX_EPOCH
};

/// Retrieve host and bot uptimes
#[poise::command(slash_command)]
pub async fn uptime(ctx: poise::Context<'_, (), Error>) -> Result<(), Error> {
  let _bot = ctx.http().get_current_user().await.unwrap();
  let mut sys = System::new_all();
  sys.refresh_all();

  // Fetch system's uptime
  let sys_uptime = get().unwrap().as_secs();

  // Fetch bot's process uptime
  let curr_pid = sysinfo::get_current_pid().unwrap();
  let now = SystemTime::now();
  let mut proc_uptime = 0;
  if let Some(process) = sys.process(curr_pid) {
    let time_started = UNIX_EPOCH + Duration::from_secs(process.start_time());
    proc_uptime = now.duration_since(time_started).unwrap().as_secs();
  }

  let stat_msg = vec![
    format!("**{} {}**", _bot.name, BOT_VERSION.as_str()),
    format!(">>> System: `{}`", format_duration(sys_uptime)),
    format!("Process: `{}`", format_duration(proc_uptime))
  ];
  ctx.reply(concat_message(stat_msg)).await?;

  Ok(())
}
