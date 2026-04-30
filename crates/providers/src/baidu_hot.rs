use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{CoreError, CoreResult, HotItem};

#[async_trait]
pub trait HotSource: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    async fn fetch(&self, http: &reqwest::Client) -> CoreResult<Vec<HotItem>>;
}

pub struct WeiboHotSource;

#[async_trait]
impl HotSource for WeiboHotSource {
    fn id(&self) -> &'static str {
        "weibo"
    }

    fn name(&self) -> &'static str {
        "微博热搜"
    }

    async fn fetch(&self, http: &reqwest::Client) -> CoreResult<Vec<HotItem>> {
        #[derive(Deserialize)]
        struct Response {
            data: Data,
        }

        #[derive(Deserialize)]
        struct Data {
            realtime: Vec<RealtimeItem>,
        }

        #[derive(Deserialize)]
        struct RealtimeItem {
            word: String,
            #[serde(default)]
            num: Option<i64>,
            #[serde(default)]
            label_name: String,
        }

        let url = "https://weibo.com/ajax/side/hotSearch";
        let resp = http.get(url).send().await.map_err(CoreError::Upstream)?;

        if !resp.status().is_success() {
            return Err(CoreError::Provider(format!("Weibo API error: {}", resp.status())));
        }

        let data: Response = resp.json().await.map_err(CoreError::Upstream)?;

        let items = data
            .data
            .realtime
            .into_iter()
            .map(|item| HotItem {
                title: item.word.clone(),
                url: format!("https://s.weibo.com/weibo?q=%23{}%23", urlencoding::encode(&item.word)),
                hot_value: item.num.map(|n| n.to_string()),
                label: if item.label_name.is_empty() {
                    None
                } else {
                    Some(item.label_name)
                },
            })
            .collect();

        Ok(items)
    }
}

pub struct BilibiliHotSource;

#[async_trait]
impl HotSource for BilibiliHotSource {
    fn id(&self) -> &'static str {
        "bilibili"
    }

    fn name(&self) -> &'static str {
        "B站热门"
    }

    async fn fetch(&self, http: &reqwest::Client) -> CoreResult<Vec<HotItem>> {
        #[derive(Deserialize)]
        struct Response {
            data: Data,
        }

        #[derive(Deserialize)]
        struct Data {
            list: Vec<VideoItem>,
        }

        #[derive(Deserialize)]
        struct VideoItem {
            title: String,
            short_link_v2: String,
            #[serde(default)]
            stat: Stat,
        }

        #[derive(Deserialize, Default)]
        struct Stat {
            #[serde(default)]
            view: i64,
        }

        let url = "https://api.bilibili.com/x/web-interface/popular";
        let resp = http.get(url).send().await.map_err(CoreError::Upstream)?;

        if !resp.status().is_success() {
            return Err(CoreError::Provider(format!("Bilibili API error: {}", resp.status())));
        }

        let data: Response = resp.json().await.map_err(CoreError::Upstream)?;

        let items = data
            .data
            .list
            .into_iter()
            .map(|item| HotItem {
                title: item.title,
                url: item.short_link_v2,
                hot_value: Some(item.stat.view.to_string()),
                label: None,
            })
            .collect();

        Ok(items)
    }
}

pub struct BaiduHotSource;

#[async_trait]
impl HotSource for BaiduHotSource {
    fn id(&self) -> &'static str {
        "baidu"
    }

    fn name(&self) -> &'static str {
        "百度热搜"
    }

    async fn fetch(&self, http: &reqwest::Client) -> CoreResult<Vec<HotItem>> {
        #[derive(Deserialize)]
        struct Response {
            data: Data,
        }

        #[derive(Deserialize)]
        struct Data {
            cards: Vec<Card>,
        }

        #[derive(Deserialize)]
        struct Card {
            content: Vec<ContentItem>,
        }

        #[derive(Deserialize)]
        struct ContentItem {
            word: String,
            #[serde(default)]
            url: String,
            #[serde(default)]
            hot_score: String,
        }

        let url = "https://top.baidu.com/api/board?platform=wise&tab=realtime";
        let resp = http.get(url).send().await.map_err(CoreError::Upstream)?;

        if !resp.status().is_success() {
            return Err(CoreError::Provider(format!("Baidu API error: {}", resp.status())));
        }

        let data: Response = resp.json().await.map_err(CoreError::Upstream)?;

        let mut items = Vec::new();
        for card in data.data.cards {
            for content in card.content {
                let word = content.word.clone();
                items.push(HotItem {
                    title: content.word,
                    url: if content.url.is_empty() {
                        format!("https://www.baidu.com/s?wd={}", urlencoding::encode(&word))
                    } else {
                        content.url
                    },
                    hot_value: if content.hot_score.is_empty() {
                        None
                    } else {
                        Some(content.hot_score)
                    },
                    label: None,
                });
            }
        }

        Ok(items)
    }
}

