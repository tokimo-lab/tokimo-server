use std::time::Duration;

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

pub struct ToutiaoHotSource;

#[async_trait]
impl HotSource for ToutiaoHotSource {
    fn id(&self) -> &'static str {
        "toutiao"
    }

    fn name(&self) -> &'static str {
        "今日头条"
    }

    async fn fetch(&self, http: &reqwest::Client) -> CoreResult<Vec<HotItem>> {
        #[derive(Deserialize)]
        struct ToutiaoResp {
            data: Vec<ToutiaoItem>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "PascalCase")]
        struct ToutiaoItem {
            title: String,
            #[serde(default)]
            hot_value: String,
            #[serde(default)]
            url: String,
            #[serde(default)]
            cluster_id_str: String,
        }

        let resp = http
            .get("https://www.toutiao.com/hot-event/hot-board/?origin=toutiao_pc")
            .header(
                "User-Agent",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
            )
            .header("Accept", "application/json, text/plain, */*")
            .header("Referer", "https://www.toutiao.com/")
            .timeout(Duration::from_secs(15))
            .send()
            .await
            .map_err(CoreError::Upstream)?;

        if !resp.status().is_success() {
            return Err(CoreError::Provider(format!("Toutiao API error: {}", resp.status())));
        }

        let data: ToutiaoResp = resp.json().await.map_err(CoreError::Upstream)?;

        let items = data
            .data
            .into_iter()
            .map(|item| {
                let title = item.title;
                let url = if item.url.is_empty() {
                    if item.cluster_id_str.is_empty() {
                        format!("https://www.toutiao.com/search?keyword={}", urlencoding::encode(&title))
                    } else {
                        format!("https://www.toutiao.com/trending/{}/", item.cluster_id_str)
                    }
                } else {
                    item.url
                };

                HotItem {
                    title,
                    url,
                    hot_value: if item.hot_value.is_empty() {
                        None
                    } else {
                        Some(item.hot_value)
                    },
                    label: None,
                }
            })
            .take(50)
            .collect();

        Ok(items)
    }
}

pub struct KrHotSource;

#[async_trait]
impl HotSource for KrHotSource {
    fn id(&self) -> &'static str {
        "36kr"
    }

    fn name(&self) -> &'static str {
        "36氪"
    }

    async fn fetch(&self, http: &reqwest::Client) -> CoreResult<Vec<HotItem>> {
        #[derive(Deserialize)]
        struct KrResp {
            data: KrData,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct KrData {
            hot_rank_list: Vec<KrRankItem>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct KrRankItem {
            item_id: u64,
            template_material: KrMaterial,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct KrMaterial {
            widget_title: String,
            #[serde(default)]
            stat_format: String,
        }

        let body = serde_json::json!({"partner_id": "wap", "param": {"siteId": 1, "platformId": 2}});
        let resp = http
            .post("https://gateway.36kr.com/api/mis/nav/home/nav/rank/hot")
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
            .header("Referer", "https://36kr.com/")
            .json(&body)
            .timeout(Duration::from_secs(15))
            .send()
            .await
            .map_err(CoreError::Upstream)?;

        if !resp.status().is_success() {
            return Err(CoreError::Provider(format!("36kr API error: {}", resp.status())));
        }

        let data: KrResp = resp.json().await.map_err(CoreError::Upstream)?;

        let items = data
            .data
            .hot_rank_list
            .into_iter()
            .map(|item| HotItem {
                title: item.template_material.widget_title,
                url: format!("https://36kr.com/p/{}", item.item_id),
                hot_value: if item.template_material.stat_format.is_empty() {
                    None
                } else {
                    Some(item.template_material.stat_format)
                },
                label: None,
            })
            .take(50)
            .collect();

        Ok(items)
    }
}

pub struct JuejinHotSource;

#[async_trait]
impl HotSource for JuejinHotSource {
    fn id(&self) -> &'static str {
        "juejin"
    }

    fn name(&self) -> &'static str {
        "掘金"
    }

    async fn fetch(&self, http: &reqwest::Client) -> CoreResult<Vec<HotItem>> {
        #[derive(Deserialize)]
        struct JuejinResp {
            data: Vec<JuejinEntry>,
        }

        #[derive(Deserialize)]
        struct JuejinEntry {
            #[serde(default)]
            item_info: Option<JuejinItemInfo>,
        }

        #[derive(Deserialize)]
        struct JuejinItemInfo {
            #[serde(default)]
            article_info: Option<JuejinArticle>,
        }

        #[derive(Deserialize)]
        struct JuejinArticle {
            article_id: String,
            title: String,
            #[serde(default)]
            view_count: u64,
        }

        let resp = http
            .post("https://api.juejin.cn/recommend_api/v1/article/recommend_all_feed?aid=2608&uuid=7180000000000001")
            .header(
                "User-Agent",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
            )
            .header("Content-Type", "application/json")
            .header("Referer", "https://juejin.cn/")
            .json(&serde_json::json!({"id_type":2,"sort_type":200,"cursor":"0","limit":50,"client_type":2608}))
            .timeout(Duration::from_secs(15))
            .send()
            .await
            .map_err(CoreError::Upstream)?;

        if !resp.status().is_success() {
            return Err(CoreError::Provider(format!("Juejin API error: {}", resp.status())));
        }

        let data: JuejinResp = resp.json().await.map_err(CoreError::Upstream)?;

        let items = data
            .data
            .into_iter()
            .filter_map(|entry| {
                let article = entry.item_info.and_then(|ii| ii.article_info)?;
                Some(HotItem {
                    title: article.title,
                    url: format!("https://juejin.cn/post/{}", article.article_id),
                    hot_value: if article.view_count > 0 {
                        Some(format!("{}阅读", format_score(article.view_count)))
                    } else {
                        None
                    },
                    label: None,
                })
            })
            .collect();

        Ok(items)
    }
}

