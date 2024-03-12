pub fn concat_message(messages: Vec<String>) -> String {
  messages.join("\n")
}

pub fn format_duration(secs: u64) -> String {
  let days = secs / 86400;
  let hours = (secs % 86400) / 3600;
  let minutes = (secs % 3600) / 60;
  let seconds = secs % 60;

  let mut formatted_string = String::new();
  if days > 0 {
    formatted_string.push_str(&format!("{}d, ", days));
  }
  if hours > 0 || days > 0 {
    formatted_string.push_str(&format!("{}h, ", hours));
  }
  if minutes > 0 || hours > 0 {
    formatted_string.push_str(&format!("{}m, ", minutes));
  }
  formatted_string.push_str(&format!("{}s", seconds));

  formatted_string
}

pub fn format_memory(bytes: u64) -> String {
  let kb = 1024;
  let mb = 1024 * 1024;
  let gb = 1024 * 1024 * 1024;

  match bytes {
    b if b >= gb => format!("{:.0} GB", (b as f64 / (1024.0 * 1024.0 * 1024.0)).ceil()),
    b if b >= mb => format!("{:.0} MB", (b as f64 / (1024.0 * 1024.0)).ceil()),
    b if b >= kb => format!("{:.0} KB", (b as f64 / 1024.0).ceil()),
    _ => format!("{:.0} B", bytes),
  }
}
