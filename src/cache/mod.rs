//! Cache module for storing news articles
//!
//! Provides in-memory caching for news articles organized by category
//! and full article content cached by URL.

mod article_cache;
mod news_cache;

pub use article_cache::*;
pub use news_cache::*;

use std::sync::Arc;

/// Create a new news cache with the given configuration
pub fn create_cache(max_articles_per_category: usize) -> NewsCache {
    NewsCache::new(max_articles_per_category)
}

/// Create a shared news cache wrapped in Arc
pub fn create_shared_cache(max_articles_per_category: usize) -> Arc<NewsCache> {
    Arc::new(create_cache(max_articles_per_category))
}

/// Create a new article content cache
pub fn create_article_cache(max_articles: usize) -> ArticleCache {
    ArticleCache::new(max_articles)
}

/// Create a shared article content cache wrapped in Arc
pub fn create_shared_article_cache(max_articles: usize) -> Arc<ArticleCache> {
    Arc::new(create_article_cache(max_articles))
}
