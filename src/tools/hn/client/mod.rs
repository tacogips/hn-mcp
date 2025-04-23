use anyhow::{anyhow, Result};
use newswrap::clients::{HackerNewsItemClient, HackerNewsRealtimeClient};
use newswrap::models::{HackerNewsID, HackerNewsStory};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, error, info};

#[cfg(test)]
mod tests;

pub struct HnClient {
    item_client: HackerNewsItemClient,
    realtime_client: HackerNewsRealtimeClient,
}

impl HnClient {
    pub fn new() -> Self {
        Self {
            item_client: HackerNewsItemClient::new(),
            realtime_client: HackerNewsRealtimeClient::new(),
        }
    }

    // Get top stories from Hacker News
    pub async fn get_top_stories(&self, limit: Option<usize>) -> Result<Vec<HackerNewsID>> {
        let stories = self.realtime_client.get_top_stories().await
            .map_err(|e| anyhow!("Failed to fetch top stories: {}", e))?;

        let limit = limit.unwrap_or(stories.len());
        Ok(stories.into_iter().take(limit).collect())
    }

    // Get latest stories from Hacker News
    pub async fn get_latest_stories(&self, limit: Option<usize>) -> Result<Vec<HackerNewsID>> {
        let stories = self.realtime_client.get_latest_stories().await
            .map_err(|e| anyhow!("Failed to fetch latest stories: {}", e))?;

        let limit = limit.unwrap_or(stories.len());
        Ok(stories.into_iter().take(limit).collect())
    }

    // Get best stories from Hacker News
    pub async fn get_best_stories(&self, limit: Option<usize>) -> Result<Vec<HackerNewsID>> {
        let stories = self.realtime_client.get_best_stories().await
            .map_err(|e| anyhow!("Failed to fetch best stories: {}", e))?;

        let limit = limit.unwrap_or(stories.len());
        Ok(stories.into_iter().take(limit).collect())
    }

    // Get ask HN stories
    pub async fn get_ask_stories(&self, limit: Option<usize>) -> Result<Vec<HackerNewsID>> {
        let stories = self.realtime_client.get_ask_hacker_news_stories().await
            .map_err(|e| anyhow!("Failed to fetch Ask HN stories: {}", e))?;

        let limit = limit.unwrap_or(stories.len());
        Ok(stories.into_iter().take(limit).collect())
    }

    // Get show HN stories
    pub async fn get_show_stories(&self, limit: Option<usize>) -> Result<Vec<HackerNewsID>> {
        let stories = self.realtime_client.get_show_hacker_news_stories().await
            .map_err(|e| anyhow!("Failed to fetch Show HN stories: {}", e))?;

        let limit = limit.unwrap_or(stories.len());
        Ok(stories.into_iter().take(limit).collect())
    }

    // Get details for a single story by ID
    pub async fn get_story_details(&self, id: HackerNewsID) -> Result<HackerNewsStory> {
        self.item_client.get_story(id).await
            .map_err(|e| anyhow!("Failed to fetch story with ID {}: {}", id, e))
    }

    // Get details for multiple stories in parallel, processing in chunks
    pub async fn get_stories_details(&self, ids: Vec<HackerNewsID>, chunk_size: Option<usize>) -> Result<Vec<HackerNewsStory>> {
        let chunk_size = chunk_size.unwrap_or(5);
        let semaphore = Arc::new(Semaphore::new(chunk_size));
        let mut tasks = Vec::new();

        // Split IDs into chunks to process in parallel
        for id in ids {
            let permit = semaphore.clone().acquire_owned().await?;
            let item_client = self.item_client.clone();
            
            // Spawn a task for each story
            let task = tokio::spawn(async move {
                let result = item_client.get_story(id).await;
                drop(permit); // Release the semaphore permit
                result
            });
            
            tasks.push(task);
        }

        let mut stories = Vec::new();
        for task in tasks {
            match task.await {
                Ok(result) => match result {
                    Ok(story) => stories.push(story),
                    Err(e) => error!("Error fetching story: {}", e),
                },
                Err(e) => error!("Task error: {}", e),
            }
        }

        Ok(stories)
    }

    // Format a story into a readable string
    pub fn format_story(story: &HackerNewsStory) -> String {
        let url_section = match &story.url {
            Some(url) => format!("URL: {}\n", url),
            None => String::new(),
        };

        let text_section = match &story.text {
            Some(text) => format!("Text: {}\n", text),
            None => String::new(),
        };

        let score = story.score.unwrap_or_default();
        let time = story.time;
        let date_time = chrono::NaiveDateTime::from_timestamp_opt(time as i64, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "Unknown time".to_string());

        let descendants = story.descendants.unwrap_or_default();

        format!(
            "Title: {}\n{}{}By: {}\nScore: {}\nDate: {}\nComments: {}\nID: {}\n",
            story.title,
            url_section,
            text_section,
            story.by,
            score,
            date_time,
            descendants,
            story.id
        )
    }
}