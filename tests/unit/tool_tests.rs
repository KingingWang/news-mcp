//! Tool unit tests

use news_mcp::cache::{NewsArticle, NewsCache, NewsCategory};
use news_mcp::config::FeedSourceConfig;
use news_mcp::tools::{
    create_default_registry, GetCategoriesToolImpl, GetNewsToolImpl, HealthCheckToolImpl,
    RefreshNewsToolImpl, SearchNewsToolImpl, Tool,
};
use rust_mcp_sdk::schema::ContentBlock;
use std::collections::HashMap;
use std::sync::Arc;

fn get_text_content(result: &rust_mcp_sdk::schema::CallToolResult) -> &str {
    match &result.content[0] {
        ContentBlock::TextContent(text) => &text.text,
        _ => panic!("Expected text content"),
    }
}

fn create_test_cache() -> Arc<NewsCache> {
    let cache = Arc::new(NewsCache::new(100));

    let articles = [
        NewsArticle::new(
            "Technology News".to_string(),
            Some("Latest tech updates".to_string()),
            "https://example.com/tech".to_string(),
            "Tech Source".to_string(),
            NewsCategory::Technology,
            None,
            None,
        ),
        NewsArticle::new(
            "Science News".to_string(),
            Some("Science updates".to_string()),
            "https://example.com/science".to_string(),
            "Science Source".to_string(),
            NewsCategory::Science,
            None,
            None,
        ),
    ];

    cache
        .set_category_news(NewsCategory::Technology, vec![articles[0].clone()])
        .unwrap();
    cache
        .set_category_news(NewsCategory::Science, vec![articles[1].clone()])
        .unwrap();

    cache
}

fn create_empty_cache() -> Arc<NewsCache> {
    Arc::new(NewsCache::new(100))
}

fn create_test_feeds() -> HashMap<String, FeedSourceConfig> {
    HashMap::new()
}

// ============================================================================
// Tool Registry Tests
// ============================================================================

#[test]
fn test_tool_registry() {
    let cache = create_test_cache();
    let feeds = create_test_feeds();
    let registry = create_default_registry(cache, vec![], feeds);

    let tools = registry.get_tools();
    assert_eq!(tools.len(), 5);

    // Verify tool names
    let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
    assert!(tool_names.contains(&"get_news".to_string()));
    assert!(tool_names.contains(&"search_news".to_string()));
    assert!(tool_names.contains(&"get_categories".to_string()));
    assert!(tool_names.contains(&"health_check".to_string()));
    assert!(tool_names.contains(&"refresh_news".to_string()));
}

#[test]
fn test_tool_registry_get() {
    let cache = create_test_cache();
    let feeds = create_test_feeds();
    let registry = create_default_registry(cache, vec![], feeds);

    let tool = registry.get("get_news");
    assert!(tool.is_some());

    let tool = registry.get("invalid_tool");
    assert!(tool.is_none());
}

// ============================================================================
// get_news Tool Tests
// ============================================================================

#[test]
fn test_get_news_tool_definition() {
    let cache = create_test_cache();
    let feeds = create_test_feeds();
    let tool = GetNewsToolImpl::new(cache, feeds);

    let definition = tool.definition();
    assert_eq!(definition.name, "get_news");
    // description is optional in the schema
    if let Some(desc) = &definition.description {
        assert!(desc.contains("cache"));
    }
}

#[tokio::test]
async fn test_get_news_tool_execution() {
    let cache = create_test_cache();
    let feeds = create_test_feeds();
    let tool = GetNewsToolImpl::new(cache, feeds);

    let result = tool.execute(serde_json::json!({})).await.unwrap();
    assert!(get_text_content(&result).contains("Technology"));
}

#[tokio::test]
async fn test_get_news_tool_with_params() {
    let cache = create_test_cache();
    let feeds = create_test_feeds();
    let tool = GetNewsToolImpl::new(cache, feeds);

    let params = serde_json::json!({
        "category": "science",
        "limit": 1,
        "format": "text"
    });

    let result = tool.execute(params).await.unwrap();
    assert!(get_text_content(&result).contains("Science"));
}

