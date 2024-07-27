pub mod rss;

fn task_info(name: &str, message: &str) {
  println!("{}", format!("TaskScheduler[{}]: {}", name, message));
}

fn task_err(name: &str, message: &str) {
  eprintln!("{}", format!("TaskScheduler[{}:Error]: {}", name, message));
}
