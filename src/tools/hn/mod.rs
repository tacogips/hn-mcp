use anyhow::Result;
use tracing::info;

use rmcp::{model::*, schemars, tool, ServerHandler};

pub mod client;

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
    pub fn new(hn_client: client::HnClient) -> Self {
        Self { hn_client }
    }
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
        let chunk_size = chunk_size.unwrap_or(5).clamp(1, 10);

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
        let chunk_size = chunk_size.unwrap_or(5).clamp(1, 10);

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
        let chunk_size = chunk_size.unwrap_or(5).clamp(1, 10);

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
        let chunk_size = chunk_size.unwrap_or(5).clamp(1, 10);

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
        let chunk_size = chunk_size.unwrap_or(5).clamp(1, 10);

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
            .map(client::HnClient::format_story)
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
