//! News service implementation
//!
//! Handles fetching and parsing RSS feeds.

use crate::cache::{NewsArticle, NewsCategory};
use crate::error::{Error, Result};
use crate::utils::get_feed_urls;
use feed_rs::parser;
use tracing::{debug, error, info, warn};

/// News service for fetching RSS feeds
pub struct NewsService {
    /// HTTP client with retry middleware
    client: reqwest_middleware::ClientWithMiddleware,
}

impl NewsService {
    /// Create a new news service
    pub fn new() -> Self {
        Self {
            client: crate::utils::build_http_client_with_retry(),
        }
    }

    /// Fetch RSS feed from URL and parse articles
    pub async fn fetch_rss_feed(
        &self,
        url: &str,
        category: NewsCategory,
    ) -> Result<Vec<NewsArticle>> {
        debug!("Fetching RSS feed from: {}", url);

        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(Error::rss(format!(
                "Failed to fetch {}: status {}",
                url,
                response.status()
            )));
        }

        let content = response.text().await?;
        self.parse_feed(&content, category)
    }

    /// Parse RSS/Atom feed content into articles
    pub fn parse_feed(&self, content: &str, category: NewsCategory) -> Result<Vec<NewsArticle>> {
        let feed = parser::parse(content.as_bytes())
            .map_err(|e| Error::rss(format!("Failed to parse feed: {}", e)))?;

        // Clone the feed title before the closure to avoid move issues
        let feed_title = feed.title.clone();

        let articles: Vec<NewsArticle> = feed
            .entries
            .into_iter()
            .filter_map(|entry| {
                let title = entry
                    .title
                    .map(|t| t.content)
                    .unwrap_or_else(|| "Untitled".to_string());

                let description = entry
                    .summary
                    .map(|s| s.content)
                    .or_else(|| entry.content.map(|c| c.body.unwrap_or_default()));

                let link = entry
                    .links
                    .first()
                    .map(|l| l.href.clone())
                    .unwrap_or_else(String::new);

                if link.is_empty() {
                    warn!("Article '{}' has no link, skipping", title);
                    return None;
                }

                let source = feed_title
                    .clone()
                    .map(|t| t.content)
                    .unwrap_or_else(|| "Unknown Source".to_string());

                let published_at = entry.published.or(entry.updated);

                let author = entry.authors.first().map(|a| a.name.clone());

                Some(NewsArticle::new(
                    title,
                    description,
                    link,
                    source,
                    category,
                    published_at,
                    author,
                ))
            })
            .collect();

        info!("Parsed {} articles from feed", articles.len());
        Ok(articles)
    }

    /// Fetch all feeds for a category concurrently
    pub async fn fetch_category(&self, category: NewsCategory) -> Result<Vec<NewsArticle>> {
        let urls = get_feed_urls(&category);
        let mut all_articles = Vec::new();

        for url in urls {
            match self.fetch_rss_feed(url, category).await {
                Ok(articles) => {
                    all_articles.extend(articles);
                }
                Err(e) => {
                    error!("Failed to fetch feed {}: {}", url, e);
                }
            }
        }

        // Sort by publication date (most recent first)
        all_articles.sort_by(|a, b| match (a.published_at, b.published_at) {
            (Some(a_date), Some(b_date)) => b_date.cmp(&a_date),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });

        info!(
            "Fetched {} total articles for category {}",
            all_articles.len(),
            category
        );
        Ok(all_articles)
    }

    /// Fetch all categories concurrently
    pub async fn fetch_all_categories(
        &self,
    ) -> Result<std::collections::HashMap<NewsCategory, Vec<NewsArticle>>> {
        let categories = NewsCategory::all();
        let mut results = std::collections::HashMap::new();

        // Use futures to fetch concurrently
        let futures: Vec<_> = categories
            .iter()
            .map(|category| {
                let cat = *category;
                async move {
                    let articles = self.fetch_category(cat).await?;
                    Ok::<_, Error>((cat, articles))
                }
            })
            .collect();

        // Execute all futures concurrently
        let results_vec = futures::future::try_join_all(futures).await?;

        for (category, articles) in results_vec {
            results.insert(category, articles);
        }

        Ok(results)
    }
}

impl Default for NewsService {
    fn default() -> Self {
        Self::new()
    }
}
