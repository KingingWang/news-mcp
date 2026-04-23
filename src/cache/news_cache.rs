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
    Science,
    HackerNews,
    // China News categories
    Instant,
    Headlines,
    Politics,
    EastWest,
    Society,
    Finance,
    Life,
    Wellness,
    GreaterBayArea,
    Chinese,
    Video,
    Photo,
    Creative,
    Live,
    Education,
    Law,
    UnitedFront,
    EthnicUnity,
    Theory,
    Asean,
    // NewsNow Hot List categories
    WeiboHot,
    BaiduHot,
    ZhihuHot,
    DouyinHot,
    BilibiliHot,
    TiebaHot,
    ToutiaoHot,
    WallstreetcnHot,
    ClsHot,
    ThepaperHot,
    IfengHot,
}

impl std::str::FromStr for NewsCategory {
    type Err = Error;

    fn from_str(s: &str) -> Result<NewsCategory> {
        match s.to_lowercase().as_str() {
            "technology" | "tech" => Ok(NewsCategory::Technology),
            "science" => Ok(NewsCategory::Science),
            "hackernews" | "hn" => Ok(NewsCategory::HackerNews),
            // China News categories
            "instant" | "即时新闻" => Ok(NewsCategory::Instant),
            "headlines" | "要闻导读" => Ok(NewsCategory::Headlines),
            "politics" | "时政新闻" => Ok(NewsCategory::Politics),
            "eastwest" | "东西问" => Ok(NewsCategory::EastWest),
            "society" | "社会新闻" => Ok(NewsCategory::Society),
            "finance" | "财经新闻" => Ok(NewsCategory::Finance),
            "life" | "生活" => Ok(NewsCategory::Life),
            "wellness" | "健康" => Ok(NewsCategory::Wellness),
            "greaterbayarea" | "大湾区" => Ok(NewsCategory::GreaterBayArea),
            "chinese" | "华人" => Ok(NewsCategory::Chinese),
            "video" | "视频" => Ok(NewsCategory::Video),
            "photo" | "图片" => Ok(NewsCategory::Photo),
            "creative" | "创意" => Ok(NewsCategory::Creative),
            "live" | "直播" => Ok(NewsCategory::Live),
            "education" | "教育" => Ok(NewsCategory::Education),
            "law" | "法治" => Ok(NewsCategory::Law),
            "unitedfront" | "同心" => Ok(NewsCategory::UnitedFront),
            "ethnicunity" | "铸牢中华民族共同体意识" => Ok(NewsCategory::EthnicUnity),
            "theory" | "理论" => Ok(NewsCategory::Theory),
            "asean" | "中国—东盟商贸资讯平台" => Ok(NewsCategory::Asean),
            // NewsNow Hot List categories
            "weibohot" | "微博热搜" => Ok(NewsCategory::WeiboHot),
            "baiduhot" | "百度热搜" => Ok(NewsCategory::BaiduHot),
            "zhihuhot" | "知乎热榜" => Ok(NewsCategory::ZhihuHot),
            "douyinhot" | "抖音热点" => Ok(NewsCategory::DouyinHot),
            "bilibilihot" | "b站热搜" => Ok(NewsCategory::BilibiliHot),
            "tiebahot" | "贴吧热议" => Ok(NewsCategory::TiebaHot),
            "toutiaohot" | "今日头条热点" => Ok(NewsCategory::ToutiaoHot),
            "wallstreetcnhot" | "华尔街见闻热门" => Ok(NewsCategory::WallstreetcnHot),
            "clshot" | "财联社热门" => Ok(NewsCategory::ClsHot),
            "thepaperhot" | "澎湃热门" => Ok(NewsCategory::ThepaperHot),
            "ifenghot" | "凤凰网热门" => Ok(NewsCategory::IfengHot),
            _ => Err(Error::invalid_category(s)),
        }
    }
}

