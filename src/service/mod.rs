//! Service module for fetching and processing news
//!
//! Provides the news service for fetching RSS feeds and Hacker News.

mod hn_service;
mod news_service;

pub use hn_service::*;
pub use news_service::*;
