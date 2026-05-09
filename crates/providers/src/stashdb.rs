use serde::{Deserialize, Serialize};
use tokimo_core::{CoreError, CoreResult};

const SCENE_SEARCH_QUERY: &str = r#"
query SearchScenes($term: String!) {
  queryScenes(input: { text: $term, per_page: 5 }) {
    scenes {
      id
      title
      date
      duration
      code
      images { url }
      studio { name }
      performers { performer { name } }
      tags { name }
    }
  }
}
"#;

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

#[derive(Debug, Serialize)]
struct GraphqlRequest {
    query: &'static str,
    variables: GraphqlVariables,
}

#[derive(Debug, Serialize)]
struct GraphqlVariables {
    term: String,
}

#[derive(Debug, Deserialize)]
struct GraphqlResponse {
    data: Option<GraphqlData>,
}

#[derive(Debug, Deserialize)]
struct GraphqlData {
    #[serde(rename = "queryScenes")]
    query_scenes: Option<QueryScenes>,
}

#[derive(Debug, Deserialize)]
struct QueryScenes {
    scenes: Option<Vec<StashdbScene>>,
}

#[derive(Debug, Deserialize)]
struct StashdbScene {
    id: String,
    title: Option<String>,
    date: Option<String>,
    duration: Option<i64>,
    code: Option<String>,
    images: Option<Vec<SceneImage>>,
    studio: Option<SceneStudio>,
    performers: Option<Vec<ScenePerformer>>,
    tags: Option<Vec<SceneTag>>,
}

#[derive(Debug, Deserialize)]
struct SceneImage {
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SceneStudio {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ScenePerformer {
    performer: Option<PerformerInner>,
}

#[derive(Debug, Deserialize)]
struct PerformerInner {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SceneTag {
    name: Option<String>,
}

pub async fn search_by_video_id(
    http: &reqwest::Client,
    base_url: &str,
    video_id: &str,
    api_key: Option<&str>,
) -> CoreResult<Option<serde_json::Value>> {
    let id = video_id.trim();
    if id.is_empty() {
        return Ok(None);
    }

    let mut request = http
        .post(base_url.trim_end_matches('/'))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::ACCEPT, "application/json")
        .json(&GraphqlRequest {
            query: SCENE_SEARCH_QUERY,
            variables: GraphqlVariables { term: id.to_string() },
        });

    if let Some(value) = api_key.map(str::trim).filter(|v| !v.is_empty()) {
        request = request.header("ApiKey", value);
    }

    let response = request.send().await.map_err(CoreError::Upstream)?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(CoreError::Provider(format!(
            "stashdb returned status {}: {}",
            status, body
        )));
    }

    let payload: GraphqlResponse = response.json().await.map_err(CoreError::Upstream)?;
    let scenes = payload
        .data
        .and_then(|data| data.query_scenes)
        .and_then(|scenes| scenes.scenes)
        .unwrap_or_default();

    if scenes.is_empty() {
        return Ok(None);
    }

    let scene = find_best_match(&scenes, id);
    scene
        .map(|item| serde_json::to_value(transform_scene(item, id, base_url.trim_end_matches('/'))))
        .transpose()
        .map_err(|e| CoreError::Provider(format!("stashdb serialize error: {e}")))
}

fn find_best_match<'a>(scenes: &'a [StashdbScene], video_id: &str) -> Option<&'a StashdbScene> {
    let normalized = normalize_video_id(video_id);

    for scene in scenes {
        if let Some(code) = scene.code.as_ref() {
            if normalize_video_id(code) == normalized {
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

fn transform_scene(scene: &StashdbScene, video_id: &str, graphql_url: &str) -> AdultMetadata {
    let scene_base = graphql_url.strip_suffix("/graphql").unwrap_or(graphql_url).to_string();

    AdultMetadata {
        video_id: video_id.to_string(),
        title: scene.title.clone(),
        poster_url: scene
            .images
            .as_ref()
            .and_then(|images| images.first())
            .and_then(|image| image.url.clone()),
        cover_url: None,
        source_url: Some(format!("{scene_base}/scenes/{}", scene.id)),
        actors: scene
            .performers
            .as_ref()
            .map(|performers| {
                performers
                    .iter()
                    .filter_map(|item| item.performer.as_ref().and_then(|inner| inner.name.clone()))
                    .collect()
            })
            .filter(|actors: &Vec<String>| !actors.is_empty()),
        genres: scene
            .tags
            .as_ref()
            .map(|tags| tags.iter().filter_map(|item| item.name.clone()).collect())
            .filter(|genres: &Vec<String>| !genres.is_empty()),
        release_date: scene.date.clone(),
        studio: scene.studio.as_ref().and_then(|studio| studio.name.clone()),
        duration: scene.duration.and_then(|value| u32::try_from(value / 60).ok()),
        rating: None,
        source: "stashdb".to_string(),
    }
}

fn normalize_video_id(value: &str) -> String {
    value.trim().to_ascii_uppercase().replace(['-', '_', ' '], "")
}
