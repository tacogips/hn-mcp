# HN Search MCP Specification

## Overview

Model Context Protocol (MCP) interface to Hacker News (https://news.ycombinator.com/)

API endpoints used:
- /v0/topstories
- /v0/newstories
- /v0/beststories
- /v0/askstories
- /v0/showstories

This MCP uses the Rust SDK: https://github.com/JoeyMckenzie/newswrap/

## Documentation

### Hacker News API Specification

https://github.com/HackerNews/API

## Dependencies

### Hacker News Client

- GitHub: https://github.com/JoeyMckenzie/newswrap/
- Crate: https://crates.io/crates/newswrap

The source code is cloned at hn-mcp/newswrap so you can refer to all of the sources if needed.

## Implementation Details

### Story ID Retrieval

Story IDs are retrieved using `HackerNewsRealtimeClient`:

```rust
pub async fn get_top_stories(&self) -> HackerNewsResult<HackerNewsItemList> {
    self.get_realtime_story_data(TOP_STORIES_ENDPOINT).await
}

pub async fn get_latest_stories(&self) -> HackerNewsResult<HackerNewsItemList> {
    self.get_realtime_story_data(NEW_STORIES_ENDPOINT).await
}

pub async fn get_best_stories(&self) -> HackerNewsResult<HackerNewsItemList> {
    self.get_realtime_story_data(BEST_STORIES_ENDPOINT).await
}

pub async fn get_ask_hacker_news_stories(&self) -> HackerNewsResult<HackerNewsItemList> {
    self.get_realtime_story_data(ASK_STORIES_ENDPOINT).await
}

pub async fn get_show_hacker_news_stories(&self) -> HackerNewsResult<HackerNewsItemList> {
    self.get_realtime_story_data(SHOW_STORIES_ENDPOINT).await
}
```

### Story Content Retrieval

Story contents are retrieved from IDs using `HackerNewsItemClient`:

```rust
/// Retrieves a story from Hacker News, returning errors if the item was not a valid story type.
pub async fn get_story(&self, id: HackerNewsID) -> HackerNewsResult<HackerNewsStory> {
    self.get_typed_item(id).await
}
```

### Concurrency Model

Multiple news IDs are retrieved concurrently using Tokio. The process:
1. News IDs are divided into chunks (default 5, max 10, min 1)
2. Each chunk is processed concurrently
3. The `chunk_size` parameter is clamped using Rust's `clamp()` method:
   ```rust
   let chunk_size = chunk_size.unwrap_or(5).clamp(1, 10);
   ```

### Caching

A local LRU (Least Recently Used) cache is implemented to reduce API requests:
1. The lru-rs crate is used for efficient caching
2. Story details are cached with a default capacity of 100 items
3. When fetching story details, the cache is checked first before making API requests
4. Cache hits/misses are logged for performance monitoring
5. If a story is not in the cache, it is fetched from the API and then stored in the cache
6. A custom wrapper type `CachedStory` is used to store cloneable story data since `HackerNewsStory` does not implement `Clone`
   ```rust
   #[derive(Debug, Clone)]
   struct CachedStory {
       id: HackerNewsID,
       title: String,
       url: String, 
       text: String,
       by: String,
       score: u32,
       // other fields...
   }
   ```

### HnClient Implementation

The `HnClient` implements the `Default` trait for better ergonomics:

```rust
impl Default for HnClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HnClient {
    pub fn new() -> Self {
        Self {
            client: Arc::new(HackerNewsClient::new()),
        }
    }
}
```

## Tool Methods

The MCP exposes the following tool methods:
- `hn_top_stories`: Retrieves the top stories from Hacker News
- `hn_latest_stories`: Retrieves the latest stories from Hacker News
- `hn_best_stories`: Retrieves the best stories from Hacker News
- `hn_ask_stories`: Retrieves Ask HN stories from Hacker News
- `hn_show_stories`: Retrieves Show HN stories from Hacker News
- `hn_story_by_id`: Retrieves story details by ID from Hacker News
