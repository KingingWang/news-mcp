//! Article content cache implementation
//!
//! Thread-safe in-memory cache for storing fetched full article content by URL.
//! Provides high-concurrency access with RwLock<HashMap> pattern.

use crate::error::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

/// Cached article content structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedArticle {
    /// Full article content (plain text, cleaned from HTML)
    pub content: String,
    /// When the content was fetched
    pub fetched_at: DateTime<Utc>,
    /// Word count of the content
    pub word_count: usize,
}

impl CachedArticle {
    /// Create a new cached article
    pub fn new(content: String) -> Self {
        let word_count = content.split_whitespace().count();
        Self {
            content,
            fetched_at: Utc::now(),
            word_count,
        }
    }
}

/// Article content cache structure
#[derive(Debug)]
pub struct ArticleCache {
    /// Cached articles by URL
    articles: RwLock<HashMap<String, CachedArticle>>,
    /// Maximum cached articles
    max_articles: usize,
}

impl ArticleCache {
    /// Create a new article cache
    pub fn new(max_articles: usize) -> Self {
        Self {
            articles: RwLock::new(HashMap::new()),
            max_articles,
        }
    }

    /// Get cached article content by URL
    pub fn get(&self, url: &str) -> Result<Option<CachedArticle>> {
        let articles = self
            .articles
            .read()
            .map_err(|e| Error::cache(e.to_string()))?;
        Ok(articles.get(url).cloned())
    }

    /// Insert or update cached article content
    pub fn insert(&self, url: String, article: CachedArticle) -> Result<()> {
        let mut articles = self
            .articles
            .write()
            .map_err(|e| Error::cache(e.to_string()))?;

        // If cache is full and URL is new, remove oldest entry
        if articles.len() >= self.max_articles && !articles.contains_key(&url) {
            if let Some((oldest_url, _)) = articles.iter().min_by_key(|(_, a)| a.fetched_at) {
                let oldest_url = oldest_url.clone();
                articles.remove(&oldest_url);
            }
        }

        articles.insert(url, article);
        Ok(())
    }

    /// Get total cached article count
    pub fn count(&self) -> Result<usize> {
        let articles = self
            .articles
            .read()
            .map_err(|e| Error::cache(e.to_string()))?;
        Ok(articles.len())
    }

    /// Clear all cached articles
    pub fn clear(&self) -> Result<()> {
        let mut articles = self
            .articles
            .write()
            .map_err(|e| Error::cache(e.to_string()))?;
        articles.clear();
        Ok(())
    }
}

impl Default for ArticleCache {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cached_article_creation() {
        let article = CachedArticle::new("This is test content with seven words.".to_string());
        assert_eq!(article.word_count, 7);
    }

    #[test]
    fn test_cache_insert_and_get() {
        let cache = ArticleCache::new(10);
        let article = CachedArticle::new("Content".to_string());

        cache
            .insert("https://example.com".to_string(), article.clone())
            .unwrap();

        let cached = cache.get("https://example.com").unwrap();
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().content, "Content");
    }

    #[test]
    fn test_cache_max_capacity() {
        let cache = ArticleCache::new(3);

        for i in 0..5 {
            let article = CachedArticle::new(format!("Content {}", i));
            cache
                .insert(format!("https://example.com/{}", i), article)
                .unwrap();
        }

        let count = cache.count().unwrap();
        assert_eq!(count, 3);
    }
}