pub struct SspaiHotSource;

#[async_trait]
impl HotSource for SspaiHotSource {
    fn id(&self) -> &'static str {
        "sspai"
    }

    fn name(&self) -> &'static str {
        "少数派"
    }

    async fn fetch(&self, http: &reqwest::Client) -> CoreResult<Vec<HotItem>> {
        let resp = http
            .get("https://sspai.com/feed")
            .header(
                "User-Agent",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
            )
            .header("Accept", "application/rss+xml, application/xml, text/xml")
            .timeout(Duration::from_secs(15))
            .send()
            .await
            .map_err(CoreError::Upstream)?;

        if !resp.status().is_success() {
            return Err(CoreError::Provider(format!("Sspai API error: {}", resp.status())));
        }

        let xml = resp.text().await.map_err(CoreError::Upstream)?;
        let mut items = Vec::new();
        let mut remaining = xml.as_str();

        while let Some(start) = remaining.find("<item>") {
            remaining = &remaining[start + 6..];
            let end = remaining.find("</item>").unwrap_or(remaining.len());
            let block = &remaining[..end];
            let title = extract_xml_text(block, "title");
            let url = extract_xml_text(block, "link");

            if !title.is_empty() && !url.is_empty() {
                items.push(HotItem {
                    title,
                    url,
                    hot_value: None,
                    label: None,
                });
            }

            if items.len() >= 30 {
                break;
            }
            remaining = &remaining[end..];
        }

        Ok(items)
    }
}

pub struct ZhihuHotSource;

#[async_trait]
impl HotSource for ZhihuHotSource {
    fn id(&self) -> &'static str {
        "zhihu"
    }

    fn name(&self) -> &'static str {
        "知乎热榜"
    }

    async fn fetch(&self, http: &reqwest::Client) -> CoreResult<Vec<HotItem>> {
        #[derive(Deserialize)]
        struct ZhihuResp {
            data: Vec<ZhihuEntry>,
        }

        #[derive(Deserialize)]
        struct ZhihuEntry {
            question: ZhihuQuestion,
            reaction: ZhihuReaction,
        }

        #[derive(Deserialize)]
        struct ZhihuQuestion {
            title: String,
            url: String,
        }

        #[derive(Deserialize)]
        struct ZhihuReaction {
            #[serde(default)]
            pv: u64,
        }

        let resp = http
            .get("https://www.zhihu.com/api/v4/creators/rank/hot?domain=0&limit=50")
            .header(
                "User-Agent",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
            )
            .header("Referer", "https://www.zhihu.com/")
            .timeout(Duration::from_secs(15))
            .send()
            .await
            .map_err(CoreError::Upstream)?;

        if !resp.status().is_success() {
            return Err(CoreError::Provider(format!("Zhihu API error: {}", resp.status())));
        }

        let data: ZhihuResp = resp.json().await.map_err(CoreError::Upstream)?;

        let items = data
            .data
            .into_iter()
            .map(|entry| HotItem {
                title: entry.question.title,
                url: entry.question.url,
                hot_value: if entry.reaction.pv > 0 {
                    Some(format_score(entry.reaction.pv))
                } else {
                    None
                },
                label: None,
            })
            .collect();

        Ok(items)
    }
}