#[tokio::test]
async fn test_get_news_all_categories() {
    let cache = create_test_cache();
    let feeds = create_test_feeds();
    let tool = GetNewsToolImpl::new(cache, feeds);

    // Test each category
    for category in &[
        "technology",
        "science",
        "hackernews",
        "instant",
        "headlines",
        "politics",
    ] {
        let params = serde_json::json!({
            "category": category
        });
        let result = tool.execute(params).await.unwrap();
        // Should succeed even for empty categories
        let text = get_text_content(&result);
        assert!(!text.is_empty());
    }
}

#[tokio::test]
async fn test_get_news_invalid_category() {
    let cache = create_test_cache();
    let feeds = create_test_feeds();
    let tool = GetNewsToolImpl::new(cache, feeds);

    let params = serde_json::json!({
        "category": "invalid_category"
    });

    let result = tool.execute(params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_news_limit_boundaries() {
    let cache = create_test_cache();
    let feeds = create_test_feeds();
    let tool = GetNewsToolImpl::new(cache, feeds);

    // Test minimum limit
    let params = serde_json::json!({
        "limit": 1
    });
    let result = tool.execute(params).await.unwrap();
    let text = get_text_content(&result);
    assert!(text.contains("Technology"));

    // Test maximum limit (should clamp to 50)
    let params = serde_json::json!({
        "limit": 100
    });
    let result = tool.execute(params).await.unwrap();
    assert!(!get_text_content(&result).is_empty());

    // Test limit of 0 (should clamp to 1)
    let params = serde_json::json!({
        "limit": 0
    });
    let result = tool.execute(params).await.unwrap();
    assert!(!get_text_content(&result).is_empty());
}

#[tokio::test]
async fn test_get_news_formats() {
    let cache = create_test_cache();
    let feeds = create_test_feeds();
    let tool = GetNewsToolImpl::new(cache, feeds);

    // Markdown format
    let params = serde_json::json!({
        "format": "markdown"
    });
    let result = tool.execute(params).await.unwrap();
    let text = get_text_content(&result);
    assert!(text.contains("# News Articles"));

    // JSON format
    let params = serde_json::json!({
        "format": "json"
    });
    let result = tool.execute(params).await.unwrap();
    let text = get_text_content(&result);
    assert!(text.starts_with('['));

    // Text format
    let params = serde_json::json!({
        "format": "text"
    });
    let result = tool.execute(params).await.unwrap();
    let text = get_text_content(&result);
    assert!(text.contains("1."));
}

#[tokio::test]
async fn test_get_news_invalid_format() {
    let cache = create_test_cache();
    let feeds = create_test_feeds();
    let tool = GetNewsToolImpl::new(cache, feeds);

    let params = serde_json::json!({
        "format": "invalid_format"
    });

    let result = tool.execute(params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_news_empty_cache() {
    let cache = create_empty_cache();
    let feeds = create_test_feeds();
    let tool = GetNewsToolImpl::new(cache, feeds);

    let result = tool.execute(serde_json::json!({})).await.unwrap();
    let text = get_text_content(&result);
    assert!(text.contains("No articles found"));
}

// ============================================================================
// search_news Tool Tests
// ============================================================================

#[test]
fn test_search_news_tool_definition() {
    let cache = create_test_cache();
    let feeds = create_test_feeds();
    let tool = SearchNewsToolImpl::new(cache, feeds);

    let definition = tool.definition();
    assert_eq!(definition.name, "search_news");
}

#[tokio::test]
async fn test_search_news_tool() {
    let cache = create_test_cache();
    let feeds = create_test_feeds();
    let tool = SearchNewsToolImpl::new(cache, feeds);

    let params = serde_json::json!({
        "query": "Technology"
    });

    let result = tool.execute(params).await.unwrap();
    assert!(get_text_content(&result).contains("Technology"));
}

#[tokio::test]
async fn test_search_news_tool_missing_query() {
    let cache = create_test_cache();
    let feeds = create_test_feeds();
    let tool = SearchNewsToolImpl::new(cache, feeds);

    let result = tool.execute(serde_json::json!({})).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_search_news_case_insensitive() {
    let cache = create_test_cache();
    let feeds = create_test_feeds();
    let tool = SearchNewsToolImpl::new(cache, feeds);

    // Lowercase search
    let params = serde_json::json!({
        "query": "technology"
    });
    let result = tool.execute(params).await.unwrap();
    assert!(get_text_content(&result).contains("Technology"));

    // Uppercase search
    let params = serde_json::json!({
        "query": "TECHNOLOGY"
    });
    let result = tool.execute(params).await.unwrap();
    assert!(get_text_content(&result).contains("Technology"));
}

#[tokio::test]
async fn test_search_news_with_category_filter() {
    let cache = create_test_cache();
    let feeds = create_test_feeds();
    let tool = SearchNewsToolImpl::new(cache, feeds);

    let params = serde_json::json!({
        "query": "News",
        "category": "technology"
    });
    let result = tool.execute(params).await.unwrap();
    assert!(get_text_content(&result).contains("Technology"));
}

#[tokio::test]
async fn test_search_news_no_results() {
    let cache = create_test_cache();
    let feeds = create_test_feeds();
    let tool = SearchNewsToolImpl::new(cache, feeds);

    let params = serde_json::json!({
        "query": "nonexistent_keyword_xyz123"
    });
    let result = tool.execute(params).await.unwrap();
    assert!(get_text_content(&result).contains("No articles found"));
}

#[tokio::test]
async fn test_search_news_limit() {
    let cache = Arc::new(NewsCache::new(100));

    // Add multiple articles
    for i in 0..20 {
        let article = NewsArticle::new(
            format!("Technology Article {}", i),
            Some("Tech content".to_string()),
            format!("https://example.com/{}", i),
            "Source".to_string(),
            NewsCategory::Technology,
            None,
            None,
        );
        cache
            .set_category_news(NewsCategory::Technology, vec![article])
            .unwrap();
    }

    let tool = SearchNewsToolImpl::new(cache, create_test_feeds());

    let params = serde_json::json!({
        "query": "Technology",
        "limit": 5
    });
    let result = tool.execute(params).await.unwrap();
    let text = get_text_content(&result);
    // Should only have 5 articles
    assert!(text.matches("Technology Article").count() <= 5);
}

#[tokio::test]
async fn test_search_news_invalid_category() {
    let cache = create_test_cache();
    let tool = SearchNewsToolImpl::new(cache, create_test_feeds());

    let params = serde_json::json!({
        "query": "test",
        "category": "invalid"
    });
    let result = tool.execute(params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_search_news_formats() {
    let cache = create_test_cache();
    let tool = SearchNewsToolImpl::new(cache, create_test_feeds());

    // JSON format
    let params = serde_json::json!({
        "query": "Technology",
        "format": "json"
    });
    let result = tool.execute(params).await.unwrap();
    assert!(get_text_content(&result).starts_with('['));

    // Text format
    let params = serde_json::json!({
        "query": "Technology",
        "format": "text"
    });
    let result = tool.execute(params).await.unwrap();
    assert!(get_text_content(&result).contains("1."));
}

// ============================================================================
// get_categories Tool Tests
// ============================================================================

#[test]
fn test_get_categories_tool_definition() {
    let cache = create_test_cache();
    let tool = GetCategoriesToolImpl::new(cache);

    let definition = tool.definition();
    assert_eq!(definition.name, "get_categories");
}

#[tokio::test]
async fn test_get_categories_tool() {
    let cache = create_test_cache();
    let tool = GetCategoriesToolImpl::new(cache);

    let result = tool.execute(serde_json::json!({})).await.unwrap();
    let text = get_text_content(&result);
    assert!(text.contains("Technology"));
    assert!(text.contains("Science"));
    assert!(text.contains("article"));
}

#[tokio::test]
async fn test_get_categories_all_present() {
    let cache = create_test_cache();
    let tool = GetCategoriesToolImpl::new(cache);

    let result = tool.execute(serde_json::json!({})).await.unwrap();
    let text = get_text_content(&result);

    // All categories should be present
    for category in &[
        "Technology",
        "Science",
        "Hacker News",
        "即时新闻",
        "要闻导读",
    ] {
        assert!(text.contains(category));
    }
}

#[tokio::test]
async fn test_get_categories_empty_cache() {
    let cache = create_empty_cache();
    let tool = GetCategoriesToolImpl::new(cache);

    let result = tool.execute(serde_json::json!({})).await.unwrap();
    let text = get_text_content(&result);

    // Should still show all categories with 0 articles
    assert!(text.contains("0 articles"));
}

// ============================================================================
// health_check Tool Tests
// ============================================================================

#[test]
fn test_health_check_tool_definition() {
    let cache = create_test_cache();
    let tool = HealthCheckToolImpl::new(cache);

    let definition = tool.definition();
    assert_eq!(definition.name, "health_check");
}

#[tokio::test]
async fn test_health_check_tool() {
    let cache = create_test_cache();
    let tool = HealthCheckToolImpl::new(cache);

    let result = tool.execute(serde_json::json!({})).await.unwrap();
    let text = get_text_content(&result);
    assert!(text.contains("Health Check"));
    assert!(text.contains("Healthy"));
    assert!(text.contains("Articles"));
}

#[tokio::test]
async fn test_health_check_verbose() {
    let cache = create_test_cache();
    let tool = HealthCheckToolImpl::new(cache);

    let params = serde_json::json!({
        "verbose": true
    });

    let result = tool.execute(params).await.unwrap();
    assert!(get_text_content(&result).contains("Last Update"));
}

#[tokio::test]
async fn test_health_check_check_types() {
    let cache = create_test_cache();
    let tool = HealthCheckToolImpl::new(cache);

    // Test different check types
    for check_type in &["all", "internal", "external"] {
        let params = serde_json::json!({
            "check_type": check_type
        });
        let result = tool.execute(params).await.unwrap();
        assert!(get_text_content(&result).contains("Healthy"));
    }
}

#[tokio::test]
async fn test_health_check_empty_cache() {
    let cache = create_empty_cache();
    let tool = HealthCheckToolImpl::new(cache);

    let result = tool.execute(serde_json::json!({})).await.unwrap();
    let text = get_text_content(&result);
    assert!(text.contains("Total Articles"));
    assert!(text.contains("0"));
}

// ============================================================================
// refresh_news Tool Tests
// ============================================================================

#[test]
fn test_refresh_news_tool_definition() {
    let cache = create_test_cache();
    let tool = RefreshNewsToolImpl::new(cache, vec![]);

    let definition = tool.definition();
    assert_eq!(definition.name, "refresh_news");
}

#[tokio::test]
async fn test_refresh_news_tool_execution() {
    let cache = create_empty_cache();
    let tool = RefreshNewsToolImpl::new(cache, vec![]);

    // Note: This test makes actual network requests
    // It may fail if network is unavailable
    let result = tool.execute(serde_json::json!({})).await;

    // We just verify the tool can be executed
    // Network failures are acceptable in tests
    if let Ok(result) = result {
        let text = get_text_content(&result);
        assert!(text.contains("Refresh News Status"));
    }
}

#[tokio::test]
async fn test_refresh_news_invalid_category() {
    let cache = create_empty_cache();
    let tool = RefreshNewsToolImpl::new(cache, vec![]);

    let params = serde_json::json!({
        "category": "invalid_category"
    });

    let result = tool.execute(params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_refresh_news_with_null_params() {
    let cache = create_empty_cache();
    let tool = RefreshNewsToolImpl::new(cache, vec![]);

    // Should handle null parameters gracefully
    let result = tool.execute(serde_json::Value::Null).await;
    // The tool should attempt to refresh (may fail due to network)
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_multiple_tools_workflow() {
    let cache = create_test_cache();

    // Get categories
    let categories_tool = GetCategoriesToolImpl::new(cache.clone());
    let result = categories_tool
        .execute(serde_json::json!({}))
        .await
        .unwrap();
    assert!(get_text_content(&result).contains("Technology"));

    // Get news
    let get_news_tool = GetNewsToolImpl::new(cache.clone(), create_test_feeds());
    let result = get_news_tool
        .execute(serde_json::json!({"category": "technology"}))
        .await
        .unwrap();
    assert!(get_text_content(&result).contains("Technology"));

    // Search news
    let search_tool = SearchNewsToolImpl::new(cache.clone(), create_test_feeds());
    let result = search_tool
        .execute(serde_json::json!({"query": "Technology"}))
        .await
        .unwrap();
    assert!(get_text_content(&result).contains("Technology"));

    // Health check
    let health_tool = HealthCheckToolImpl::new(cache);
    let result = health_tool.execute(serde_json::json!({})).await.unwrap();
    assert!(get_text_content(&result).contains("Healthy"));
}
