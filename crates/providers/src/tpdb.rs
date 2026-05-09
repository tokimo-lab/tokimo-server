use serde::{Deserialize, Serialize};
use tokimo_core::{CoreError, CoreResult};

#[derive(Debug, Clone, Serialize)]
struct AdultMetadata {
    video_id: String,
    title: Option<String>,
    poster_url: Option<String>,
    cover_url: Option<String>,
    source_url: Option<String>,
    actors: Option<Vec<String>>,
    genres: Option<Vec<String>>,
    release_date: Option<String>,
    studio: Option<String>,
    duration: Option<u32>,
    rating: Option<f64>,
    source: String,
}

#[derive(Debug, Deserialize)]
struct TpdbSearchResponse {
    data: Vec<TpdbScene>,
}

#[derive(Debug, Deserialize)]
struct TpdbScene {
    id: i64,
    title: Option<String>,
    date: Option<String>,
    duration: Option<i64>,
    poster: Option<String>,
    background: Option<TpdbBackground>,
    site: Option<TpdbSite>,
    performers: Option<Vec<TpdbPerformer>>,
    tags: Option<Vec<TpdbTag>>,
    external_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TpdbBackground {
    full: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TpdbSite {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TpdbPerformer {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TpdbTag {
    name: Option<String>,
}

pub async fn search_by_video_id(
    http: &reqwest::Client,
    api_key: &str,
    base_url: &str,
    video_id: &str,
) -> CoreResult<Option<serde_json::Value>> {
    let id = video_id.trim();
    if id.is_empty() {
        return Ok(None);
    }

    let mut url = reqwest::Url::parse(&format!("{}/scenes", base_url.trim_end_matches('/')))
        .map_err(|e| CoreError::Provider(format!("tpdb invalid base URL: {e}")))?;
    url.query_pairs_mut().append_pair("q", id).append_pair("per_page", "5");

    let response = http
        .get(url)
        .header("Authorization", format!("Bearer {}", api_key.trim()))
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await
        .map_err(CoreError::Upstream)?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(CoreError::Provider(format!(
            "tpdb returned status {}: {}",
            status, body
        )));
    }

    let payload: TpdbSearchResponse = response.json().await.map_err(CoreError::Upstream)?;
    if payload.data.is_empty() {
        return Ok(None);
    }

    let scene = find_best_match(&payload.data, id);
    scene
        .map(|item| serde_json::to_value(transform_scene(item, id)))
        .transpose()
        .map_err(|e| CoreError::Provider(format!("tpdb serialize error: {e}")))
}

fn find_best_match<'a>(scenes: &'a [TpdbScene], video_id: &str) -> Option<&'a TpdbScene> {
    let normalized = normalize_video_id(video_id);

    for scene in scenes {
        if let Some(external_id) = scene.external_id.as_ref() {
            if normalize_video_id(external_id) == normalized {
                return Some(scene);
            }
        }
    }

    for scene in scenes {
        if let Some(title) = scene.title.as_ref() {
            if normalize_video_id(title).contains(&normalized) {
                return Some(scene);
            }
        }
    }

    scenes.first()
}

fn transform_scene(scene: &TpdbScene, video_id: &str) -> AdultMetadata {
    AdultMetadata {
        video_id: video_id.to_string(),
        title: scene.title.clone(),
        poster_url: scene
            .poster
            .clone()
            .or_else(|| scene.background.as_ref().and_then(|bg| bg.full.clone())),
        cover_url: None,
        source_url: Some(format!("https://theporndb.net/scenes/{}", scene.id)),
        actors: scene
            .performers
            .as_ref()
            .map(|performers| performers.iter().filter_map(|item| item.name.clone()).collect())
            .filter(|actors: &Vec<String>| !actors.is_empty()),
        genres: scene
            .tags
            .as_ref()
            .map(|tags| tags.iter().filter_map(|item| item.name.clone()).collect())
            .filter(|genres: &Vec<String>| !genres.is_empty()),
        release_date: scene.date.clone(),
        studio: scene.site.as_ref().and_then(|site| site.name.clone()),
        duration: scene.duration.and_then(|value| u32::try_from(value / 60).ok()),
        rating: None,
        source: "tpdb".to_string(),
    }
}

fn normalize_video_id(value: &str) -> String {
    value.trim().to_ascii_uppercase().replace(['-', '_', ' '], "")
}