pub struct DouyinHotSource;

#[async_trait]
impl HotSource for DouyinHotSource {
    fn id(&self) -> &'static str {
        "douyin"
    }

    fn name(&self) -> &'static str {
        "抖音热搜"
    }

    async fn fetch(&self, http: &reqwest::Client) -> CoreResult<Vec<HotItem>> {
        #[derive(Deserialize)]
        struct DouyinResp {
            data: DouyinData,
        }

        #[derive(Deserialize)]
        struct DouyinData {
            word_list: Vec<DouyinWord>,
        }

        #[derive(Deserialize)]
        struct DouyinWord {
            word: String,
            #[serde(default)]
            hot_value: u64,
        }

        let resp = http
            .get("https://www.douyin.com/aweme/v1/web/hot/search/list/")
            .header(
                "User-Agent",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
            )
            .header("Referer", "https://www.douyin.com/")
            .timeout(Duration::from_secs(15))
            .send()
            .await
            .map_err(CoreError::Upstream)?;

        if !resp.status().is_success() {
            return Err(CoreError::Provider(format!("Douyin API error: {}", resp.status())));
        }

        let data: DouyinResp = resp.json().await.map_err(CoreError::Upstream)?;

        let items = data
            .data
            .word_list
            .into_iter()
            .map(|item| {
                let word = item.word;
                HotItem {
                    url: format!(
                        "https://www.douyin.com/search/{}?type=general",
                        urlencoding::encode(&word)
                    ),
                    title: word,
                    hot_value: if item.hot_value > 0 {
                        Some(format_score(item.hot_value))
                    } else {
                        None
                    },
                    label: None,
                }
            })
            .take(50)
            .collect();

        Ok(items)
    }
}

pub struct DoubanMovieHotSource;

#[async_trait]
impl HotSource for DoubanMovieHotSource {
    fn id(&self) -> &'static str {
        "douban-movie"
    }

    fn name(&self) -> &'static str {
        "豆瓣电影"
    }

    async fn fetch(&self, http: &reqwest::Client) -> CoreResult<Vec<HotItem>> {
        #[derive(Deserialize)]
        struct DoubanResp {
            subjects: Vec<DoubanSubject>,
        }

        #[derive(Deserialize)]
        struct DoubanSubject {
            title: String,
            url: String,
            #[serde(default)]
            rate: String,
        }

        let resp = http
            .get("https://movie.douban.com/j/search_subjects?type=movie&tag=%E7%83%AD%E9%97%A8&page_limit=30&page_start=0")
            .header(
                "User-Agent",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
            )
            .header("Referer", "https://movie.douban.com/")
            .timeout(Duration::from_secs(15))
            .send()
            .await
            .map_err(CoreError::Upstream)?;

        if !resp.status().is_success() {
            return Err(CoreError::Provider(format!(
                "Douban Movie API error: {}",
                resp.status()
            )));
        }

        let data: DoubanResp = resp.json().await.map_err(CoreError::Upstream)?;

        let items = data
            .subjects
            .into_iter()
            .map(|subject| HotItem {
                title: subject.title,
                url: subject.url,
                hot_value: if subject.rate.is_empty() || subject.rate == "0" {
                    None
                } else {
                    Some(format!("评分 {}", subject.rate))
                },
                label: None,
            })
            .collect();

        Ok(items)
    }
}

pub struct ThepaperHotSource;

