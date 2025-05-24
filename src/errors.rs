use {
  kon_libs::{
    KonError,
    mention_dev
  },
  poise::FrameworkError
};

pub async fn fw_errors(error: FrameworkError<'_, (), KonError>) {
  match error {
    FrameworkError::Command { error, ctx, .. } => {
      eprintln!("PoiseCommandError({}): {error}", ctx.command().qualified_name);
      ctx
        .reply(format!(
          "Encountered an error during command execution, ask {} to check console for more details!",
          mention_dev(ctx).unwrap_or_default()
        ))
        .await
        .expect("Error sending message");
    },
    FrameworkError::CommandPanic { payload, ctx, .. } => {
      if ctx
        .reply(format!(
          "Command panicked during execution, ask {} to check console for more details!",
          mention_dev(ctx).unwrap_or_default()
        ))
        .await
        .is_err()
      {
        eprintln!("PoiseCommandPanicError({}): {payload:#?}", ctx.command().qualified_name);
      }
    },
    FrameworkError::UnknownInteraction { interaction, .. } => eprintln!(
      "PoiseUnknownInteractionError: {} tried to execute an unknown interaction ({})",
      interaction.user.name, interaction.data.name
    ),
    other => eprintln!("PoiseOtherError: {other}")
  }
}
