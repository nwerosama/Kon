use kon_libs::{
  GIT_COMMIT_BRANCH,
  GIT_COMMIT_HASH
};

use {
  kon_libs::{
    BOT_VERSION,
    KonResult,
    format_duration
  },
  std::{
    fs::File,
    io::{
      BufRead,
      BufReader
    },
    path::Path,
    time::{
      Duration,
      SystemTime,
      UNIX_EPOCH
    }
  },
  sysinfo::System,
  uptime_lib::get
};

fn get_os_info() -> String {
  let path = Path::new("/etc/os-release");
  let mut name = "BoringOS".to_string();
  let mut version = "v0.0".to_string();

  if let Ok(file) = File::open(path) {
    let reader = BufReader::new(file);
    let set_value = |s: String| s.split('=').nth(1).unwrap_or_default().trim_matches('"').to_string();
    reader.lines().map_while(Result::ok).for_each(|line| match line {
      l if l.starts_with("NAME=") => name = set_value(l),
      l if l.starts_with("VERSION=") => version = set_value(l),
      l if l.starts_with("VERSION_ID=") => version = set_value(l),
      _ => {}
    });
  }

  format!("{name} {version}")
}

/// Retrieve host and bot uptimes
#[poise::command(slash_command, install_context = "Guild|User", interaction_context = "Guild|BotDm|PrivateChannel")]
pub async fn uptime(ctx: super::PoiseCtx<'_>) -> KonResult<()> {
  let bot = ctx.http().get_current_user().await.unwrap();
  let mut sys = System::new_all();
  sys.refresh_all();

  // Fetch system's uptime
  let sys_uptime = get().unwrap().as_secs();

  // Fetch system's processor
  let cpu = sys.cpus();

  // Fetch bot's process uptime
  let curr_pid = sysinfo::get_current_pid().unwrap();
  let now = SystemTime::now();
  let mut proc_uptime = 0;
  if let Some(process) = sys.process(curr_pid) {
    let time_started = UNIX_EPOCH + Duration::from_secs(process.start_time());
    proc_uptime = now.duration_since(time_started).unwrap().as_secs();
  }

  let stat_msg = [
    format!("**{} {}** `{GIT_COMMIT_HASH}:{GIT_COMMIT_BRANCH}`", bot.name, BOT_VERSION.as_str()),
    format!(">>> System: `{}`", format_duration(sys_uptime)),
    format!("Process: `{}`", format_duration(proc_uptime)),
    format!("CPU: `{}`", cpu[0].brand()),
    format!("OS: `{}`", get_os_info())
  ];
  ctx.reply(stat_msg.join("\n")).await?;

  Ok(())
}
