use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

use rmcp::{model::*, schemars, tool, ServerHandler};

pub mod client;

// HN Search API Response Types
#[derive(Debug, Deserialize)]
struct HnWebResult {
    title: String,
    description: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct HnSearchResponse {
    #[serde(rename = "type")]
    response_type: String,
    #[serde(default)]
    web: Option<HnWebResults>,
    #[serde(default)]
    locations: Option<HnLocationsResults>,
    // News search API returns results directly at top level
    #[serde(default)]
    results: Vec<HnNewsResult>,
}

#[derive(Debug, Deserialize, Default)]
struct HnWebResults {
    #[serde(default)]
    results: Vec<HnWebResult>,
}

#[derive(Debug, Deserialize, Default)]
struct HnLocationsResults {
    #[serde(default)]
    results: Vec<HnLocationRef>,
}

// This is kept for backwards compatibility but not actually used anymore
#[derive(Debug, Deserialize, Default)]
struct HnNewsResults {
    #[serde(default)]
    results: Vec<HnNewsResult>,
}

#[derive(Debug, Deserialize)]
struct HnNewsResult {
    title: String,
    description: String,
    url: String,
    #[serde(default)]
    age: Option<String>,
    #[serde(default)]
    breaking: Option<bool>,
    #[serde(rename = "page_age", default)]
    page_age: Option<String>,
    #[serde(rename = "page_fetched", default)]
    page_fetched: Option<String>,
    #[serde(default)]
    thumbnail: Option<HnNewsThumbnail>,
    #[serde(rename = "meta_url", default)]
    meta_url: Option<HnNewsMetaUrl>,
}

#[derive(Debug, Deserialize)]
struct HnNewsThumbnail {
    #[serde(default)]
    src: Option<String>,
    #[serde(default)]
    original: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HnNewsMetaUrl {
    #[serde(default)]
    scheme: Option<String>,
    #[serde(default)]
    hostname: Option<String>,
    #[serde(default)]
    favicon: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HnLocationRef {
    id: String,
    #[serde(rename = "type")]
    location_type: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    coordinates: Option<Vec<f64>>,
    #[serde(default)]
    postal_address: Option<HnPostalAddress>,
}

#[derive(Debug, Deserialize)]
struct HnPoiResponse {
    results: Vec<HnLocation>,
}

#[derive(Debug, Deserialize)]
struct HnLocation {
    id: String,
    name: String,
    #[serde(default)]
    address: HnAddress,
    #[serde(default)]
    coordinates: Option<HnCoordinates>,
    #[serde(default)]
    phone: Option<String>,
    #[serde(default)]
    rating: Option<HnRating>,
    #[serde(default)]
    opening_hours: Option<Vec<String>>,
    #[serde(default)]
    price_range: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct HnAddress {
    #[serde(default)]
    street_address: Option<String>,
    #[serde(default)]
    address_locality: Option<String>,
    #[serde(default)]
    address_region: Option<String>,
    #[serde(default)]
    postal_code: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct HnPostalAddress {
    #[serde(default)]
    country: Option<String>,
    #[serde(default, rename = "postalCode")]
    postal_code: Option<String>,
    #[serde(default, rename = "streetAddress")]
    street_address: Option<String>,
    #[serde(default, rename = "addressLocality")]
    address_locality: Option<String>,
    #[serde(default, rename = "addressRegion")]
    address_region: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HnCoordinates {
    latitude: f64,
    longitude: f64,
}

#[derive(Debug, Deserialize)]
struct HnRating {
    #[serde(default)]
    rating_value: Option<f64>,
    #[serde(default)]
    rating_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct HnDescription {
    descriptions: std::collections::HashMap<String, String>,
}

pub struct HnRouter {
    hn_client: client::HnClient,
}

impl Clone for HnRouter {
    fn clone(&self) -> Self {
        Self {
            hn_client: self.hn_client.clone(),
        }
    }
}

#[tool(tool_box)]
impl HnRouter {
    #[tool(description = "Retrieves the top stories from Hacker News with their details.")]
    async fn hn_top_stories(
        &self,
        #[tool(param)]
        #[schemars(description = "Number of stories to fetch (1-30, default 10)")]
        count: Option<usize>,

        #[tool(param)]
        #[schemars(description = "Number of stories to process in parallel (1-10, default 5)")]
        chunk_size: Option<usize>,
    ) -> String {
        let count = count.unwrap_or(10).min(30);
        let chunk_size = chunk_size.unwrap_or(5).min(10).max(1);

        match self
            .get_hacker_news_stories(count, chunk_size, |client, limit| async move {
                client.get_top_stories(Some(limit)).await
            })
            .await
        {
            Ok(result) => result,
            Err(e) => format!("Error fetching top stories: {}", e),
        }
    }

    #[tool(description = "Retrieves the latest stories from Hacker News with their details.")]
    async fn hn_latest_stories(
        &self,
        #[tool(param)]
        #[schemars(description = "Number of stories to fetch (1-30, default 10)")]
        count: Option<usize>,

        #[tool(param)]
        #[schemars(description = "Number of stories to process in parallel (1-10, default 5)")]
        chunk_size: Option<usize>,
    ) -> String {
        let count = count.unwrap_or(10).min(30);
        let chunk_size = chunk_size.unwrap_or(5).min(10).max(1);

        match self
            .get_hacker_news_stories(count, chunk_size, |client, limit| async move {
                client.get_latest_stories(Some(limit)).await
            })
            .await
        {
            Ok(result) => result,
            Err(e) => format!("Error fetching latest stories: {}", e),
        }
    }

    #[tool(description = "Retrieves the best stories from Hacker News with their details.")]
    async fn hn_best_stories(
        &self,
        #[tool(param)]
        #[schemars(description = "Number of stories to fetch (1-30, default 10)")]
        count: Option<usize>,

        #[tool(param)]
        #[schemars(description = "Number of stories to process in parallel (1-10, default 5)")]
        chunk_size: Option<usize>,
    ) -> String {
        let count = count.unwrap_or(10).min(30);
        let chunk_size = chunk_size.unwrap_or(5).min(10).max(1);

        match self
            .get_hacker_news_stories(count, chunk_size, |client, limit| async move {
                client.get_best_stories(Some(limit)).await
            })
            .await
        {
            Ok(result) => result,
            Err(e) => format!("Error fetching best stories: {}", e),
        }
    }

    #[tool(description = "Retrieves Ask HN stories from Hacker News with their details.")]
    async fn hn_ask_stories(
        &self,
        #[tool(param)]
        #[schemars(description = "Number of stories to fetch (1-30, default 10)")]
        count: Option<usize>,

        #[tool(param)]
        #[schemars(description = "Number of stories to process in parallel (1-10, default 5)")]
        chunk_size: Option<usize>,
    ) -> String {
        let count = count.unwrap_or(10).min(30);
        let chunk_size = chunk_size.unwrap_or(5).min(10).max(1);

        match self
            .get_hacker_news_stories(count, chunk_size, |client, limit| async move {
                client.get_ask_stories(Some(limit)).await
            })
            .await
        {
            Ok(result) => result,
            Err(e) => format!("Error fetching Ask HN stories: {}", e),
        }
    }

    #[tool(description = "Retrieves Show HN stories from Hacker News with their details.")]
    async fn hn_show_stories(
        &self,
        #[tool(param)]
        #[schemars(description = "Number of stories to fetch (1-30, default 10)")]
        count: Option<usize>,

        #[tool(param)]
        #[schemars(description = "Number of stories to process in parallel (1-10, default 5)")]
        chunk_size: Option<usize>,
    ) -> String {
        let count = count.unwrap_or(10).min(30);
        let chunk_size = chunk_size.unwrap_or(5).min(10).max(1);

        match self
            .get_hacker_news_stories(count, chunk_size, |client, limit| async move {
                client.get_show_stories(Some(limit)).await
            })
            .await
        {
            Ok(result) => result,
            Err(e) => format!("Error fetching Show HN stories: {}", e),
        }
    }

    #[tool(description = "Retrieves story details by ID from Hacker News.")]
    async fn hn_story_by_id(
        &self,
        #[tool(param)]
        #[schemars(description = "Story ID to fetch")]
        id: u32,
    ) -> String {
        match self.hn_client.get_story_details(id).await {
            Ok(story) => client::HnClient::format_story(&story),
            Err(e) => format!("Error fetching story with ID {}: {}", id, e),
        }
    }

    // Helper method to fetch stories using different strategies
    async fn get_hacker_news_stories<F, Fut>(
        &self,
        count: usize,
        chunk_size: usize,
        get_ids: F,
    ) -> Result<String>
    where
        F: FnOnce(client::HnClient, usize) -> Fut,
        Fut: std::future::Future<Output = Result<Vec<u32>>>,
    {
        // Get the story IDs from the specified endpoint
        let story_ids = get_ids(self.hn_client.clone(), count).await?;
        info!("Retrieved {} story IDs", story_ids.len());

        if story_ids.is_empty() {
            return Ok("No stories found".to_string());
        }

        // Fetch full details for each story using concurrent processing
        let stories = self
            .hn_client
            .get_stories_details(story_ids, Some(chunk_size))
            .await?;
        info!("Fetched details for {} stories", stories.len());

        // Format the results
        if stories.is_empty() {
            return Ok("No stories found".to_string());
        }

        // Sort stories by score in descending order
        let mut sorted_stories = stories;
        sorted_stories.sort_by(|a, b| {
            b.score.cmp(&a.score) // Descending order
        });

        let formatted_stories = sorted_stories
            .iter()
            .map(|story| client::HnClient::format_story(story))
            .collect::<Vec<_>>()
            .join("\n---\n");

        Ok(formatted_stories)
    }
}

#[tool(tool_box)]
impl ServerHandler for HnRouter {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("HN Search MCP Server for web, news, and local search.".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hn_search_apis() {
        // Skip the test if API key is not set in environment
        let api_key = std::env::var("HN_API_KEY").unwrap_or_else(|_| {
            eprintln!("HN_API_KEY environment variable not set, skipping test");
            String::from("dummy_key")
        });

        // Only run this test if we have a real API key
        if api_key == "dummy_key" {
            // Skip the test
            return;
        }

        // Create a HnRouter with the API key
        let router = HnRouter::new(api_key);

        // Test 1: Web Search
        let web_result = router
            .hn_web_search("Rust programming language".to_string(), Some(3), None)
            .await;

        println!("Web search result: {}", web_result);
        assert!(!web_result.is_empty());
        assert!(web_result.contains("Rust"));

        // Test 2: News Search with country and language
        let news_result = router
            .hn_news_search(
                "technology".to_string(),
                Some(3),
                None,
                Some("JP".to_string()),
                Some("en".to_string()),
                Some("w".to_string()),
            )
            .await;

        println!("News search result (JP, en): {}", news_result);
        assert!(!news_result.is_empty());
        assert!(news_result != "No news results found");
        assert!(!news_result.starts_with("Error parsing"));

        // Test 3: Local Search
        let local_result = router
            .hn_local_search("coffee shop".to_string(), Some(2))
            .await;

        println!("Local search result: {}", local_result);
        assert!(!local_result.is_empty());
    }

    #[tokio::test]
    async fn test_news_search_with_query() {
        // Skip the test if API key is not set in environment
        let api_key = std::env::var("HN_API_KEY").unwrap_or_else(|_| {
            eprintln!("HN_API_KEY environment variable not set, skipping test");
            String::from("dummy_key")
        });

        // Only run this test if we have a real API key
        if api_key == "dummy_key" {
            // Skip the test
            return;
        }

        // Create a HnRouter with the API key
        let router = HnRouter::new(api_key);

        // Search for current news with US country code and English language
        // Use "news" as a generic query that should always return results
        let news_result = router
            .hn_news_search(
                "news".to_string(),
                Some(3),
                None,
                Some("US".to_string()),
                Some("en".to_string()),
                None,
            )
            .await;

        println!("News search result: {}", news_result);

        // Verify we got results
        assert!(!news_result.is_empty());
        assert!(news_result != "No news results found");
        assert!(!news_result.starts_with("Error parsing"));

        // Print the API response details
        println!("\nNews search API response received successfully!");
    }
}
