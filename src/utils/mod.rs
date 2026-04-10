//! Utilities module
//!
//! Provides helper functions and constants for the news-mcp server.

use crate::cache::NewsCategory;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};

/// RSS feed URLs for each category
pub fn get_feed_urls(category: &NewsCategory) -> Vec<&'static str> {
    match category {
        NewsCategory::Technology => vec![
            "https://techcrunch.com/feed/",
            "https://feeds.arstechnica.com/arstechnica/index",
            "https://www.theverge.com/rss/index.xml",
        ],
        NewsCategory::Business => vec!["https://feeds.bbci.co.uk/news/business/rss.xml"],
        NewsCategory::Science => vec!["https://www.sciencedaily.com/rss/all.xml"],
        NewsCategory::Health => vec!["https://feeds.bbci.co.uk/news/health/rss.xml"],
        NewsCategory::Sports => vec!["https://feeds.bbci.co.uk/sport/rss.xml"],
        NewsCategory::Entertainment => {
            vec!["https://feeds.bbci.co.uk/news/entertainment_and_arts/rss.xml"]
        }
        NewsCategory::General => vec!["https://feeds.bbci.co.uk/news/rss.xml"],
        NewsCategory::World => vec!["https://feeds.bbci.co.uk/news/world/rss.xml"],
    }
}

/// Build HTTP client with retry middleware
pub fn build_http_client_with_retry() -> reqwest_middleware::ClientWithMiddleware {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(10))
        .user_agent("news-mcp/0.1.0")
        .build()
        .expect("Failed to create HTTP client");

    // Create retry policy with exponential backoff
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

    ClientBuilder::new(client)
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build()
}

/// Initialize logging based on configuration
pub fn init_logging(level: &str, enable_console: bool) {
    if enable_console {
        let filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(level));

        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .with_thread_ids(false)
            .init();
    }
}

/// Format articles for output
pub fn format_articles_as_markdown(articles: &[crate::cache::NewsArticle]) -> String {
    if articles.is_empty() {
        return "No articles found.".to_string();
    }

    let mut output = String::new();
    output.push_str("# News Articles\n\n");

    for article in articles {
        output.push_str(&format!("## {}\n", article.title));

        if let Some(desc) = &article.description {
            output.push_str(&format!("{}\n\n", desc));
        }

        output.push_str(&format!("- **Source**: {}\n", article.source));
        output.push_str(&format!("- **Link**: {}\n", article.link));

        if let Some(date) = &article.published_at {
            output.push_str(&format!(
                "- **Published**: {}\n",
                date.format("%Y-%m-%d %H:%M UTC")
            ));
        }

        if let Some(author) = &article.author {
            output.push_str(&format!("- **Author**: {}\n", author));
        }

        output.push_str("\n---\n\n");
    }

    output
}

/// Format articles as JSON
pub fn format_articles_as_json(articles: &[crate::cache::NewsArticle]) -> String {
    serde_json::to_string_pretty(articles).unwrap_or_else(|_| "[]".to_string())
}

/// Format articles as plain text
pub fn format_articles_as_text(articles: &[crate::cache::NewsArticle]) -> String {
    if articles.is_empty() {
        return "No articles found.".to_string();
    }

    let mut output = String::new();

    for (i, article) in articles.iter().enumerate() {
        output.push_str(&format!("{}. {}\n", i + 1, article.title));

        if let Some(desc) = &article.description {
            output.push_str(&format!("   {}\n", desc));
        }

        output.push_str(&format!(
            "   Source: {} | Link: {}\n",
            article.source, article.link
        ));

        if let Some(date) = &article.published_at {
            output.push_str(&format!(
                "   Published: {}\n",
                date.format("%Y-%m-%d %H:%M UTC")
            ));
        }

        output.push('\n');
    }

    output
}
