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
    let locale = "zh";
    let search_url = format!(
        "{}/search?q={}&locale={locale}",
        normalized_base,
        urlencoding::encode(id)
    );

    let final_cookie = compose_cookie(cookie);

    let search_response = http
        .get(&search_url)
        .headers(build_headers(final_cookie.as_deref())?)
        .send()
        .await
        .map_err(CoreError::Upstream)?;

    let search_status = search_response.status();
    let search_body = search_response.text().await.map_err(CoreError::Upstream)?;

    if is_challenge_page(&search_body) {
        return Err(CoreError::Provider(
            "javdb anti-bot challenge detected on search page; provide JAVDB_COOKIE or verify upstream access"
                .to_string(),
        ));
    }

    if !search_status.is_success() {
        return Err(CoreError::Provider(format!(
            "javdb search returned status {}",
            search_status
        )));
    }

    let Some(detail_path) = extract_detail_path(&search_body, id)? else {
        return Ok(None);
    };

    let detail_url = format!("{}{}?locale={locale}", normalized_base, detail_path);
    let detail_response = http
        .get(&detail_url)
        .headers(build_headers(final_cookie.as_deref())?)
        .send()
        .await
        .map_err(CoreError::Upstream)?;

    let detail_status = detail_response.status();
    let detail_body = detail_response.text().await.map_err(CoreError::Upstream)?;

    if is_challenge_page(&detail_body) {
        return Err(CoreError::Provider(
            "javdb anti-bot challenge detected on detail page; provide JAVDB_COOKIE or verify upstream access"
                .to_string(),
        ));
    }

    if detail_status == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }

    if !detail_status.is_success() {
        return Err(CoreError::Provider(format!(
            "javdb detail returned status {}",
            detail_status
        )));
    }

    let parsed = parse_detail_page(&detail_body, id, &detail_url, normalized_base)?;
    parsed
        .map(serde_json::to_value)
        .transpose()
        .map_err(|e| CoreError::Provider(format!("javdb serialize error: {e}")))
}

fn compose_cookie(cookie: Option<&str>) -> Option<String> {
    let base = "over18=1; locale=zh";
    match cookie.map(str::trim).filter(|v| !v.is_empty()) {
        Some(raw) => Some(format!("{base}; {raw}")),
        None => Some(base.to_string()),
    }
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
            .map_err(|e| CoreError::Provider(format!("javdb invalid cookie header: {e}")))?;
        headers.insert(reqwest::header::COOKIE, value);
    }

    Ok(headers)
}

fn extract_detail_path(html: &str, video_id: &str) -> CoreResult<Option<String>> {
    let document = Html::parse_document(html);
    let normalized = normalize_video_id(video_id);

    let link_sel = parse_selector(".movie-list .item a, .grid-item a")?;
    let uid_sel = parse_selector(".uid, .video-title strong")?;

    let mut first_href: Option<String> = None;

    for element in document.select(&link_sel) {
        if first_href.is_none() {
            first_href = element.value().attr("href").map(str::to_string);
        }

        let uid = element
            .select(&uid_sel)
            .next()
            .map(|u| normalize_video_id(&u.text().collect::<String>()));

        if uid.as_deref() == Some(normalized.as_str()) {
            if let Some(href) = element.value().attr("href") {
                return Ok(Some(href.to_string()));
            }
        }
    }

    Ok(first_href)
}

fn parse_detail_page(
    html: &str,
    video_id: &str,
    source_url: &str,
    base_url: &str,
) -> CoreResult<Option<AdultMetadata>> {
    let document = Html::parse_document(html);

    let title_sel_primary = parse_selector("h2.title strong.current-title")?;
    let title_sel_fallback = parse_selector("h2.title")?;
    let title = document
        .select(&title_sel_primary)
        .next()
        .or_else(|| document.select(&title_sel_fallback).next())
        .map(|el| el.text().collect::<String>().trim().to_string())
        .filter(|v| !v.is_empty());

    let Some(raw_title) = title else {
        return Ok(None);
    };

    let poster_sel = parse_selector(".video-cover img, .column-video-cover img")?;
    let poster_url = document
        .select(&poster_sel)
        .next()
        .and_then(|el| el.value().attr("src"))
        .map(|url| resolve_url(url, base_url));

    let rating_sel = parse_selector(".score .value")?;
    let rating = document
        .select(&rating_sel)
        .next()
        .and_then(|el| el.text().collect::<String>().trim().parse::<f64>().ok());

    let panel_sel = parse_selector(".movie-panel-info .panel-block, .video-meta-panel .panel-block")?;
    let strong_sel = parse_selector("strong, .header")?;
    let value_sel = parse_selector(".value, span:not(.header)")?;
    let link_sel = parse_selector("a")?;

    let date_re =
        Regex::new(r"\d{4}-\d{2}-\d{2}").map_err(|e| CoreError::Provider(format!("javdb date regex error: {e}")))?;
    let duration_re =
        Regex::new(r"(\d+)").map_err(|e| CoreError::Provider(format!("javdb duration regex error: {e}")))?;

    let mut actors: Vec<String> = Vec::new();
    let mut genres: Vec<String> = Vec::new();
    let mut release_date = None;
    let mut studio = None;
    let mut duration = None;

    for panel in document.select(&panel_sel) {
        let label = panel
            .select(&strong_sel)
            .next()
            .map(|s| s.text().collect::<String>())
            .unwrap_or_default();
        let value = panel
            .select(&value_sel)
            .next()
            .map(|s| s.text().collect::<String>())
            .unwrap_or_default();

        if label.contains("日期") || label.contains("Date") {
            release_date = date_re.find(&value).map(|m| m.as_str().to_string());
            continue;
        }

        if label.contains("片商") || label.contains("Maker") {
            studio = panel
                .select(&link_sel)
                .next()
                .map(|a| a.text().collect::<String>().trim().to_string())
                .filter(|v| !v.is_empty());
            continue;
        }

        if label.contains("時長") || label.contains("时间") || label.contains("Duration") {
            duration = duration_re
                .captures(&value)
                .and_then(|cap| cap.get(1))
                .and_then(|m| m.as_str().parse::<u32>().ok());
            continue;
        }

        if label.contains("類別") || label.contains("类别") || label.contains("Genre") {
            for link in panel.select(&link_sel) {
                let genre = link.text().collect::<String>().trim().to_string();
                if !genre.is_empty() {
                    genres.push(genre);
                }
            }
            continue;
        }

        if label.contains("演員") || label.contains("演员") || label.contains("Actor") {
            for link in panel.select(&link_sel) {
                let actor = link.text().collect::<String>().trim().to_string();
                if !actor.is_empty() {
                    actors.push(actor);
                }
            }
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
        cover_url: None,
        source_url: Some(source_url.to_string()),
        actors: if actors.is_empty() { None } else { Some(actors) },
        genres: if genres.is_empty() { None } else { Some(genres) },
        release_date,
        studio,
        duration,
        rating,
        source: "javdb".to_string(),
    }))
}

fn parse_selector(selector: &str) -> CoreResult<Selector> {
    Selector::parse(selector).map_err(|e| CoreError::Provider(format!("javdb selector parse error '{selector}': {e}")))
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

fn normalize_video_id(value: &str) -> String {
    value.trim().to_ascii_uppercase().replace(['-', '_', ' '], "")
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
