//! Cache module for storing news articles
//!
//! Provides in-memory caching for news articles organized by category.

mod news_cache;

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
