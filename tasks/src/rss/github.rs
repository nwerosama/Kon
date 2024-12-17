use super::{
  IncidentColorMap,
  RSSFeed,
  RSSFeedOutput,
  embed,
  fetch_feed,
  format_html_to_discord,
  get_redis,
  parse,
  save_to_redis,
  task_err,
  task_info,
  trim_old_content
};

use {
  kon_libs::KonResult,
  poise::serenity_prelude::{
    Context,
    Timestamp,
    async_trait
  },
  regex::Regex,
  std::{
    io::Cursor,
    sync::Arc
  }
};

pub struct GitHub {
  url: String
}

impl GitHub {
  pub fn new(url: String) -> Self { Self { url } }
}

#[async_trait]
impl RSSFeed for GitHub {
  fn name(&self) -> &str { "GitHub" }

  fn url(&self) -> &str { self.url.as_str() }

  async fn process(
    &self,
    _ctx: Arc<Context>
  ) -> KonResult<Option<RSSFeedOutput>> {
    let redis = get_redis().await;
    let rkey = "RSS_GitHub";
    let rkey_content = format!("{rkey}_Content");

    let res = fetch_feed(self.url()).await?;
    let data = res.text().await?;
    let cursor = Cursor::new(data);

    let feed = parse(cursor).map_err(|e| {
      task_err("RSS:GitHub", &format!("Error parsing RSS feed: {e}"));
      e
    })?;

    if feed.entries.is_empty() {
      task_err("RSS:GitHub", "No entries found in the feed!");
      return Ok(None);
    }

    let incident_page = feed.entries[0].links[0].clone().href;
    let article = feed.entries[0].clone();

    fn get_incident_id(input: &str) -> Option<String> {
      let re = Regex::new(r#"/incidents/([a-zA-Z0-9]+)$"#).unwrap();
      re.captures(input).map(|caps| caps[1].to_string())
    }

    let cached_incident = redis.get(rkey).await.unwrap().unwrap_or_default();
    let new_content = format_html_to_discord(article.content.unwrap().body.unwrap());

    let update_patt = Regex::new(r"(?i)\bupdate\b").unwrap();
    let investigating_patt = Regex::new(r"(?i)\binvestigating\b").unwrap();
    let resolved_patt = Regex::new(r"(?i)\bresolved\b").unwrap();
    let date_patt = Regex::new(r"\b[A-Z][a-z]{2} \d{2}, \d{2}:\d{2} UTC\b").unwrap();

    let first_entry = date_patt
      .split(&new_content)
      .map(str::trim)
      .find(|e| !e.is_empty())
      .unwrap_or(&new_content);

    let color: u32 = if update_patt.is_match(first_entry) {
      IncidentColorMap::Update.color()
    } else if investigating_patt.is_match(first_entry) {
      IncidentColorMap::Investigating.color()
    } else if resolved_patt.is_match(first_entry) {
      IncidentColorMap::Resolved.color()
    } else {
      IncidentColorMap::Default.color()
    };

    task_info("RSS:GitHub:Debug", &format!("Checking cache for incident ID: {}", &article.links[0].href));
    if cached_incident.is_empty() {
      save_to_redis(rkey, &get_incident_id(&article.links[0].href).unwrap()).await?;
      save_to_redis(&rkey_content, &new_content).await?;
      return Ok(None);
    }

    if let Some(incident) = get_incident_id(&article.links[0].href) {
      if incident == cached_incident {
        let cached_content = redis.get(&rkey_content).await.unwrap().unwrap_or_default();
        if cached_content == new_content {
          Ok(None)
        } else {
          redis.set(&rkey_content, &new_content).await.unwrap();
          redis.expire(&rkey_content, 21600).await.unwrap();
          task_info("RSS:GitHub:Debug", "Incident added in cache and preparing to send embed to Discord");

          Ok(Some(RSSFeedOutput::IncidentEmbed(embed(
            color,
            article.title.unwrap().content,
            incident_page,
            trim_old_content(&new_content),
            Timestamp::from(article.updated.unwrap())
          ))))
        }
      } else {
        save_to_redis(rkey, &incident).await?;
        redis.set(&rkey_content, &new_content).await.unwrap();
        task_info("RSS:GitHub:Debug", "Incident updated in cache and preparing to send embed to Discord");

        Ok(Some(RSSFeedOutput::IncidentEmbed(embed(
          color,
          article.title.unwrap().content,
          incident_page,
          trim_old_content(&new_content),
          Timestamp::from(article.updated.unwrap())
        ))))
      }
    } else {
      task_err(
        "RSS:GitHub",
        &format!("Incident ID does not match the expected RegEx pattern! ({})", &article.links[0].href)
      );
      Ok(None)
    }
  }
}
