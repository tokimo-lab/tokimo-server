use regex::Regex;
use scraper::{Html, Selector};
use serde::Serialize;
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

pub async fn search_by_video_id(
    http: &reqwest::Client,
    base_url: &str,
    video_id: &str,
    cookie: Option<&str>,
) -> CoreResult<Option<serde_json::Value>> {
    let id = video_id.trim();
    if id.is_empty() {
        return Ok(None);
    }

    let normalized_base = base_url.trim_end_matches('/');
    let url = format!("{}/{}", normalized_base, urlencoding::encode(id));

    let response = http
        .get(&url)
        .headers(build_headers(cookie)?)
        .send()
        .await
        .map_err(CoreError::Upstream)?;

    let status = response.status();
    let body = response.text().await.map_err(CoreError::Upstream)?;

    if is_challenge_page(&body) {
        return Err(CoreError::Provider(
            "javbus anti-bot challenge detected; provide JAVBUS_COOKIE or verify upstream access".to_string(),
        ));
    }

    if status == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }

    if !status.is_success() {
        return Err(CoreError::Provider(format!("javbus returned status {}", status)));
    }

    let parsed = parse_detail_page(&body, id, &url, normalized_base)?;
    parsed
        .map(serde_json::to_value)
        .transpose()
        .map_err(|e| CoreError::Provider(format!("javbus serialize error: {e}")))
}

fn build_headers(cookie: Option<&str>) -> CoreResult<reqwest::header::HeaderMap> {
    use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, USER_AGENT};

    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36",
        ),
    );
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
    );
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("zh-CN,zh;q=0.9,en;q=0.8"));

    if let Some(raw_cookie) = cookie.map(str::trim).filter(|v| !v.is_empty()) {
        let value = HeaderValue::from_str(raw_cookie)
            .map_err(|e| CoreError::Provider(format!("javbus invalid cookie header: {e}")))?;
        headers.insert(reqwest::header::COOKIE, value);
    }

    Ok(headers)
}

fn parse_detail_page(
    html: &str,
    video_id: &str,
    source_url: &str,
    base_url: &str,
) -> CoreResult<Option<AdultMetadata>> {
    let document = Html::parse_document(html);

    let h3_sel = parse_selector("h3")?;
    let title = document
        .select(&h3_sel)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .filter(|v| !v.is_empty());

    let Some(raw_title) = title else {
        return Ok(None);
    };

    let big_image_sel = parse_selector("a.bigImage")?;
    let big_img_sel = parse_selector("a.bigImage img")?;
    let raw_cover = document
        .select(&big_image_sel)
        .next()
        .and_then(|el| el.value().attr("href"))
        .or_else(|| {
            document
                .select(&big_img_sel)
                .next()
                .and_then(|el| el.value().attr("src"))
        });
    let cover_url = raw_cover.map(|url| resolve_url(url, base_url));
    let poster_url = cover_url
        .as_ref()
        .map(|url| url.replace("/pics/cover/", "/pics/thumb/").replace("_b.jpg", ".jpg"));

    let actor_sel = parse_selector(".star-name a")?;
    let actors: Vec<String> = document
        .select(&actor_sel)
        .filter_map(|el| {
            let name = el.text().collect::<String>().trim().to_string();
            if name.is_empty() {
                None
            } else {
                Some(name)
            }
        })
        .collect();

    let genre_sel = parse_selector(r#"span.genre a[href*='genre']"#)?;
    let genres: Vec<String> = document
        .select(&genre_sel)
        .filter_map(|el| {
            let name = el.text().collect::<String>().trim().to_string();
            if name.is_empty() {
                None
            } else {
                Some(name)
            }
        })
        .collect();

    let info_sel = parse_selector(".info p")?;
    let link_sel = parse_selector("a")?;
    let date_re =
        Regex::new(r"\d{4}-\d{2}-\d{2}").map_err(|e| CoreError::Provider(format!("javbus date regex error: {e}")))?;
    let duration_re =
        Regex::new(r"(\d+)").map_err(|e| CoreError::Provider(format!("javbus duration regex error: {e}")))?;

    let mut release_date = None;
    let mut studio = None;
    let mut duration = None;

    for el in document.select(&info_sel) {
        let text = el.text().collect::<String>();
        if (text.contains("發行日期") || text.contains("发行日期")) && release_date.is_none() {
            release_date = date_re.find(&text).map(|m| m.as_str().to_string());
        }
        if (text.contains("製作商") || text.contains("制作商")) && studio.is_none() {
            studio = el
                .select(&link_sel)
                .next()
                .map(|a| a.text().collect::<String>().trim().to_string())
                .filter(|v| !v.is_empty());
        }
        if (text.contains("長度") || text.contains("长度")) && duration.is_none() {
            duration = duration_re
                .captures(&text)
                .and_then(|cap| cap.get(1))
                .and_then(|m| m.as_str().parse::<u32>().ok());
        }
    }

    let title_without_code = raw_title.replace(video_id, "").trim().to_string();

    Ok(Some(AdultMetadata {
        video_id: video_id.to_string(),
        title: if title_without_code.is_empty() {
            Some(raw_title)
        } else {
            Some(title_without_code)
        },
        poster_url,
        cover_url,
        source_url: Some(source_url.to_string()),
        actors: if actors.is_empty() { None } else { Some(actors) },
        genres: if genres.is_empty() { None } else { Some(genres) },
        release_date,
        studio,
        duration,
        rating: None,
        source: "javbus".to_string(),
    }))
}

fn parse_selector(selector: &str) -> CoreResult<Selector> {
    Selector::parse(selector).map_err(|e| CoreError::Provider(format!("javbus selector parse error '{selector}': {e}")))
}

fn resolve_url(url: &str, base_url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        return url.to_string();
    }
    if url.starts_with('/') {
        return format!("{base_url}{url}");
    }
    format!("{base_url}/{url}")
}

fn is_challenge_page(body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    [
        "cf-browser-verification",
        "cf-challenge",
        "just a moment",
        "checking your browser",
        "cloudflare",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}
