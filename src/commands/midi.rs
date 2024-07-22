use crate::{
  Error,
  internals::http::HttpClient
};

use regex::Regex;
use std::fs::{
  write,
  read_to_string,
  remove_file
};
use poise::{
  CreateReply,
  serenity_prelude::CreateAttachment
};

/// Convert MIDI file to WAV
#[poise::command(context_menu_command = "MIDI -> WAV")]
pub async fn midi_to_wav(
  ctx: poise::Context<'_, (), Error>,
  #[description = "MIDI file to be converted"] message: poise::serenity_prelude::Message
) -> Result<(), Error> {
  ctx.defer().await?;

  let http = HttpClient::new();
  let resp = http.get(&message.attachments[0].url, "MIDI Conversion").await?;
  let bytes = resp.bytes().await?;

  let midi_path = &message.attachments[0].filename;
  write(midi_path, bytes)?;

  let re = Regex::new(r"(?i)\.mid$").unwrap();

  let wav_path = re.replace(&midi_path, ".wav");

  let alpine_sf2 = include_bytes!("../internals/assets/FluidR3_GM.sf2");
  let sf2_path = if let Ok(os_release) = read_to_string("/etc/os-release") {
    if os_release.contains("Alpine") {
      let sf2_path = "/tmp/FluidR3_GM.sf2";
      write(sf2_path, alpine_sf2)?;
      sf2_path
    } else {
      "/usr/share/sounds/sf2/FluidR3_GM.sf2"
    }
  } else {
    return Err(Error::from("Couldn't read \"/etc/os-release\" file!"))
  };

  let output = std::process::Command::new("fluidsynth")
    .args(&[
      "-ni", sf2_path, midi_path, "-F", &wav_path
    ])
    .output();

  match output {
    Ok(_) => {
      ctx.send(CreateReply::default()
        .attachment(CreateAttachment::path(&*wav_path).await.unwrap())
      ).await.expect("Reply failed");

      remove_file(midi_path)?;
      remove_file(&*wav_path)?;
    },
    Err(y) => {
      ctx.send(CreateReply::default()
        .content("Command didn't execute successfully, check console for more information!")
      )
      .await.unwrap();

      return Err(Error::from(format!(
        "Midi conversion failed: {}",
        y
      )))
    }
  }

  Ok(())
}
