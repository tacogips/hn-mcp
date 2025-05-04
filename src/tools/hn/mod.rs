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
    #[tool(description = "Retrieves the top trending stories from Hacker News (HN is the common abbreviation for Hacker News) with their complete details including title, URL, text, author, score, date, and comment count. Results are sorted by score in descending order. Example: `hn_top_stories(count=3)` returns the three highest-scored stories currently trending on HN, displaying their full details including URLs and comment counts.")]
    async fn hn_top_stories(
        &self,
        #[tool(param)]
        #[schemars(description = "Number of stories to fetch (1-30, default 10). Controls how many top stories will be returned. Example: 5 will return the 5 highest-scoring top stories. Higher values provide more comprehensive results but take longer to process.")]
        count: Option<usize>,

        #[tool(param)]
        #[schemars(description = "Number of stories to process in parallel (1-10, default 5). Higher values may speed up retrieval but increase API load. Example: 10 for maximum concurrency, 3 for lighter load on the API. This affects performance but not the actual results.")]
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

    #[tool(description = "Retrieves the most recently submitted stories from Hacker News (HN is the common abbreviation for Hacker News) with their complete details including title, URL, text, author, score, date, and comment count. Useful for discovering brand new content that hasn't been widely seen yet. Results are sorted by score in descending order. Example: `hn_latest_stories(count=2)` would return content like 'Ask HN: Why is Reddit down?' (Score: 42) and 'The Future of Rust Web Development' (Score: 37) that were just submitted minutes ago.")]
    async fn hn_latest_stories(
        &self,
        #[tool(param)]
        #[schemars(description = "Number of stories to fetch (1-30, default 10). Controls how many latest stories will be returned. Example: 15 will return the 15 most recent stories, while 3 will focus only on the very newest submissions with highest scores.")]
        count: Option<usize>,

        #[tool(param)]
        #[schemars(description = "Number of stories to process in parallel (1-10, default 5). Higher values may speed up retrieval but increase API load. Example: 8 for faster retrieval, 2 for minimal API impact. This is particularly useful when fetching many stories at once.")]
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

    #[tool(description = "Retrieves the highest-quality stories from Hacker News (HN is the common abbreviation for Hacker News) based on a combination of score, comments, and other factors. Returns complete details including title, URL, text, author, score, date, and comment count. Best for finding the most interesting content over a longer time period. Results are sorted by score in descending order. Example: `hn_best_stories(count=2)` might return stories like 'Show HN: Structify – Convert unstructured text to structured data with AI' (Score: 943) and 'The History of Programming Languages Visualized' (Score: 876) that have gained significant attention over days.")]
    async fn hn_best_stories(
        &self,
        #[tool(param)]
        #[schemars(description = "Number of stories to fetch (1-30, default 10). Controls how many best stories will be returned. Example: 20 will return the 20 highest-quality stories from recent days, while 5 will focus only on the absolute best content. With count=1, you'll get the single highest-quality story.")]
        count: Option<usize>,

        #[tool(param)]
        #[schemars(description = "Number of stories to process in parallel (1-10, default 5). Higher values may speed up retrieval but increase API load. Example: 7 for balanced performance, 4 for slightly reduced load. Setting chunk_size=1 processes sequentially but puts minimal load on the API.")]
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

    #[tool(description = "Retrieves 'Ask HN' question posts from Hacker News (HN is the common abbreviation for Hacker News) where users ask the community for advice, opinions, or information. Returns complete details including title, text, author, score, date, and comment count. Particularly useful for finding discussions, questions, and community interactions. Results are sorted by score in descending order. Example: `hn_ask_stories(count=2)` might return questions like 'Ask HN: What productivity tools do you use in 2025?' (Score: 183, Comments: 207) and 'Ask HN: How are you using the new GPT-4o in your workflow?' (Score: 156, Comments: 142).")]
    async fn hn_ask_stories(
        &self,
        #[tool(param)]
        #[schemars(description = "Number of stories to fetch (1-30, default 10). Controls how many Ask HN stories will be returned. Example: 12 will return the 12 highest-scoring Ask HN stories. Setting count=30 will give you the most comprehensive view of current community questions. Popular Ask HN posts often have many comments, making them valuable for research.")]
        count: Option<usize>,

        #[tool(param)]
        #[schemars(description = "Number of stories to process in parallel (1-10, default 5). Higher values may speed up retrieval but increase API load. Example: 6 for moderate concurrency. For Ask HN stories, which often contain more text content, a moderate chunk_size of 4-6 is generally optimal for balanced performance.")]
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

    #[tool(description = "Retrieves 'Show HN' posts from Hacker News (HN is the common abbreviation for Hacker News) where users showcase their projects, websites, apps, or creations to get feedback from the community. Returns complete details including title, URL, text, author, score, date, and comment count. Ideal for discovering new projects and innovations. Results are sorted by score in descending order. Example: `hn_show_stories(count=2)` might return projects like 'Show HN: Structify – Convert unstructured text to structured data with AI' (URL: https://github.com/structify/structify) and 'Show HN: LocalLLM – Run powerful language models on consumer hardware' (URL: https://localllm.ai).")]
    async fn hn_show_stories(
        &self,
        #[tool(param)]
        #[schemars(description = "Number of stories to fetch (1-30, default 10). Controls how many Show HN stories will be returned. Example: 10 will return the 10 highest-scoring Show HN stories. For discovering the widest range of new projects, try count=25, while for finding only the most popular showcases, try count=3. Show HN posts typically include project URLs and descriptions.")]
        count: Option<usize>,

        #[tool(param)]
        #[schemars(description = "Number of stories to process in parallel (1-10, default 5). Higher values may speed up retrieval but increase API load. Example: 5 for default concurrency. Since Show HN posts often include links to external sites, a moderate chunk_size of 5 balances speed and API load effectively.")]
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

    #[tool(description = "Retrieves complete details of a specific Hacker News (HN is the common abbreviation for Hacker News) story by its unique ID. Returns all available information including title, URL, text, author, score, date, and comment count. Use this when you have a specific story ID and need to fetch its contents. Example: `hn_story_by_id(id=39617316)` returns the full details of that specific story ('Show HN: GPT-4o 10x faster for me using Alt+Enter vs Enter').")]
    async fn hn_story_by_id(
        &self,
        #[tool(param)]
        #[schemars(description = "Numeric ID of the Hacker News story to fetch. Every HN story has a unique ID which can be found in story listings or URLs. Example: 39617316 (a Show HN post about GPT-4o) or 39617842 (an Ask HN post about productivity tools). These IDs are visible in the output of other HN tool functions or can be found in HN URLs.")]
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
            instructions: Some("Hacker News (HN) MCP Server providing access to content categories from Hacker News (HN), a popular tech-focused news aggregation site. Note: 'HN' is commonly used as an abbreviation for 'Hacker News' in function names and throughout this documentation. This server provides access to top, latest, best, Ask HN, and Show HN stories. Supports retrieval by story ID and concurrent processing for efficiency.

## Example Usage with Input/Output:

1. Get top stories: 
   ```
   Input: hn_top_stories(count=3)
   Output:
   Title: Show HN: GPT-4o 10x faster for me using Alt+Enter vs Enter 
   URL: https://twitter.com/tinkergoblin/status/1790778491434525211
   By: tinkergoblin
   Score: 256
   Date: 2025-05-04 15:43:20.000 +00:00:00
   Comments: 89
   ID: 39617316
   ---
   Title: Find My Apple Watch
   URL: https://support.apple.com/en-us/108602
   By: andygambles
   Score: 214
   Date: 2025-05-04 14:03:11.000 +00:00:00
   Comments: 132
   ID: 39617052
   ---
   Title: OpenAI has been training GPT-5 since December 2023
   URL: https://www.theverge.com/2025/5/4/24142756/openai-has-been-training-gpt-5-since-december-2023
   By: skilled
   Score: 187
   Date: 2025-05-04 23:17:43.000 +00:00:00
   Comments: 74
   ID: 39618653
   ```

2. Get latest stories with parallelism:
   ```
   Input: hn_latest_stories(count=2, chunk_size=2)
   Output:
   Title: Ask HN: Why is Reddit down?
   Text: The site seems to be experiencing issues for the past hour
   By: questioner123
   Score: 42
   Date: 2025-05-05 01:23:15.000 +00:00:00
   Comments: 28
   ID: 39619872
   ---
   Title: The Future of Rust Web Development
   URL: https://blog.rust-lang.org/2025/05/05/web-framework-developments.html
   By: rustacean
   Score: 37
   Date: 2025-05-05 01:15:33.000 +00:00:00
   Comments: 19
   ID: 39619844
   ```
   
3. Find Ask HN discussions:
   ```
   Input: hn_ask_stories(count=2)
   Output:
   Title: Ask HN: What productivity tools do you use in 2025?
   Text: Looking for recommendations on the latest tools that have improved your workflow
   By: productive_coder
   Score: 183
   Date: 2025-05-04 18:27:41.000 +00:00:00
   Comments: 207
   ID: 39617842
   ---
   Title: Ask HN: How are you using the new GPT-4o in your workflow?
   Text: Curious about real-world applications and how it's changing your daily tasks
   By: ai_enthusiast
   Score: 156
   Date: 2025-05-04 16:32:18.000 +00:00:00
   Comments: 142
   ID: 39617482
   ```

4. View Show HN projects:
   ```
   Input: hn_show_stories(count=2)
   Output:
   Title: Show HN: Structify – Convert unstructured text to structured data with AI
   URL: https://github.com/structify/structify
   Text: I built this tool to help parse messy text into clean JSON/CSV. It uses a fine-tuned LLM specifically for structure extraction.
   By: dev_builder
   Score: 164
   Date: 2025-05-04 20:15:37.000 +00:00:00
   Comments: 48
   ID: 39618123
   ---
   Title: Show HN: LocalLLM – Run powerful language models on consumer hardware
   URL: https://localllm.ai
   Text: We've optimized large language models to run efficiently on standard consumer GPUs
   By: llm_optimizer
   Score: 147
   Date: 2025-05-04 19:42:11.000 +00:00:00
   Comments: 62
   ID: 39618042
   ```

5. Lookup by specific ID:
   ```
   Input: hn_story_by_id(id=39617316)
   Output:
   Title: Show HN: GPT-4o 10x faster for me using Alt+Enter vs Enter 
   URL: https://twitter.com/tinkergoblin/status/1790778491434525211
   By: tinkergoblin
   Score: 256
   Date: 2025-05-04 15:43:20.000 +00:00:00
   Comments: 89
   ID: 39617316
   ```".to_string()),
        }
    }
}