#[async_trait]
impl HotSource for ThepaperHotSource {
    fn id(&self) -> &'static str {
        "thepaper"
    }

    fn name(&self) -> &'static str {
        "澎湃新闻"
    }

    async fn fetch(&self, http: &reqwest::Client) -> CoreResult<Vec<HotItem>> {
        #[derive(Deserialize)]
        struct PaperResp {
            data: PaperData,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct PaperData {
            #[serde(default)]
            hot_news: Vec<PaperItem>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct PaperItem {
            #[serde(default)]
            cont_id: String,
            #[serde(default)]
            name: String,
            #[serde(default)]
            praise_times: String,
        }

        let resp = http
            .get("https://cache.thepaper.cn/contentapi/wwwIndex/rightSidebar")
            .header(
                "User-Agent",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
            )
            .timeout(Duration::from_secs(15))
            .send()
            .await
            .map_err(CoreError::Upstream)?;

        if !resp.status().is_success() {
            return Err(CoreError::Provider(format!("ThePaper API error: {}", resp.status())));
        }

        let data: PaperResp = resp.json().await.map_err(CoreError::Upstream)?;

        let items = data
            .data
            .hot_news
            .into_iter()
            .map(|item| HotItem {
                title: item.name,
                url: format!("https://www.thepaper.cn/newsDetail_forward_{}", item.cont_id),
                hot_value: item
                    .praise_times
                    .parse::<u64>()
                    .ok()
                    .filter(|&n| n > 0)
                    .map(|n| format!("{n}赞")),
                label: None,
            })
            .collect();

        Ok(items)
    }
}

pub struct HupuHotSource;

#[async_trait]
impl HotSource for HupuHotSource {
    fn id(&self) -> &'static str {
        "hupu"
    }

    fn name(&self) -> &'static str {
        "虎扑步行街"
    }

    async fn fetch(&self, http: &reqwest::Client) -> CoreResult<Vec<HotItem>> {
        let resp = http
            .get("https://bbs.hupu.com/all-gambia")
            .header(
                "User-Agent",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
            )
            .header("Accept", "text/html,application/xhtml+xml")
            .timeout(Duration::from_secs(15))
            .send()
            .await
            .map_err(CoreError::Upstream)?;

        if !resp.status().is_success() {
            return Err(CoreError::Provider(format!("Hupu API error: {}", resp.status())));
        }

        let html = resp.text().await.map_err(CoreError::Upstream)?;
        let document = scraper::Html::parse_document(&html);
        let info_sel = scraper::Selector::parse(".t-info")
            .map_err(|e| CoreError::Provider(format!("Failed to parse Hupu info selector: {e:?}")))?;
        let title_sel = scraper::Selector::parse(".t-title")
            .map_err(|e| CoreError::Provider(format!("Failed to parse Hupu title selector: {e:?}")))?;
        let replies_sel = scraper::Selector::parse(".t-replies")
            .map_err(|e| CoreError::Provider(format!("Failed to parse Hupu replies selector: {e:?}")))?;
        let link_sel = scraper::Selector::parse("a[href]")
            .map_err(|e| CoreError::Provider(format!("Failed to parse Hupu link selector: {e:?}")))?;

        let mut items = Vec::new();
        for element in document.select(&info_sel) {
            let Some(title_el) = element.select(&title_sel).next() else {
                continue;
            };
            let title = title_el.text().collect::<String>().trim().to_string();
            if title.is_empty() {
                continue;
            }

            let url = element
                .select(&link_sel)
                .next()
                .and_then(|a| a.value().attr("href"))
                .map(|href| format!("https://bbs.hupu.com{href}"))
                .unwrap_or_default();
            let hot_value = element
                .select(&replies_sel)
                .next()
                .map(|t| t.text().collect::<String>().trim().to_string())
                .filter(|s| !s.is_empty());

            items.push(HotItem {
                title,
                url,
                hot_value,
                label: None,
            });

            if items.len() >= 50 {
                break;
            }
        }

        Ok(items)
    }
}

pub struct IthomeHotSource;

#[async_trait]
impl HotSource for IthomeHotSource {
    fn id(&self) -> &'static str {
        "ithome"
    }

    fn name(&self) -> &'static str {
        "IT之家"
    }

    async fn fetch(&self, http: &reqwest::Client) -> CoreResult<Vec<HotItem>> {
        #[derive(Deserialize)]
        struct IthomeResp {
            #[serde(rename = "Result")]
            result: Vec<IthomeItem>,
        }

        #[derive(Deserialize)]
        struct IthomeItem {
            newsid: u64,
            title: String,
            #[serde(default)]
            commentcount: u32,
        }

        let resp = http
            .get("https://m.ithome.com/api/news/newslistpageget?categoryid=0&dt=0&startkey=")
            .header(
                "User-Agent",
                "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1",
            )
            .timeout(Duration::from_secs(15))
            .send()
            .await
            .map_err(CoreError::Upstream)?;

        if !resp.status().is_success() {
            return Err(CoreError::Provider(format!("IT之家 API error: {}", resp.status())));
        }

        let data: IthomeResp = resp.json().await.map_err(CoreError::Upstream)?;

        let items = data
            .result
            .into_iter()
            .map(|item| HotItem {
                title: item.title,
                url: format!(
                    "https://www.ithome.com/0/{}/{}.htm",
                    item.newsid / 10000,
                    item.newsid % 10000
                ),
                hot_value: if item.commentcount > 0 {
                    Some(format!("{}评论", item.commentcount))
                } else {
                    None
                },
                label: None,
            })
            .take(30)
            .collect();

        Ok(items)
    }
}

