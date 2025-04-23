use anyhow::{anyhow, Result};
use lru::LruCache;
use newswrap::client::HackerNewsClient;
use newswrap::items::stories::HackerNewsStory;
use newswrap::HackerNewsID;
use std::num::NonZeroUsize;
use std::sync::Arc;
use time::OffsetDateTime;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

#[cfg(test)]
mod tests;

// Since HackerNewsStory doesn't implement Clone, we'll store the essential fields we need
#[derive(Debug, Clone)]
struct CachedStory {
    id: HackerNewsID,
    title: String,
    url: String, 
    text: String,
    by: String,
    score: u32,
    created_at_string: String,
    number_of_comments: u32,
    // Keep comments as empty vector since we don't use them directly
    comments: Vec<HackerNewsID>,
}

impl From<HackerNewsStory> for CachedStory {
    fn from(story: HackerNewsStory) -> Self {
        CachedStory {
            id: story.id,
            title: story.title.clone(),
            url: story.url.clone(),
            text: story.text.clone(),
            by: story.by.clone(),
            score: story.score,
            created_at_string: story.created_at.to_string(),
            number_of_comments: story.number_of_comments,
            comments: story.comments.clone(),
        }
    }
}

impl CachedStory {
    // Create a new HackerNewsStory from cached data
    fn to_story(&self) -> Result<HackerNewsStory, anyhow::Error> {
        // Parse the date string into OffsetDateTime (simplistic approach)
        let created_at = match OffsetDateTime::parse(&self.created_at_string, &time::format_description::well_known::Rfc3339) {
            Ok(dt) => dt,
            Err(_) => OffsetDateTime::now_utc(), // Fallback to current time if parsing fails
        };
        
        // Create a new story by copying the cached fields
        Ok(HackerNewsStory {
            id: self.id,
            title: self.title.clone(),
            url: self.url.clone(),
            text: self.text.clone(),
            by: self.by.clone(),
            score: self.score,
            created_at,
            number_of_comments: self.number_of_comments,
            comments: self.comments.clone(),
        })
    }
}

pub struct HnClient {
    client: Arc<HackerNewsClient>,
    story_cache: Arc<Mutex<LruCache<HackerNewsID, CachedStory>>>,
}

impl Clone for HnClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            story_cache: self.story_cache.clone(),
        }
    }
}

impl Default for HnClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HnClient {
    pub fn new() -> Self {
        // Create a cache with capacity of 100 stories
        let cache_size = NonZeroUsize::new(100).expect("Cache size must be non-zero");
        Self {
            client: Arc::new(HackerNewsClient::new()),
            story_cache: Arc::new(Mutex::new(LruCache::new(cache_size))),
        }
    }
    
    /// Set a custom cache size (for testing or special use cases)
    pub fn with_cache_size(cache_size: usize) -> Self {
        let cache_size = NonZeroUsize::new(cache_size.max(1)).expect("Cache size must be non-zero");
        Self {
            client: Arc::new(HackerNewsClient::new()),
            story_cache: Arc::new(Mutex::new(LruCache::new(cache_size))),
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

    // Get details for a single story by ID with caching
    pub async fn get_story_details(&self, id: HackerNewsID) -> Result<HackerNewsStory> {
        // Check if the story is in cache first
        {
            let mut cache = self.story_cache.lock().await;
            if let Some(cached_story) = cache.get(&id) {
                debug!("Cache hit for story ID: {}", id);
                return cached_story.to_story();
            }
        }
        
        // If not in cache, fetch from API
        debug!("Cache miss for story ID: {}, fetching from API", id);
        let story = self.client.items.get_story(id).await
            .map_err(|e| anyhow!("Failed to fetch story with ID {}: {}", id, e))?;
        
        // Store in cache
        {
            let mut cache = self.story_cache.lock().await;
            let cached_story = CachedStory::from(story);
            
            // We need to re-fetch the story because we've consumed it
            match self.client.items.get_story(id).await {
                Ok(story) => {
                    // Put in cache anyway
                    cache.put(id, cached_story);
                    Ok(story)
                }
                Err(e) => {
                    // Attempt to convert from cache as a fallback
                    if let Ok(story) = cached_story.to_story() {
                        Ok(story)
                    } else {
                        Err(anyhow!("Failed to fetch story with ID {}: {}", id, e))
                    }
                }
            }
        }
    }

    // Get details for multiple stories in parallel, processing in chunks with caching
    pub async fn get_stories_details(&self, ids: Vec<HackerNewsID>, chunk_size: Option<usize>) -> Result<Vec<HackerNewsStory>> {
        let chunk_size = chunk_size.unwrap_or(5);
        debug!("Fetching {} stories with chunk size {}", ids.len(), chunk_size);
        
        let mut all_stories = Vec::with_capacity(ids.len());
        let mut ids_to_fetch = Vec::new();
        
        // First check which stories are already in cache
        {
            let mut cache = self.story_cache.lock().await;
            for id in &ids {
                if let Some(cached_story) = cache.get(id) {
                    debug!("Cache hit for story ID: {}", *id);
                    if let Ok(story) = cached_story.to_story() {
                        all_stories.push(story);
                        continue;
                    }
                    // If there's an error converting the cached story, we'll fetch it again
                    debug!("Error converting cached story ID: {}, will re-fetch", *id);
                }
                ids_to_fetch.push(*id);
            }
        }
        
        if ids_to_fetch.is_empty() {
            debug!("All stories were in cache. No API requests needed.");
            return Ok(all_stories);
        }
        
        debug!("{} stories found in cache, fetching {} from API", 
            ids.len() - ids_to_fetch.len(), ids_to_fetch.len());
        
        // Create chunks of IDs to process in parallel batches
        let chunks: Vec<Vec<HackerNewsID>> = ids_to_fetch
            .chunks(chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect();
        
        // Process each chunk concurrently
        for chunk in chunks {
            debug!("Processing chunk of {} story IDs", chunk.len());
            let mut tasks = Vec::new();
            
            // Create a task for each story ID in the current chunk
            for id in chunk {
                let client = self.clone();
                
                // Spawn a task for each story (now using our get_story_details method which includes caching)
                let task = tokio::spawn(async move {
                    info!("Fetching story ID: {}", id);
                    client.get_story_details(id).await
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