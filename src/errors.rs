use {
  kon_libs::{
    KonData,
    KonError,
    mention_dev
  },
  poise::FrameworkError
};

pub async fn fw_errors(error: FrameworkError<'_, KonData, KonError>) {
  match error {
    poise::FrameworkError::Command { error, ctx, .. } => {
      println!("PoiseCommandError({}): {error}", ctx.command().qualified_name);
      ctx
        .reply(format!(
          "Encountered an error during command execution, ask {} to check console for more details!",
          mention_dev(ctx).unwrap_or_default()
        ))
        .await
        .expect("Error sending message");
    },
    poise::FrameworkError::EventHandler { error, event, .. } => println!("PoiseEventHandlerError({}): {error}", event.snake_case_name()),
    poise::FrameworkError::UnknownInteraction { interaction, .. } => println!(
      "PoiseUnknownInteractionError: {} tried to execute an unknown interaction ({})",
      interaction.user.name, interaction.data.name
    ),
    other => println!("PoiseOtherError: {other}")
  }
}