pub struct TiebaHotSource;

#[async_trait]
impl HotSource for TiebaHotSource {
    fn id(&self) -> &'static str {
        "tieba"
    }

    fn name(&self) -> &'static str {
        "百度贴吧"
    }

    async fn fetch(&self, http: &reqwest::Client) -> CoreResult<Vec<HotItem>> {
        #[derive(Deserialize)]
        struct TiebaResp {
            data: TiebaData,
        }

        #[derive(Deserialize)]
        struct TiebaData {
            bang_topic: TiebaBangTopic,
        }

        #[derive(Deserialize)]
        struct TiebaBangTopic {
            topic_list: Vec<TiebaTopic>,
        }

        #[derive(Deserialize)]
        struct TiebaTopic {
            topic_id: u64,
            topic_name: String,
            #[serde(default)]
            discuss_num: u64,
        }

        let resp = http
            .get("https://tieba.baidu.com/hottopic/browse/topicList")
            .header(
                "User-Agent",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
            )
            .timeout(Duration::from_secs(15))
            .send()
            .await
            .map_err(CoreError::Upstream)?;

        if !resp.status().is_success() {
            return Err(CoreError::Provider(format!("Tieba API error: {}", resp.status())));
        }

        let data: TiebaResp = resp.json().await.map_err(CoreError::Upstream)?;

        let items = data
            .data
            .bang_topic
            .topic_list
            .into_iter()
            .map(|topic| {
                let topic_name = topic.topic_name;
                HotItem {
                    url: format!(
                        "https://tieba.baidu.com/hottopic/browse/hottopic?topic_id={}&topic_name={}",
                        topic.topic_id,
                        urlencoding::encode(&topic_name)
                    ),
                    title: topic_name,
                    hot_value: if topic.discuss_num > 0 {
                        Some(format!("{}讨论", format_score(topic.discuss_num)))
                    } else {
                        None
                    },
                    label: None,
                }
            })
            .collect();

        Ok(items)
    }
}

pub struct LinuxdoHotSource;

#[async_trait]
impl HotSource for LinuxdoHotSource {
    fn id(&self) -> &'static str {
        "linuxdo"
    }

    fn name(&self) -> &'static str {
        "Linux.do"
    }

    async fn fetch(&self, http: &reqwest::Client) -> CoreResult<Vec<HotItem>> {
        #[derive(Deserialize)]
        struct LinuxdoResp {
            topic_list: LinuxdoTopicList,
        }

        #[derive(Deserialize)]
        struct LinuxdoTopicList {
            topics: Vec<LinuxdoTopic>,
        }

        #[derive(Deserialize)]
        struct LinuxdoTopic {
            id: u64,
            title: String,
            #[serde(default)]
            views: u64,
        }

        let resp = http
            .get("https://linux.do/top.json?period=daily")
            .header(
                "User-Agent",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
            )
            .header("Accept", "application/json, text/javascript, */*; q=0.01")
            .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
            .header("Referer", "https://linux.do/")
            .header("X-Requested-With", "XMLHttpRequest")
            .header("Sec-Fetch-Dest", "empty")
            .header("Sec-Fetch-Mode", "cors")
            .header("Sec-Fetch-Site", "same-origin")
            .timeout(Duration::from_secs(15))
            .send()
            .await
            .map_err(CoreError::Upstream)?;

        if !resp.status().is_success() {
            return Err(CoreError::Provider(format!("Linux.do API error: {}", resp.status())));
        }

        let data: LinuxdoResp = resp.json().await.map_err(CoreError::Upstream)?;

        let items = data
            .topic_list
            .topics
            .into_iter()
            .map(|topic| HotItem {
                title: topic.title,
                url: format!("https://linux.do/t/topic/{}", topic.id),
                hot_value: if topic.views > 0 {
                    Some(format!("{}浏览", format_score(topic.views)))
                } else {
                    None
                },
                label: None,
            })
            .take(30)
            .collect();

        Ok(items)
    }
}

