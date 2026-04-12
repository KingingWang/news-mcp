//! Hacker News service implementation
//!
//! Handles fetching and processing stories from Hacker News API.

use crate::cache::{NewsArticle, NewsCategory};
use crate::error::{Error, Result};
use crate::service::NewsSource;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use newswrap::client::HackerNewsClient;
use newswrap::items::stories::HackerNewsStory;
use std::collections::HashMap;
use time::OffsetDateTime;
use tracing::{debug, info, warn};

/// Hacker News service for fetching stories
pub struct HnService {
    /// HN API client
    client: HackerNewsClient,
}

impl HnService {
    /// Create a new Hacker News service
    pub fn new() -> Self {
        Self {
            client: HackerNewsClient::new(),
        }
    }

    /// Convert HN story to NewsArticle with text cleaning
    fn story_to_article(story: &HackerNewsStory) -> Option<NewsArticle> {
        // Skip stories without valid content
        if story.title.is_empty() {
            warn!("Skipping story with empty title: ID {}", story.id);
            return None;
        }

        // Clean and process the title
        let title = clean_text(&story.title);

        // Clean and process the text content (for Ask HN etc.)
        let description = if !story.text.is_empty() {
            Some(clean_text(&story.text))
        } else {
            None
        };

        // Use URL if available, otherwise link to HN discussion
        let link = if !story.url.is_empty() {
            story.url.clone()
        } else {
            format!("https://news.ycombinator.com/item?id={}", story.id)
        };

        // Convert time to chrono DateTime
        let published_at = convert_hn_time(&story.created_at);

        Some(NewsArticle::new(
            title,
            description,
            link,
            "Hacker News".to_string(),
            NewsCategory::HackerNews,
            published_at,
            Some(story.by.clone()),
        ))
    }

    /// Fetch top stories from Hacker News
    pub async fn fetch_top_stories(&self, limit: usize) -> Result<Vec<NewsArticle>> {
        debug!("Fetching {} top stories from Hacker News", limit);

        let story_ids = self
            .client
            .realtime
            .get_top_stories()
            .await
            .map_err(|e| Error::rss(format!("Failed to fetch top stories: {}", e)))?;

        let limited_ids: Vec<_> = story_ids.into_iter().take(limit).collect();
        self.fetch_stories_by_ids(&limited_ids).await
    }

    /// Fetch best stories from Hacker News
    pub async fn fetch_best_stories(&self, limit: usize) -> Result<Vec<NewsArticle>> {
        debug!("Fetching {} best stories from Hacker News", limit);

        let story_ids = self
            .client
            .realtime
            .get_best_stories()
            .await
            .map_err(|e| Error::rss(format!("Failed to fetch best stories: {}", e)))?;

        let limited_ids: Vec<_> = story_ids.into_iter().take(limit).collect();
        self.fetch_stories_by_ids(&limited_ids).await
    }

    /// Fetch latest/new stories from Hacker News
    pub async fn fetch_new_stories(&self, limit: usize) -> Result<Vec<NewsArticle>> {
        debug!("Fetching {} new stories from Hacker News", limit);

        let story_ids = self
            .client
            .realtime
            .get_latest_stories()
            .await
            .map_err(|e| Error::rss(format!("Failed to fetch new stories: {}", e)))?;

        let limited_ids: Vec<_> = story_ids.into_iter().take(limit).collect();
        self.fetch_stories_by_ids(&limited_ids).await
    }

    /// Fetch Ask HN stories
    pub async fn fetch_ask_stories(&self, limit: usize) -> Result<Vec<NewsArticle>> {
        debug!("Fetching {} Ask HN stories", limit);

        let story_ids = self
            .client
            .realtime
            .get_ask_hacker_news_stories()
            .await
            .map_err(|e| Error::rss(format!("Failed to fetch Ask HN stories: {}", e)))?;

        let limited_ids: Vec<_> = story_ids.into_iter().take(limit).collect();
        self.fetch_stories_by_ids(&limited_ids).await
    }

    /// Fetch Show HN stories
    pub async fn fetch_show_stories(&self, limit: usize) -> Result<Vec<NewsArticle>> {
        debug!("Fetching {} Show HN stories", limit);

        let story_ids = self
            .client
            .realtime
            .get_show_hacker_news_stories()
            .await
            .map_err(|e| Error::rss(format!("Failed to fetch Show HN stories: {}", e)))?;

        let limited_ids: Vec<_> = story_ids.into_iter().take(limit).collect();
        self.fetch_stories_by_ids(&limited_ids).await
    }

    /// Fetch stories by IDs with concurrent processing
    async fn fetch_stories_by_ids(
        &self,
        ids: &[newswrap::HackerNewsID],
    ) -> Result<Vec<NewsArticle>> {
        let mut articles = Vec::with_capacity(ids.len());

        // Process in chunks of 5 for concurrent fetching
        let chunk_size = 5;
        for chunk in ids.chunks(chunk_size) {
            let mut tasks = Vec::new();

            for id in chunk {
                let client = &self.client;
                let task = async move { client.items.get_story(*id).await };
                tasks.push(task);
            }

            // Execute chunk concurrently
            let results = futures::future::join_all(tasks).await;

            for result in results {
                match result {
                    Ok(story) => {
                        if let Some(article) = Self::story_to_article(&story) {
                            articles.push(article);
                            debug!("Successfully fetched story: {}", story.title);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to fetch story: {}", e);
                    }
                }
            }
        }

        // Sort by score (descending)
        articles.sort_by(|a, b| {
            // We don't have score in NewsArticle, so sort by published_at
            match (a.published_at, b.published_at) {
                (Some(a_date), Some(b_date)) => b_date.cmp(&a_date),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
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

/// Convert HN time (time::OffsetDateTime) to chrono DateTime
fn convert_hn_time(time: &OffsetDateTime) -> Option<DateTime<Utc>> {
    // Convert time crate to chrono
    let unix_timestamp = time.unix_timestamp();
    DateTime::from_timestamp(unix_timestamp, 0)
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
