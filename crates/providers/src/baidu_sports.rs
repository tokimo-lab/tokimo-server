use serde::{Deserialize, Serialize};
use tokimo_core::{CoreError, CoreResult, SportMatch};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SportSchedule {
    pub matches: Vec<SportMatch>,
}

pub async fn fetch_schedule(http: &reqwest::Client, match_type: &str, date: &str) -> CoreResult<SportSchedule> {
    #[derive(Deserialize)]
    struct Response {
        #[serde(default)]
        data: Option<Data>,
    }

    #[derive(Deserialize)]
    struct Data {
        #[serde(default)]
        list: Vec<MatchItem>,
    }

    #[derive(Deserialize)]
    struct MatchItem {
        #[serde(default)]
        match_name: String,
        #[serde(default)]
        match_time: String,
        #[serde(default)]
        status: String,
        #[serde(default)]
        vs_line: String,
        #[serde(default)]
        left_team: Option<serde_json::Value>,
        #[serde(default)]
        right_team: Option<serde_json::Value>,
        #[serde(default)]
        game: String,
        #[serde(default)]
        link: String,
        #[serde(default)]
        has_live: i32,
    }

    let next_date = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|e| CoreError::Provider(format!("Invalid date format: {}", e)))?
        .succ_opt()
        .ok_or_else(|| CoreError::Provider("Failed to calculate next date".into()))?;

    let request_date = next_date.format("%Y-%m-%d").to_string();

    let url = format!(
        "https://tiyu.baidu.com/al/api/home/schedule?direction=forward&type={}&date={}",
        match_type, request_date
    );

    let resp = http.get(&url).send().await.map_err(CoreError::Upstream)?;

    if !resp.status().is_success() {
        return Err(CoreError::Provider(format!(
            "Baidu Sports API error: {}",
            resp.status()
        )));
    }

    let response: Response = resp.json().await.map_err(CoreError::Upstream)?;

    let matches = response
        .data
        .unwrap_or(Data { list: vec![] })
        .list
        .into_iter()
        .map(|item| SportMatch {
            match_name: item.match_name,
            match_date: date.to_string(),
            start_time: if item.match_time.is_empty() {
                None
            } else {
                Some(item.match_time)
            },
            status: if item.status.is_empty() {
                None
            } else {
                Some(item.status)
            },
            vs_line: if item.vs_line.is_empty() {
                None
            } else {
                Some(item.vs_line)
            },
            left_team: item.left_team,
            right_team: item.right_team,
            game: if item.game.is_empty() { None } else { Some(item.game) },
            link: if item.link.is_empty() { None } else { Some(item.link) },
            has_live: item.has_live != 0,
        })
        .collect();

    Ok(SportSchedule { matches })
}

#[cfg(all(test, feature = "live-api"))]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_baidu_sports() {
        let client = reqwest::Client::new();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let result = fetch_schedule(&client, "hot", &today).await;
        assert!(result.is_ok());
    }
}
