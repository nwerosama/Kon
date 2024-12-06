mod rss;

pub use rss::rss;

use {
  kon_libs::KonResult,
  poise::serenity_prelude::Context,
  std::{
    future::Future,
    sync::{
      Arc,
      atomic::{
        AtomicBool,
        Ordering
      }
    }
  },
  tokio::task::spawn
};

fn task_info(
  name: &str,
  message: &str
) {
  println!("TaskScheduler[{name}]: {message}")
}

fn task_err(
  name: &str,
  message: &str
) {
  eprintln!("TaskScheduler[{name}:Error]: {message}")
}

static TASK_RUNNING: AtomicBool = AtomicBool::new(false);

pub async fn run_task<F, T>(
  ctx: Arc<Context>,
  task: F
) where
  F: Fn(Arc<Context>) -> T + Send + 'static,
  T: Future<Output = KonResult<()>> + Send + 'static
{
  let ctx_cl = Arc::clone(&ctx);

  if !TASK_RUNNING.load(Ordering::SeqCst) {
    TASK_RUNNING.store(true, Ordering::SeqCst);
    spawn(async move {
      if let Err(y) = task(ctx_cl).await {
        eprintln!("TaskScheduler[Main:Error]: Failed to execute the task, error reason: {y}");
        if let Some(source) = y.source() {
          eprintln!("TaskScheduler[Main:Error]: Failed to execute the task, this is caused by: {source:#?}");
        }
      }
      TASK_RUNNING.store(false, Ordering::SeqCst);
    });
  }
}
