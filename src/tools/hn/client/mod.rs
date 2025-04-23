use anyhow::{anyhow, Result};
use newswrap::client::HackerNewsClient;
use newswrap::items::stories::HackerNewsStory;
use newswrap::HackerNewsID;
use tracing::{debug, error, info};
use std::sync::Arc;

#[cfg(test)]
mod tests;

pub struct HnClient {
    client: Arc<HackerNewsClient>,
}

impl Clone for HnClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
        }
    }
}

impl HnClient {
    pub fn new() -> Self {
        Self {
            client: Arc::new(HackerNewsClient::new()),
        }
    }

    // Get top stories from Hacker News
    pub async fn get_top_stories(&self, limit: Option<usize>) -> Result<Vec<HackerNewsID>> {
        let stories = self.client.realtime.get_top_stories().await
            .map_err(|e| anyhow!("Failed to fetch top stories: {}", e))?;

        let limit = limit.unwrap_or(stories.len());
        Ok(stories.into_iter().take(limit).collect())
    }

    // Get latest stories from Hacker News
    pub async fn get_latest_stories(&self, limit: Option<usize>) -> Result<Vec<HackerNewsID>> {
        let stories = self.client.realtime.get_latest_stories().await
            .map_err(|e| anyhow!("Failed to fetch latest stories: {}", e))?;

        let limit = limit.unwrap_or(stories.len());
        Ok(stories.into_iter().take(limit).collect())
    }

    // Get best stories from Hacker News
    pub async fn get_best_stories(&self, limit: Option<usize>) -> Result<Vec<HackerNewsID>> {
        let stories = self.client.realtime.get_best_stories().await
            .map_err(|e| anyhow!("Failed to fetch best stories: {}", e))?;

        let limit = limit.unwrap_or(stories.len());
        Ok(stories.into_iter().take(limit).collect())
    }

    // Get ask HN stories
    pub async fn get_ask_stories(&self, limit: Option<usize>) -> Result<Vec<HackerNewsID>> {
        let stories = self.client.realtime.get_ask_hacker_news_stories().await
            .map_err(|e| anyhow!("Failed to fetch Ask HN stories: {}", e))?;

        let limit = limit.unwrap_or(stories.len());
        Ok(stories.into_iter().take(limit).collect())
    }

    // Get show HN stories
    pub async fn get_show_stories(&self, limit: Option<usize>) -> Result<Vec<HackerNewsID>> {
        let stories = self.client.realtime.get_show_hacker_news_stories().await
            .map_err(|e| anyhow!("Failed to fetch Show HN stories: {}", e))?;

        let limit = limit.unwrap_or(stories.len());
        Ok(stories.into_iter().take(limit).collect())
    }

    // Get details for a single story by ID
    pub async fn get_story_details(&self, id: HackerNewsID) -> Result<HackerNewsStory> {
        self.client.items.get_story(id).await
            .map_err(|e| anyhow!("Failed to fetch story with ID {}: {}", id, e))
    }

    // Get details for multiple stories in parallel, processing in chunks
    pub async fn get_stories_details(&self, ids: Vec<HackerNewsID>, chunk_size: Option<usize>) -> Result<Vec<HackerNewsStory>> {
        let chunk_size = chunk_size.unwrap_or(5);
        debug!("Fetching {} stories with chunk size {}", ids.len(), chunk_size);
        
        // Create chunks of IDs to process in parallel batches
        let chunks: Vec<Vec<HackerNewsID>> = ids
            .chunks(chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect();
        
        let mut all_stories = Vec::new();
        
        // Process each chunk concurrently
        for chunk in chunks {
            debug!("Processing chunk of {} story IDs", chunk.len());
            let mut tasks = Vec::new();
            
            // Create a task for each story ID in the current chunk
            for id in chunk {
                let client_clone = self.client.clone();
                
                // Spawn a task for each story
                let task = tokio::spawn(async move {
                    info!("Fetching story ID: {}", id);
                    let result = client_clone.items.get_story(id).await;
                    if let Err(ref e) = result {
                        error!("Failed to fetch story ID {}: {}", id, e);
                    }
                    result
                });
                
                tasks.push(task);
            }
            
            // Await all tasks in the current chunk
            let chunk_results = futures::future::join_all(tasks).await;
            
            // Process results from the current chunk
            for result in chunk_results {
                match result {
                    Ok(story_result) => match story_result {
                        Ok(story) => {
                            debug!("Successfully fetched story ID: {}", story.id);
                            all_stories.push(story);
                        }
                        Err(e) => error!("Error fetching story: {}", e),
                    },
                    Err(e) => error!("Task error: {}", e),
                }
            }
        }
        
        debug!("Fetched {} stories successfully", all_stories.len());
        Ok(all_stories)
    }

    // Format a story into a readable string
    pub fn format_story(story: &HackerNewsStory) -> String {
        // URLが空でない場合に表示
        let url_section = if !story.url.is_empty() {
            format!("URL: {}\n", story.url)
        } else {
            String::new()
        };

        // テキストが空でない場合に表示
        let text_section = if !story.text.is_empty() {
            format!("Text: {}\n", story.text)
        } else {
            String::new()
        };

        // created_atを文字列にフォーマット
        let created_at = &story.created_at;
        let date_time = format!("{}", created_at);

        format!(
            "Title: {}\n{}{}By: {}\nScore: {}\nDate: {}\nComments: {}\nID: {}\n",
            story.title,
            url_section,
            text_section,
            story.by,
            story.score,
            date_time,
            story.number_of_comments,
            story.id
        )
    }
}