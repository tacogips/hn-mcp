# HN Search MCP Specification

## Overview

model context protocol to hacker new https://news.ycombinator.com/

/v0/topstories
/v0/newstories
/v0/beststories

using rust sdk https://github.com/JoeyMckenzie/newswrap/

## docs

### hacker news api spec

https://github.com/HackerNews/API

## dependency

### hacker news client

github: https://github.com/JoeyMckenzie/newswrap/
crate: https://crates.io/crates/newswrap

the source code is cloned at hn-mcp/newswrap so you can refer the all of the sources if needed

storiesのidの取得は
HackerNewsRealtimeClient の

```
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

を使用する。

そのidからstoryのcontentsを取得するには、

HackerNewsItemClient の

```
    /// Retrieves a story from Hacker News, returning errors if the item was not a valid story type.
    pub async fn get_story(&self, id: HackerNewsID) -> HackerNewsResult<HackerNewsStory> {
        self.get_typed_item(id).await
    }
```

を使用して行う。

複数のnews idをtokioでconcurrentに取得する。複数のnews idをchunkに分けて(default 5) それぞれを同時に取得する。
