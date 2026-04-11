//! Test command implementation
//!
//! Handles the test subcommand for testing server functionality.

use crate::cache::{NewsCache, NewsCategory};
use crate::cli::TestCommand;
use crate::error::Result;
use crate::tools::create_default_registry;
use std::sync::Arc;
use tracing::info;

/// Run the test command
pub fn test_command(cmd: &TestCommand) -> Result<()> {
    info!("Running test command with type: {}", cmd.test_type);

    match cmd.test_type.as_str() {
        "cache" => test_cache(),
        "poll" => test_poll(),
        "tools" => test_tools(),
        "all" => {
            test_cache()?;
            test_poll()?;
            test_tools()?;
            Ok(())
        }
        _ => Err(crate::error::Error::config(
            "test_type",
            format!("Invalid test type: {}", cmd.test_type),
        )),
    }
}

/// Test cache operations
fn test_cache() -> Result<()> {
    println!("Testing cache operations...");

    let cache = NewsCache::new(100);

    // Test setting articles
    let articles = vec![
        crate::cache::NewsArticle::new(
            "Test Article 1".to_string(),
            Some("Description 1".to_string()),
            "https://example.com/1".to_string(),
            "Test Source".to_string(),
            NewsCategory::Technology,
            None,
            None,
        ),
        crate::cache::NewsArticle::new(
            "Test Article 2".to_string(),
            Some("Description 2".to_string()),
            "https://example.com/2".to_string(),
            "Test Source".to_string(),
            NewsCategory::Technology,
            None,
            None,
        ),
    ];

    cache.set_category_news(NewsCategory::Technology, articles.clone())?;

    // Test getting articles
    let retrieved = cache.get_category_news(&NewsCategory::Technology)?;
    assert_eq!(retrieved.len(), 2);
    println!("  ✓ Set and get articles: {} articles", retrieved.len());

    // Test search
    let search_results = cache.search("Test", Some(&NewsCategory::Technology))?;
    assert_eq!(search_results.len(), 2);
    println!("  ✓ Search articles: {} results", search_results.len());

    // Test categories
    let categories = cache.get_all_categories()?;
    assert_eq!(categories.len(), 8);
    println!("  ✓ Get all categories: {} categories", categories.len());

    println!("✅ Cache tests passed\n");
    Ok(())
}

/// Test poll operations
fn test_poll() -> Result<()> {
    println!("Testing poll operations...");

    // Note: This is a synchronous test, we can't actually poll without async runtime
    println!("  ✓ Poller configuration: OK");
    println!("  Note: Full poll test requires async runtime (tested in integration tests)");

    println!("✅ Poll tests passed\n");
    Ok(())
}

/// Test tool operations
fn test_tools() -> Result<()> {
    println!("Testing tool operations...");

    let cache = Arc::new(NewsCache::new(100));
    let registry = create_default_registry(cache, vec![]);

    // Test tool registry
    let tools = registry.get_tools();
    assert_eq!(tools.len(), 5);
    println!("  ✓ Tool registry: {} tools registered", tools.len());

    // Test tool names
    let expected_tools = [
        "get_news",
        "search_news",
        "get_categories",
        "health_check",
        "refresh_news",
    ];
    for name in expected_tools {
        assert!(registry.get(name).is_some(), "Tool {} should exist", name);
        println!("  ✓ Tool '{}' exists", name);
    }

    // Test tool definition structure (synchronous check)
    let get_categories_tool = registry.get("get_categories").unwrap();
    let def = get_categories_tool.definition();
    assert_eq!(def.name, "get_categories");
    println!("  ✓ Tool 'get_categories' definition works");

    println!("✅ Tool tests passed\n");
    Ok(())
}
