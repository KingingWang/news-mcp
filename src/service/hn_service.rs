//! Hacker News service implementation
//!
//! Fetches stories from the Hacker News Firebase API via plain HTTP.

use crate::cache::{NewsArticle, NewsCategory};
use crate::error::{Error, Result};
use crate::service::NewsSource;
use async_trait::async_trait;
use chrono::DateTime;
use serde::Deserialize;
use std::collections::HashMap;
use tracing::{debug, info, warn};

const HN_API_BASE: &str = "https://hacker-news.firebaseio.com/v0";

/// Raw HN item returned by the Firebase API
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HnItem {
    id: u64,
    #[serde(default)]
    title: String,
    #[serde(default)]
    text: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    by: String,
    /// Unix timestamp
    #[serde(default)]
    time: i64,
    #[serde(default)]
    score: i64,
    /// item type: "story", "job", "comment", etc.
    #[serde(default, rename = "type")]
    item_type: String,
}

/// Hacker News service for fetching stories via HTTP
pub struct HnService {
    client: reqwest::Client,
}

impl HnService {
    /// Create a new Hacker News service
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Fetch story IDs from a given endpoint, then fetch each story.
    async fn fetch_stories(&self, endpoint: &str, limit: usize) -> Result<Vec<NewsArticle>> {
        let url = format!("{}/{}.json", HN_API_BASE, endpoint);
        debug!("Fetching story IDs from {}", url);

        let ids: Vec<u64> = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::rss(format!("Failed to fetch {}: {}", endpoint, e)))?
            .json()
            .await
            .map_err(|e| Error::rss(format!("Failed to parse {}: {}", endpoint, e)))?;

        let limited_ids: Vec<_> = ids.into_iter().take(limit).collect();
        self.fetch_items_by_ids(&limited_ids).await
    }

    /// Fetch top stories
    pub async fn fetch_top_stories(&self, limit: usize) -> Result<Vec<NewsArticle>> {
        debug!("Fetching {} top stories from Hacker News", limit);
        self.fetch_stories("topstories", limit).await
    }

    /// Fetch best stories
    pub async fn fetch_best_stories(&self, limit: usize) -> Result<Vec<NewsArticle>> {
        debug!("Fetching {} best stories from Hacker News", limit);
        self.fetch_stories("beststories", limit).await
    }

    /// Fetch latest/new stories
    pub async fn fetch_new_stories(&self, limit: usize) -> Result<Vec<NewsArticle>> {
        debug!("Fetching {} new stories from Hacker News", limit);
        self.fetch_stories("newstories", limit).await
    }

    /// Fetch Ask HN stories
    pub async fn fetch_ask_stories(&self, limit: usize) -> Result<Vec<NewsArticle>> {
        debug!("Fetching {} Ask HN stories", limit);
        self.fetch_stories("askstories", limit).await
    }

    /// Fetch Show HN stories
    pub async fn fetch_show_stories(&self, limit: usize) -> Result<Vec<NewsArticle>> {
        debug!("Fetching {} Show HN stories", limit);
        self.fetch_stories("showstories", limit).await
    }

    /// Convert a raw HN item to a NewsArticle
    fn item_to_article(item: &HnItem) -> Option<NewsArticle> {
        if item.title.is_empty() || item.item_type != "story" {
            return None;
        }

        let title = clean_text(&item.title);

        let description = if !item.text.is_empty() {
            Some(clean_text(&item.text))
        } else {
            None
        };

        let link = if !item.url.is_empty() {
            item.url.clone()
        } else {
            format!("https://news.ycombinator.com/item?id={}", item.id)
        };

        let published_at = DateTime::from_timestamp(item.time, 0);

        Some(NewsArticle::new(
            title,
            description,
            link,
            "Hacker News".to_string(),
            NewsCategory::HackerNews,
            published_at,
            if item.by.is_empty() {
                None
            } else {
                Some(item.by.clone())
            },
        ))
    }

    /// Fetch items by IDs concurrently (chunks of 5)
    async fn fetch_items_by_ids(&self, ids: &[u64]) -> Result<Vec<NewsArticle>> {
        let mut articles = Vec::with_capacity(ids.len());

        for chunk in ids.chunks(5) {
            let mut tasks = Vec::new();
            for &id in chunk {
                let client = &self.client;
                let url = format!("{}/item/{}.json", HN_API_BASE, id);
                tasks.push(async move {
                    debug!("Fetching HN item {}", id);
                    client.get(&url).send().await?.json::<HnItem>().await
                });
            }

            let results = futures::future::join_all(tasks).await;

            for result in results {
                match result {
                    Ok(item) => {
                        if let Some(article) = Self::item_to_article(&item) {
                            debug!("Successfully fetched story: {}", item.title);
                            articles.push(article);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to fetch HN item: {}", e);
                    }
                }
            }
        }

        // Sort by published time (newest first)
        articles.sort_by(|a, b| match (&a.published_at, &b.published_at) {
            (Some(a_date), Some(b_date)) => b_date.cmp(a_date),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });

        info!("Fetched {} Hacker News articles", articles.len());
        Ok(articles)
    }
}

#[async_trait]
impl NewsSource for HnService {
    fn name(&self) -> &str {
        "Hacker News"
    }

    async fn fetch(&self) -> Result<HashMap<NewsCategory, Vec<NewsArticle>>> {
        let articles = self.fetch_top_stories(30).await?;
        let mut map = HashMap::new();
        if !articles.is_empty() {
            map.insert(NewsCategory::HackerNews, articles);
        }
        Ok(map)
    }
}

impl Default for HnService {
    fn default() -> Self {
        Self::new()
    }
}

/// Clean text by removing HTML entities and unwanted content
fn clean_text(text: &str) -> String {
    let mut cleaned = text.to_string();

    // Remove common HTML entities
    cleaned = cleaned.replace("&lt;", "<");
    cleaned = cleaned.replace("&gt;", ">");
    cleaned = cleaned.replace("&amp;", "&");
    cleaned = cleaned.replace("&quot;", "\"");
    cleaned = cleaned.replace("&apos;", "'");
    cleaned = cleaned.replace("&nbsp;", " ");
    cleaned = cleaned.replace("&mdash;", "—");
    cleaned = cleaned.replace("&ndash;", "-");
    cleaned = cleaned.replace("&#x27;", "'");
    cleaned = cleaned.replace("&#x2F;", "/");

    // Remove HTML tags (basic)
    let mut result = String::new();
    let mut in_tag = false;
    for c in cleaned.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(c);
        }
    }

    // Trim and normalize whitespace
    result = result
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    // Remove multiple consecutive spaces
    while result.contains("  ") {
        result = result.replace("  ", " ");
    }

    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_text_basic() {
        let input = "Hello &amp; World";
        let output = clean_text(input);
        assert_eq!(output, "Hello & World");
    }

    #[test]
    fn test_clean_text_html_tags() {
        let input = "<p>Hello World</p>";
        let output = clean_text(input);
        assert_eq!(output, "Hello World");
    }

    #[test]
    fn test_clean_text_whitespace() {
        let input = "Hello   World  \n\n Test";
        let output = clean_text(input);
        assert_eq!(output, "Hello World Test");
    }

    #[test]
    fn test_clean_text_entities() {
        let input = "Test &lt;code&gt; &quot;quoted&quot; text";
        let output = clean_text(input);
        // HTML tags are removed, so <code> becomes empty
        assert_eq!(output, "Test \"quoted\" text");
    }
}
