use {
  kon_libs::KonResult,
  poise::{
    CreateReply,
    serenity_prelude::Message
  },
  serde::{
    Deserialize,
    Serialize
  }
};

#[derive(Serialize)]
struct DeepLRequest {
  text:        Vec<String>,
  target_lang: String
}

#[derive(Deserialize)]
struct DeepLResponse {
  translations: Vec<Translation>
}

#[derive(Deserialize)]
struct Translation {
  text:                     String,
  detected_source_language: String
}

/// Translate a given message using DeepL
#[poise::command(
  context_menu_command = "Translate via DeepL",
  install_context = "Guild|User",
  interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn translate(
  ctx: super::PoiseCtx<'_>,
  message: Message
) -> KonResult<()> {
  let content = message.content.trim();
  if content.is_empty() {
    ctx.send(CreateReply::new().content("Nothing to translate!").ephemeral(true)).await?;
    return Ok(());
  }

  let deepl_key = std::env::var("KON_DEEPL").expect("No 'KON_DEEPL' key found!");
  if deepl_key.is_empty() {
    ctx
      .send(
        CreateReply::new()
          .content("Can't translate this message, see console for more info!")
          .ephemeral(true)
      )
      .await?;
    return Ok(());
  }

  ctx.defer().await?;

  let api_url = if deepl_key.ends_with(":fx") {
    "https://api-free.deepl.com"
  } else {
    "https://api.deepl.com"
  };

  let client = reqwest::Client::new();
  let resp = match client
    .post(format!("{api_url}/v2/translate"))
    .header("User-Agent", "kon/reqwest")
    .header("Authorization", format!("DeepL-Auth-Key {deepl_key}"))
    .header("Content-Type", "application/json")
    .json(&DeepLRequest {
      text:        vec![content.to_string()],
      target_lang: "EN".to_string()
    })
    .send()
    .await
  {
    Ok(r) => r,
    Err(e) => {
      ctx
        .send(CreateReply::new().content(format!("**(DeepL) Service error:** {e}")).ephemeral(true))
        .await?;
      return Ok(());
    }
  };

  match resp.status().as_u16() {
    200 => (),
    403 => {
      ctx
        .send(CreateReply::new().content("Not authenticated to DeepL API").ephemeral(true))
        .await?;
      return Ok(());
    },
    429 => {
      ctx
        .send(CreateReply::new().content("DeepL requests ratelimited, slow down!").ephemeral(true))
        .await?;
      return Ok(());
    },
    456 => {
      ctx
        .send(
          CreateReply::new()
            .content("Quota exceeded, used up all of 500k characters for this month!\nConsider upgrading to **Pro** if still relying on DeepL!")
            .ephemeral(true)
        )
        .await?;
      return Ok(());
    },
    500 => {
      ctx
        .send(
          CreateReply::new()
            .content("DeepL service gave an internal server error, try again later!")
            .ephemeral(true)
        )
        .await?;
      return Ok(());
    },
    _ => {
      ctx
        .send(
          CreateReply::new()
            .content(format!("Unknown status code, DeepL returned with HTTP {}", resp.status().as_u16()))
            .ephemeral(true)
        )
        .await?;
      return Ok(());
    }
  }

  let translation: DeepLResponse = match resp.json().await {
    Ok(d) => d,
    Err(e) => {
      ctx
        .send(CreateReply::new().content(format!("**(Kon) Parsing error:** {e}")).ephemeral(true))
        .await?;
      return Ok(());
    }
  };

  if let Some(translation) = translation.translations.first() {
    ctx
      .send(
        CreateReply::new().content(
          [
            format!("**Translated from {}**", prettify_lang(translation.detected_source_language.as_str())),
            format!("```\n{}\n```", translation.text)
          ]
          .join("\n")
        )
      )
      .await?;
  } else {
    ctx
      .send(CreateReply::new().content("DeepL didn't send translated text back!").ephemeral(true))
      .await?;
  }

  Ok(())
}

fn prettify_lang(code: &str) -> &str {
  match code {
    "AR" => "Arabic",
    "BG" => "Bulgarian",
    "CS" => "Czech",
    "DA" => "Danish",
    "DE" => "German",
    "EL" => "Greek",
    "EN" => "English",
    "ES" => "Spanish",
    "ET" => "Estonian",
    "FI" => "Finnish",
    "FR" => "French",
    "HU" => "Hungarian",
    "ID" => "Indonesian",
    "IT" => "Italian",
    "JA" => "Japanese",
    "KO" => "Korean",
    "LT" => "Lithuanian",
    "LV" => "Latvian",
    "NB" => "Norwegian Bokmål",
    "NL" => "Dutch",
    "PL" => "Polish",
    "PT" => "Portuguese",
    "RO" => "Romanian",
    "RU" => "Russian",
    "SK" => "Slovak",
    "SL" => "Slovenian",
    "SV" => "Swedish",
    "TR" => "Turkish",
    "UK" => "Ukrainian",
    "ZH" => "Chinese",
    _ => code
  }
}
