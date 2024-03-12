use crate::{
  Error,
  utils::{
    format_duration,
    format_memory,
    concat_message
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
  let mut sys = System::new_all();
  sys.refresh_all();

  // Fetch system's memory usage
  let memory_used = System::used_memory(&sys);
  let memory_free = System::free_memory(&sys);
  let memory_total = System::total_memory(&sys);

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
    format!("System: `{}`", format_duration(sys_uptime)),
    format!("Process: `{}`", format_duration(proc_uptime)),
    format!("Memory: `{} / {} / {}`", format_memory(memory_free), format_memory(memory_used), format_memory(memory_total))
  ];
  ctx.reply(concat_message(stat_msg)).await?;

  Ok(())
}