pub struct NeteaseNewsHotSource;

#[async_trait]
impl HotSource for NeteaseNewsHotSource {
    fn id(&self) -> &'static str {
        "netease-news"
    }

    fn name(&self) -> &'static str {
        "网易新闻"
    }

    async fn fetch(&self, http: &reqwest::Client) -> CoreResult<Vec<HotItem>> {
        #[derive(Deserialize)]
        struct NeteaseResp {
            data: NeteaseData,
        }

        #[derive(Deserialize)]
        struct NeteaseData {
            list: Vec<NeteaseItem>,
        }

        #[derive(Deserialize)]
        struct NeteaseItem {
            title: String,
            #[serde(default)]
            url: String,
            #[serde(default)]
            source: String,
        }

        let resp = http
            .get("https://m.163.com/fe/api/hot/news/flow?page=0")
            .header(
                "User-Agent",
                "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1",
            )
            .timeout(Duration::from_secs(15))
            .send()
            .await
            .map_err(CoreError::Upstream)?;

        if !resp.status().is_success() {
            return Err(CoreError::Provider(format!("NetEase API error: {}", resp.status())));
        }

        let data: NeteaseResp = resp.json().await.map_err(CoreError::Upstream)?;

        let items = data
            .data
            .list
            .into_iter()
            .map(|item| HotItem {
                title: item.title,
                url: item.url,
                hot_value: None,
                label: if item.source.is_empty() {
                    None
                } else {
                    Some(item.source)
                },
            })
            .take(50)
            .collect();

        Ok(items)
    }
}

fn extract_xml_text(block: &str, tag: &str) -> String {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    if let Some(start) = block.find(&open) {
        let inner = &block[start + open.len()..];
        if let Some(end) = inner.find(&close) {
            return inner[..end]
                .trim()
                .trim_start_matches("<![CDATA[")
                .trim_end_matches("]]>")
                .trim()
                .to_string();
        }
    }
    String::new()
}

fn format_score(n: u64) -> String {
    if n >= 100_000_000 {
        format!("{:.1}亿", n as f64 / 100_000_000.0)
    } else if n >= 10_000 {
        format!("{:.1}万", n as f64 / 10_000.0)
    } else {
        n.to_string()
    }
}

pub fn create_registry() -> Vec<std::sync::Arc<dyn HotSource>> {
    vec![
        std::sync::Arc::new(WeiboHotSource),
        std::sync::Arc::new(BilibiliHotSource),
        std::sync::Arc::new(BaiduHotSource),
        std::sync::Arc::new(ToutiaoHotSource),
        std::sync::Arc::new(KrHotSource),
        std::sync::Arc::new(GithubTrendingSource),
        std::sync::Arc::new(JuejinHotSource),
        std::sync::Arc::new(V2exSource),
        std::sync::Arc::new(SspaiHotSource),
        std::sync::Arc::new(ZhihuHotSource),
        std::sync::Arc::new(DouyinHotSource),
        std::sync::Arc::new(HackerNewsSource),
        std::sync::Arc::new(DoubanMovieHotSource),
        std::sync::Arc::new(ThepaperHotSource),
        std::sync::Arc::new(HupuHotSource),
        std::sync::Arc::new(IthomeHotSource),
        std::sync::Arc::new(TiebaHotSource),
        std::sync::Arc::new(LinuxdoHotSource),
        std::sync::Arc::new(NeteaseNewsHotSource),
    ]
}

#[cfg(all(test, feature = "live-api"))]
mod tests {
    use super::*;

    // China-only source: Sina Weibo's hot-search endpoint geo-blocks /
    // WAFs non-CN traffic (GitHub Actions runners are in US/EU and get
    // 403 / 302). Set `RUN_CN_LIVE_TESTS=1` to run from a CN host.
    #[tokio::test]
    #[ignore]
    async fn test_weibo_source() {
        if std::env::var("RUN_CN_LIVE_TESTS").is_err() {
            eprintln!("skipping test_weibo_source: set RUN_CN_LIVE_TESTS=1 to enable");
            return;
        }
        let client = reqwest::Client::new();
        let source = WeiboHotSource;
        let result = source.fetch(&client).await;
        assert!(result.is_ok());
    }
}