impl NewsCategory {
    /// Get all categories
    pub fn all() -> Vec<NewsCategory> {
        vec![
            NewsCategory::Technology,
            NewsCategory::Science,
            NewsCategory::HackerNews,
            // China News categories
            NewsCategory::Instant,
            NewsCategory::Headlines,
            NewsCategory::Politics,
            NewsCategory::EastWest,
            NewsCategory::Society,
            NewsCategory::Finance,
            NewsCategory::Life,
            NewsCategory::Wellness,
            NewsCategory::GreaterBayArea,
            NewsCategory::Chinese,
            NewsCategory::Video,
            NewsCategory::Photo,
            NewsCategory::Creative,
            NewsCategory::Live,
            NewsCategory::Education,
            NewsCategory::Law,
            NewsCategory::UnitedFront,
            NewsCategory::EthnicUnity,
            NewsCategory::Theory,
            NewsCategory::Asean,
            // NewsNow Hot List categories
            NewsCategory::WeiboHot,
            NewsCategory::BaiduHot,
            NewsCategory::ZhihuHot,
            NewsCategory::DouyinHot,
            NewsCategory::BilibiliHot,
            NewsCategory::TiebaHot,
            NewsCategory::ToutiaoHot,
            NewsCategory::WallstreetcnHot,
            NewsCategory::ClsHot,
            NewsCategory::ThepaperHot,
            NewsCategory::IfengHot,
        ]
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            NewsCategory::Technology => "Technology",
            NewsCategory::Science => "Science",
            NewsCategory::HackerNews => "Hacker News",
            // China News categories
            NewsCategory::Instant => "即时新闻",
            NewsCategory::Headlines => "要闻导读",
            NewsCategory::Politics => "时政新闻",
            NewsCategory::EastWest => "东西问",
            NewsCategory::Society => "社会新闻",
            NewsCategory::Finance => "财经新闻",
            NewsCategory::Life => "生活",
            NewsCategory::Wellness => "健康",
            NewsCategory::GreaterBayArea => "大湾区",
            NewsCategory::Chinese => "华人",
            NewsCategory::Video => "视频",
            NewsCategory::Photo => "图片",
            NewsCategory::Creative => "创意",
            NewsCategory::Live => "直播",
            NewsCategory::Education => "教育",
            NewsCategory::Law => "法治",
            NewsCategory::UnitedFront => "同心",
            NewsCategory::EthnicUnity => "铸牢中华民族共同体意识",
            NewsCategory::Theory => "理论",
            NewsCategory::Asean => "中国—东盟商贸资讯平台",
            // NewsNow Hot List categories
            NewsCategory::WeiboHot => "微博热搜",
            NewsCategory::BaiduHot => "百度热搜",
            NewsCategory::ZhihuHot => "知乎热榜",
            NewsCategory::DouyinHot => "抖音热点",
            NewsCategory::BilibiliHot => "B站热搜",
            NewsCategory::TiebaHot => "贴吧热议",
            NewsCategory::ToutiaoHot => "今日头条热点",
            NewsCategory::WallstreetcnHot => "华尔街见闻热门",
            NewsCategory::ClsHot => "财联社热门",
            NewsCategory::ThepaperHot => "澎湃热门",
            NewsCategory::IfengHot => "凤凰网热门",
        }
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            NewsCategory::Technology => "Technology news from TechCrunch, Ars Technica, The Verge",
            NewsCategory::Science => "Science news from ScienceDaily",
            NewsCategory::HackerNews => "Top stories from Hacker News",
            // China News categories
            NewsCategory::Instant => "即时新闻 - 中国新闻网滚动新闻",
            NewsCategory::Headlines => "要闻导读 - 中国新闻网重要新闻",
            NewsCategory::Politics => "时政新闻 - 中国新闻网时政要闻",
            NewsCategory::EastWest => "东西问 - 中国新闻网文化对话",
            NewsCategory::Society => "社会新闻 - 中国新闻网社会百态",
            NewsCategory::Finance => "财经新闻 - 中国新闻网财经资讯",
            NewsCategory::Life => "生活 - 中国新闻网生活服务",
            NewsCategory::Wellness => "健康 - 中国新闻网健康资讯",
            NewsCategory::GreaterBayArea => "大湾区 - 中国新闻网粤港澳大湾区",
            NewsCategory::Chinese => "华人 - 中国新闻网海外华人",
            NewsCategory::Video => "视频 - 中国新闻网视频新闻",
            NewsCategory::Photo => "图片 - 中国新闻网图片新闻",
            NewsCategory::Creative => "创意 - 中国新闻网创意产业",
            NewsCategory::Live => "直播 - 中国新闻网直播报道",
            NewsCategory::Education => "教育 - 中国新闻网教育资讯",
            NewsCategory::Law => "法治 - 中国新闻网法治新闻",
            NewsCategory::UnitedFront => "同心 - 中国新闻网统战新闻",
            NewsCategory::EthnicUnity => "铸牢中华民族共同体意识 - 中国新闻网民族新闻",
            NewsCategory::Theory => "理论 - 中国新闻网理论动态",
            NewsCategory::Asean => "中国—东盟商贸资讯平台 - 中国新闻网东盟资讯",
            // NewsNow Hot List categories
            NewsCategory::WeiboHot => "微博热搜 - 实时热搜榜",
            NewsCategory::BaiduHot => "百度热搜 - 百度实时热搜",
            NewsCategory::ZhihuHot => "知乎热榜 - 知乎热门话题",
            NewsCategory::DouyinHot => "抖音热点 - 抖音热门视频",
            NewsCategory::BilibiliHot => "B站热搜 - 哔哩哔哩热搜榜",
            NewsCategory::TiebaHot => "贴吧热议 - 百度贴吧热议话题",
            NewsCategory::ToutiaoHot => "今日头条热点 - 头条热门资讯",
            NewsCategory::WallstreetcnHot => "华尔街见闻热门 - 财经资讯",
            NewsCategory::ClsHot => "财联社热门 - 金融快讯",
            NewsCategory::ThepaperHot => "澎湃热门 - 澎湃新闻热点",
            NewsCategory::IfengHot => "凤凰网热门 - 凤凰资讯热点",
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
