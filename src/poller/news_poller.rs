//! News poller implementation
//!
//! Background task that periodically fetches news and updates cache.

use crate::cache::NewsCache;
use crate::config::PollerConfig;
use crate::service::{HnService, NewsService};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};

/// News poller for background fetching
pub struct NewsPoller {
    /// News service for fetching feeds
    service: Arc<NewsService>,
    /// Hacker News service
    hn_service: Arc<HnService>,
    /// Cache to store fetched articles
    cache: Arc<NewsCache>,
    /// Polling configuration
    config: PollerConfig,
    /// Running flag
    running: std::sync::atomic::AtomicBool,
    /// Initial poll completed flag
    initial_poll_completed: std::sync::atomic::AtomicBool,
}

impl NewsPoller {
    /// Create a new poller
    pub fn new(service: Arc<NewsService>, cache: Arc<NewsCache>, config: PollerConfig) -> Self {
        Self {
            service,
            hn_service: Arc::new(HnService::new()),
            cache,
            config,
            running: std::sync::atomic::AtomicBool::new(false),
            initial_poll_completed: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Start background polling
    pub async fn start(&self) {
        if !self.config.enabled {
            info!("Poller is disabled by configuration");
            return;
        }

        self.running
            .store(true, std::sync::atomic::Ordering::SeqCst);
        info!(
            "Starting news poller with interval of {} seconds",
            self.config.interval_secs
        );

        // Initial poll immediately
        if let Err(e) = self.poll_once().await {
            error!("Initial poll failed: {}", e);
        }
        self.initial_poll_completed
            .store(true, std::sync::atomic::Ordering::SeqCst);

        // Set up interval for subsequent polls
        let mut poll_interval = interval(Duration::from_secs(self.config.interval_secs));

        loop {
            poll_interval.tick().await;

            if !self.running.load(std::sync::atomic::Ordering::SeqCst) {
                info!("Poller stopped");
                break;
            }

            if let Err(e) = self.poll_once().await {
                error!("Poll failed: {}", e);
                // Continue polling even on error
            }
        }
    }

    /// Perform a single poll cycle
    pub async fn poll_once(&self) -> crate::error::Result<()> {
        info!("Starting poll cycle");
        let start_time = std::time::Instant::now();

        let results = self.service.fetch_all_categories().await?;

        let mut total_articles = 0;
        let mut successful_categories = 0;

        for (category, articles) in results {
            let count = articles.len();
            total_articles += count;

            if count > 0 {
                self.cache.set_category_news(category, articles)?;
                successful_categories += 1;
                info!("Updated {} articles for category {}", count, category);
            } else {
                warn!("No articles fetched for category {}", category);
            }
        }

        // Fetch Hacker News top stories
        let hn_articles = self.hn_service.fetch_top_stories(30).await?;
        if !hn_articles.is_empty() {
            let hn_count = hn_articles.len();
            total_articles += hn_count;
            self.cache.set_category_news(crate::cache::NewsCategory::HackerNews, hn_articles)?;
            successful_categories += 1;
            info!("Updated {} articles for Hacker News category", hn_count);
        }

        let elapsed = start_time.elapsed();
        info!(
            "Poll cycle completed: {} articles from {} categories in {}ms",
            total_articles,
            successful_categories,
            elapsed.as_millis()
        );

        Ok(())
    }

    /// Stop the poller
    pub fn stop(&self) {
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);
        info!("Stopping news poller");
    }

    /// Check if poller is running
    pub fn is_running(&self) -> bool {
        self.running.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Check if initial poll has completed
    pub fn is_initial_poll_completed(&self) -> bool {
        self.initial_poll_completed
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Wait for initial poll to complete (with timeout)
    pub async fn wait_for_initial_poll(&self, timeout_secs: u64) -> bool {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);

        while !self
            .initial_poll_completed
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            if start.elapsed() > timeout {
                return false;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        true
    }
}
