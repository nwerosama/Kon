use {
  kon_libs::KonResult,
  poise::{
    CreateReply,
    serenity_prelude::Message
  },
  serde::{
    Deserialize,
    Serialize
  },
  std::{
    collections::HashMap,
    sync::{
      LazyLock,
      RwLock
    }
  }
};

static REQWEST_: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);
static LOCALE_CACHE: LazyLock<RwLock<HashMap<String, String>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

#[derive(Serialize)]
struct DeepLRequest {
  text:        Vec<String>,
  target_lang: &'static str
}

#[derive(Deserialize)]
struct DeepLResponse {
  translations: Vec<Translation>
}

#[derive(Deserialize)]
struct DeepLLanguage {
  language: String,
  name:     String
}

#[derive(Deserialize)]
struct DeepLUsage {
  character_count: u64,
  character_limit: u64
}

#[derive(Deserialize)]
struct Translation {
  text:                     String,
  detected_source_language: String
}

fn prettify_nums(num: u64) -> String {
  let mut s = String::new();
  let num_str = num.to_string();
  let len = num_str.len();

  for (i, c) in num_str.chars().enumerate() {
    s.push(c);
    if (len - i - 1) % 3 == 0 && i < len - 1 {
      s.push(',');
    }
  }

  s
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

  if LOCALE_CACHE.read().unwrap().is_empty() {
    update_locale_cache(&deepl_key).await.expect("Failed to update locale cache...");
  }

  ctx.defer().await?;

  let api_url = if deepl_key.ends_with(":fx") {
    "https://api-free.deepl.com"
  } else {
    "https://api.deepl.com"
  };

  let resp = match REQWEST_
    .post(format!("{api_url}/v2/translate"))
    .header("User-Agent", "kon/reqwest")
    .header("Authorization", format!("DeepL-Auth-Key {deepl_key}"))
    .header("Content-Type", "application/json")
    .json(&DeepLRequest {
      text:        vec![content.to_owned()],
      target_lang: "EN"
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
    let quota_info = match get_quota(&deepl_key).await {
      Ok(u) => &format!("-# **Quota: {}/{}**", prettify_nums(u.character_count), prettify_nums(u.character_limit)),
      Err(_) => "-# *Failed to check the quota!*"
    };

    ctx
      .send(
        CreateReply::new().content(
          [
            format!("**Translated from {}**", prettify_lang(translation.detected_source_language.as_str())),
            quota_info.to_string(),
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

async fn update_locale_cache(api_key: &str) -> Result<(), reqwest::Error> {
  let api_url = if api_key.ends_with(":fx") {
    "https://api-free.deepl.com"
  } else {
    "https://api.deepl.com"
  };

  let languages: Vec<DeepLLanguage> = REQWEST_
    .get(format!("{api_url}/v2/languages"))
    .header("User-Agent", "kon/reqwest")
    .header("Authorization", format!("DeepL-Auth-Key {api_key}"))
    .query(&[("type", "target")])
    .send()
    .await?
    .json()
    .await?;

  let mut c = LOCALE_CACHE.write().unwrap();
  for language in languages {
    c.insert(language.language, language.name);
  }

  Ok(())
}

/// List of languages that DeepL supports for translation
static LOCALE_LOOKUP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
  let mut c = HashMap::new();

  c.insert("AR", "Arabic");
  c.insert("BG", "Bulgarian");
  c.insert("CS", "Czech");
  c.insert("DA", "Danish");
  c.insert("DE", "German");
  c.insert("EL", "Greek");
  c.insert("EN", "English");
  c.insert("ES", "Spanish");
  c.insert("ET", "Estonian");
  c.insert("FI", "Finnish");
  c.insert("FR", "French");
  c.insert("HU", "Hungarian");
  c.insert("ID", "Indonesian");
  c.insert("IT", "Italian");
  c.insert("JA", "Japanese");
  c.insert("KO", "Korean");
  c.insert("LT", "Lithuanian");
  c.insert("LV", "Latvian");
  c.insert("NB", "Norwegian Bokmål");
  c.insert("NL", "Dutch");
  c.insert("PL", "Polish");
  c.insert("PT", "Portuguese");
  c.insert("RO", "Romanian");
  c.insert("RU", "Russian");
  c.insert("SK", "Slovak");
  c.insert("SL", "Slovenian");
  c.insert("SV", "Swedish");
  c.insert("TR", "Turkish");
  c.insert("UK", "Ukrainian");
  c.insert("ZH", "Chinese");

  c
});

fn prettify_lang(code: &str) -> String {
  if let Ok(cache) = LOCALE_CACHE.read() {
    if let Some(name) = cache.get(code) {
      return name.clone();
    }
  }

  LOCALE_LOOKUP.get(code).map(|&s| s.to_string()).unwrap_or_else(|| code.to_string())
}

async fn get_quota(api_key: &str) -> Result<DeepLUsage, reqwest::Error> {
  let api_url = if api_key.ends_with(":fx") {
    "https://api-free.deepl.com"
  } else {
    "https://api.deepl.com"
  };

  let usage: DeepLUsage = REQWEST_
    .get(format!("{api_url}/v2/usage"))
    .header("User-Agent", "kon/reqwest")
    .header("Authorization", format!("DeepL-Auth-Key {api_key}"))
    .send()
    .await?
    .json()
    .await?;

  Ok(usage)
}
