//! Article fetcher service implementation
//!
//! Fetches and extracts full article content from HTML pages.

use crate::error::{Error, Result};
use scraper::{Html, Selector};
use std::time::Duration;
use tracing::{debug, info, warn};

/// Article fetcher for extracting full content from HTML pages
pub struct ArticleFetcher {
    /// HTTP client
    http_client: reqwest::Client,
    /// Maximum articles to fetch per category
    max_fetch_per_category: usize,
}

impl ArticleFetcher {
    /// Create a new article fetcher
    pub fn new(timeout_secs: u64, max_fetch_per_category: usize) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .connect_timeout(Duration::from_secs(5))
            .user_agent("news-mcp/0.1.2 (article-fetcher)")
            .build()
            .expect("Failed to create HTTP client for article fetching");

        Self {
            http_client,
            max_fetch_per_category,
        }
    }

    /// Get max fetch per category
    pub fn max_fetch_per_category(&self) -> usize {
        self.max_fetch_per_category
    }

    /// Fetch article content from URL
    pub async fn fetch_content(&self, url: &str) -> Result<Option<String>> {
        debug!("Fetching article content from: {}", url);

        // Fetch HTML content
        let response = self
            .http_client
            .get(url)
            .send()
            .await
            .map_err(|e| Error::Http {
                code: crate::error::ErrorCode::Network,
                message: format!("HTTP request failed: {}", e),
                source: Box::new(e),
            })?;

        if !response.status().is_success() {
            warn!("HTTP error {} for URL: {}", response.status(), url);
            return Ok(None);
        }

        let html_content: String = response.text().await.map_err(|e| Error::Http {
            code: crate::error::ErrorCode::Network,
            message: format!("Failed to read response body: {}", e),
            source: Box::new(e),
        })?;

        // Extract main content
        let content = self.extract_content_from_html(&html_content);

        if content.is_empty() {
            warn!("Failed to extract content from HTML for URL: {}", url);
            return Ok(None);
        }

        info!(
            "Successfully fetched article content from {} ({} chars)",
            url,
            content.len()
        );

        Ok(Some(content))
    }

    /// Extract main content from HTML
    fn extract_content_from_html(&self, html: &str) -> String {
        let document = Html::parse_document(html);
        self.extract_main_content(&document)
    }

    /// Extract main content from HTML document using multiple strategies
    fn extract_main_content(&self, document: &Html) -> String {
        // Strategy 1: Try <article> tag (most semantic)
        let article_selector = Selector::parse("article").unwrap();
        if let Some(article_elem) = document.select(&article_selector).next() {
            let content = self.extract_text_from_element(&article_elem);
            if content.len() > 200 {
                return content;
            }
        }

        // Strategy 2: Try <main> tag
        let main_selector = Selector::parse("main").unwrap();
        if let Some(main_elem) = document.select(&main_selector).next() {
            let content = self.extract_text_from_element(&main_elem);
            if content.len() > 200 {
                return content;
            }
        }

        // Strategy 3: Try content divs with common class names
        let content_div_patterns = [
            ".article-content",
            ".post-content",
            ".entry-content",
            ".content-body",
            ".article-body",
            ".post-body",
            "#article-content",
            "#post-content",
            "#content",
            ".content",
        ];

        for pattern in content_div_patterns {
            if let Ok(selector) = Selector::parse(pattern) {
                if let Some(elem) = document.select(&selector).next() {
                    let content = self.extract_text_from_element(&elem);
                    if content.len() > 200 {
                        return content;
                    }
                }
            }
        }

        // Strategy 4: Find all substantial paragraphs
        let p_selector = Selector::parse("p").unwrap();
        let all_paragraphs: Vec<String> = document
            .select(&p_selector)
            .map(|p| self.extract_text_from_element(&p))
            .filter(|text| text.len() > 50)
            .collect();

        if all_paragraphs.is_empty() {
            return String::new();
        }

        // Join all substantial paragraphs
        all_paragraphs.join("\n\n")
    }

    /// Extract clean text from an HTML element
    fn extract_text_from_element(&self, elem: &scraper::ElementRef) -> String {
        let text = elem.text().collect::<String>();
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}