pub struct GithubTrendingSource;

#[async_trait]
impl HotSource for GithubTrendingSource {
    fn id(&self) -> &'static str {
        "github"
    }

    fn name(&self) -> &'static str {
        "GitHub Trending"
    }

    async fn fetch(&self, http: &reqwest::Client) -> CoreResult<Vec<HotItem>> {
        let url = "https://github.com/trending";
        let resp = http.get(url).send().await.map_err(CoreError::Upstream)?;

        if !resp.status().is_success() {
            return Err(CoreError::Provider(format!("GitHub error: {}", resp.status())));
        }

        let html = resp.text().await.map_err(CoreError::Upstream)?;
        let document = scraper::Html::parse_document(&html);
        let selector = scraper::Selector::parse("article.Box-row")
            .map_err(|_| CoreError::Provider("Failed to parse GitHub trending page selector".into()))?;

        let mut items = Vec::new();

        for element in document.select(&selector) {
            let h2_selector = scraper::Selector::parse("h2 a")
                .map_err(|_| CoreError::Provider("Failed to parse h2 selector".into()))?;

            if let Some(link_elem) = element.select(&h2_selector).next() {
                let path = link_elem.value().attr("href").unwrap_or("");
                let title = link_elem.text().collect::<String>().trim().to_string();

                items.push(HotItem {
                    title,
                    url: format!("https://github.com{}", path),
                    hot_value: None,
                    label: None,
                });
            }
        }

        Ok(items)
    }
}

pub struct HackerNewsSource;

#[async_trait]
impl HotSource for HackerNewsSource {
    fn id(&self) -> &'static str {
        "hackernews"
    }

    fn name(&self) -> &'static str {
        "Hacker News"
    }

    async fn fetch(&self, http: &reqwest::Client) -> CoreResult<Vec<HotItem>> {
        #[derive(Deserialize)]
        struct Story {
            title: String,
            #[serde(default)]
            url: Option<String>,
            id: i64,
        }

        let top_url = "https://hacker-news.firebaseio.com/v0/topstories.json";
        let resp = http.get(top_url).send().await.map_err(CoreError::Upstream)?;

        if !resp.status().is_success() {
            return Err(CoreError::Provider(format!("HN API error: {}", resp.status())));
        }

        let ids: Vec<i64> = resp.json().await.map_err(CoreError::Upstream)?;

        let mut items = Vec::new();
        for id in ids.iter().take(30) {
            let story_url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id);
            let story_resp = http.get(&story_url).send().await.map_err(CoreError::Upstream)?;

            if story_resp.status().is_success() {
                if let Ok(story) = story_resp.json::<Story>().await {
                    items.push(HotItem {
                        title: story.title,
                        url: story
                            .url
                            .unwrap_or_else(|| format!("https://news.ycombinator.com/item?id={}", story.id)),
                        hot_value: None,
                        label: None,
                    });
                }
            }
        }

        Ok(items)
    }
}

pub struct V2exSource;

#[async_trait]
impl HotSource for V2exSource {
    fn id(&self) -> &'static str {
        "v2ex"
    }

    fn name(&self) -> &'static str {
        "V2EX"
    }

    async fn fetch(&self, http: &reqwest::Client) -> CoreResult<Vec<HotItem>> {
        #[derive(Deserialize)]
        struct Topic {
            title: String,
            url: String,
        }

        let url = "https://www.v2ex.com/api/topics/hot.json";
        let resp = http.get(url).send().await.map_err(CoreError::Upstream)?;

        if !resp.status().is_success() {
            return Err(CoreError::Provider(format!("V2EX API error: {}", resp.status())));
        }

        let topics: Vec<Topic> = resp.json().await.map_err(CoreError::Upstream)?;

        let items = topics
            .into_iter()
            .map(|topic| HotItem {
                title: topic.title,
                url: topic.url,
                hot_value: None,
                label: None,
            })
            .collect();

        Ok(items)
    }
}

pub fn create_registry() -> Vec<std::sync::Arc<dyn HotSource>> {
    vec![
        std::sync::Arc::new(WeiboHotSource),
        std::sync::Arc::new(BilibiliHotSource),
        std::sync::Arc::new(BaiduHotSource),
        std::sync::Arc::new(GithubTrendingSource),
        std::sync::Arc::new(HackerNewsSource),
        std::sync::Arc::new(V2exSource),
    ]
}

#[cfg(all(test, feature = "live-api"))]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_weibo_source() {
        let client = reqwest::Client::new();
        let source = WeiboHotSource;
        let result = source.fetch(&client).await;
        assert!(result.is_ok());
    }
}
