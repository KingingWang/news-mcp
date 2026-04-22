//! Service module for fetching and processing news
//!
//! Provides the news service for fetching RSS feeds, Hacker News, and NewsNow hot lists.
//! All sources implement the [`NewsSource`] trait for extensibility.

mod hn_service;
mod news_service;
mod newsnow_service;

pub use hn_service::*;
pub use news_service::*;
pub use newsnow_service::*;

use crate::cache::{NewsArticle, NewsCategory};
use crate::error::Result;
use async_trait::async_trait;
use std::collections::HashMap;

/// A pluggable news source that can fetch articles.
///
/// Implement this trait to add new data sources (Reddit, Twitter, etc.)
/// without modifying the poller or tool layer.
#[async_trait]
pub trait NewsSource: Send + Sync {
    /// Human-readable name of this source (e.g. "RSS Feeds", "Hacker News API")
    fn name(&self) -> &str;

    /// Fetch articles, grouped by category.
    async fn fetch(&self) -> Result<HashMap<NewsCategory, Vec<NewsArticle>>>;
}
