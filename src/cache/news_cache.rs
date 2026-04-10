//! News cache implementation
//!
//! Thread-safe in-memory cache for storing news articles by category.

use crate::error::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

/// News category enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NewsCategory {
    Technology,
    Business,
    Science,
    Health,
    Sports,
    Entertainment,
    General,
    World,
}

impl std::str::FromStr for NewsCategory {
    type Err = Error;

    fn from_str(s: &str) -> Result<NewsCategory> {
        match s.to_lowercase().as_str() {
            "technology" | "tech" => Ok(NewsCategory::Technology),
            "business" => Ok(NewsCategory::Business),
            "science" => Ok(NewsCategory::Science),
            "health" => Ok(NewsCategory::Health),
            "sports" => Ok(NewsCategory::Sports),
            "entertainment" => Ok(NewsCategory::Entertainment),
            "general" => Ok(NewsCategory::General),
            "world" => Ok(NewsCategory::World),
            _ => Err(Error::invalid_category(s)),
        }
    }
}

impl NewsCategory {
    /// Get all categories
    pub fn all() -> Vec<NewsCategory> {
        vec![
            NewsCategory::Technology,
            NewsCategory::Business,
            NewsCategory::Science,
            NewsCategory::Health,
            NewsCategory::Sports,
            NewsCategory::Entertainment,
            NewsCategory::General,
            NewsCategory::World,
        ]
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            NewsCategory::Technology => "Technology",
            NewsCategory::Business => "Business",
            NewsCategory::Science => "Science",
            NewsCategory::Health => "Health",
            NewsCategory::Sports => "Sports",
            NewsCategory::Entertainment => "Entertainment",
            NewsCategory::General => "General",
            NewsCategory::World => "World",
        }
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            NewsCategory::Technology => "Technology news from TechCrunch, Ars Technica, The Verge",
            NewsCategory::Business => "Business news from BBC Business",
            NewsCategory::Science => "Science news from ScienceDaily",
            NewsCategory::Health => "Health news from BBC Health",
            NewsCategory::Sports => "Sports news from BBC Sport",
            NewsCategory::Entertainment => "Entertainment news from BBC Entertainment",
            NewsCategory::General => "General news from various sources",
            NewsCategory::World => "World news from BBC World",
        }
    }
}

impl std::fmt::Display for NewsCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// News article structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsArticle {
    /// Article title
    pub title: String,
    /// Article description/summary
    pub description: Option<String>,
    /// Article URL link
    pub link: String,
    /// Source name
    pub source: String,
    /// Category
    pub category: NewsCategory,
    /// Publication date
    pub published_at: Option<DateTime<Utc>>,
    /// Author name
    pub author: Option<String>,
}

impl NewsArticle {
    /// Create a new article
    pub fn new(
        title: String,
        description: Option<String>,
        link: String,
        source: String,
        category: NewsCategory,
        published_at: Option<DateTime<Utc>>,
        author: Option<String>,
    ) -> Self {
        Self {
            title,
            description,
            link,
            source,
            category,
            published_at,
            author,
        }
    }
}

/// News cache structure
#[derive(Debug)]
pub struct NewsCache {
    /// Articles stored by category
    articles: RwLock<HashMap<NewsCategory, Vec<NewsArticle>>>,
    /// Last update time for each category
    last_updated: RwLock<HashMap<NewsCategory, DateTime<Utc>>>,
    /// Maximum articles per category
    max_articles_per_category: usize,
}

impl NewsCache {
    /// Create a new news cache
    pub fn new(max_articles_per_category: usize) -> Self {
        Self {
            articles: RwLock::new(HashMap::new()),
            last_updated: RwLock::new(HashMap::new()),
            max_articles_per_category,
        }
    }

    /// Get articles for a specific category
    pub fn get_category_news(&self, category: &NewsCategory) -> Result<Vec<NewsArticle>> {
        let articles = self
            .articles
            .read()
            .map_err(|e| Error::cache(e.to_string()))?;
        Ok(articles.get(category).cloned().unwrap_or_default())
    }

    /// Set articles for a specific category
    pub fn set_category_news(
        &self,
        category: NewsCategory,
        articles: Vec<NewsArticle>,
    ) -> Result<()> {
        let mut cache = self
            .articles
            .write()
            .map_err(|e| Error::cache(e.to_string()))?;
        let limited_articles = articles
            .into_iter()
            .take(self.max_articles_per_category)
            .collect();
        cache.insert(category, limited_articles);

        let mut updated = self
            .last_updated
            .write()
            .map_err(|e| Error::cache(e.to_string()))?;
        updated.insert(category, Utc::now());

        Ok(())
    }

    /// Search articles by query string
    pub fn search(&self, query: &str, category: Option<&NewsCategory>) -> Result<Vec<NewsArticle>> {
        let articles = self
            .articles
            .read()
            .map_err(|e| Error::cache(e.to_string()))?;
        let query_lower = query.to_lowercase();

        let results: Vec<NewsArticle> = if let Some(cat) = category {
            articles
                .get(cat)
                .map(|arts| {
                    arts.iter()
                        .filter(|a| {
                            a.title.to_lowercase().contains(&query_lower)
                                || a.description
                                    .as_ref()
                                    .map(|d| d.to_lowercase().contains(&query_lower))
                                    .unwrap_or(false)
                        })
                        .cloned()
                        .collect()
                })
                .unwrap_or_default()
        } else {
            articles
                .values()
                .flat_map(|arts| arts.iter())
                .filter(|a| {
                    a.title.to_lowercase().contains(&query_lower)
                        || a.description
                            .as_ref()
                            .map(|d| d.to_lowercase().contains(&query_lower))
                            .unwrap_or(false)
                })
                .cloned()
                .collect()
        };

        Ok(results)
    }

    /// Get all available categories with article counts
    pub fn get_all_categories(&self) -> Result<Vec<(NewsCategory, usize)>> {
        let articles = self
            .articles
            .read()
            .map_err(|e| Error::cache(e.to_string()))?;
        Ok(NewsCategory::all()
            .into_iter()
            .map(|cat| {
                let count = articles.get(&cat).map(|v| v.len()).unwrap_or(0);
                (cat, count)
            })
            .collect())
    }

    /// Get last update time for a category
    pub fn get_last_updated(&self, category: &NewsCategory) -> Result<Option<DateTime<Utc>>> {
        let updated = self
            .last_updated
            .read()
            .map_err(|e| Error::cache(e.to_string()))?;
        Ok(updated.get(category).copied())
    }

    /// Get total article count across all categories
    pub fn total_article_count(&self) -> Result<usize> {
        let articles = self
            .articles
            .read()
            .map_err(|e| Error::cache(e.to_string()))?;
        Ok(articles.values().map(|v| v.len()).sum())
    }

    /// Clear all cached articles
    pub fn clear(&self) -> Result<()> {
        let mut articles = self
            .articles
            .write()
            .map_err(|e| Error::cache(e.to_string()))?;
        articles.clear();

        let mut updated = self
            .last_updated
            .write()
            .map_err(|e| Error::cache(e.to_string()))?;
        updated.clear();

        Ok(())
    }
}
