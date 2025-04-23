#[cfg(test)]
mod tests {
    use crate::tools::hn::client::HnClient;
    use std::time::Instant;

    #[tokio::test]
    async fn test_get_top_stories() {
        let client = HnClient::new();
        let stories = client.get_top_stories(Some(5)).await.unwrap();
        
        assert!(!stories.is_empty());
        assert!(stories.len() <= 5);
        
        println!("Top Story IDs: {:?}", stories);
    }

    #[tokio::test]
    async fn test_get_story_details() {
        let client = HnClient::new();
        
        // First get some story IDs
        let stories = client.get_top_stories(Some(1)).await.unwrap();
        assert!(!stories.is_empty());
        
        // Get details for the first story
        let story_id = stories[0];
        let story = client.get_story_details(story_id).await.unwrap();
        
        assert_eq!(story.id, story_id);
        assert!(!story.title.is_empty());
        
        // Print formatted story
        let formatted = HnClient::format_story(&story);
        println!("Formatted story:\n{}", formatted);
    }

    #[tokio::test]
    async fn test_get_stories_details() {
        let client = HnClient::new();
        
        // Get some story IDs
        let story_ids = client.get_top_stories(Some(3)).await.unwrap();
        assert!(story_ids.len() <= 3);
        
        // Get details for all stories concurrently
        let stories = client.get_stories_details(story_ids.clone(), Some(2)).await.unwrap();
        
        // Should have the same number of stories as IDs (unless some failed)
        assert!(stories.len() <= story_ids.len());
        
        // Print IDs of stories we got
        let received_ids: Vec<u32> = stories.iter().map(|s| s.id).collect();
        println!("Received story IDs: {:?}", received_ids);
    }
    
    #[tokio::test]
    async fn test_concurrency_performance() {
        let client = HnClient::new();
        
        // Get a larger batch of story IDs for testing
        let story_ids = client.get_top_stories(Some(10)).await.unwrap();
        assert!(story_ids.len() <= 10);
        
        // First test with small chunk size (more concurrent fetches)
        let start = Instant::now();
        let stories_concurrent = client.get_stories_details(story_ids.clone(), Some(5)).await.unwrap();
        let concurrent_duration = start.elapsed();
        
        // Then test with chunk size of 1 (sequential fetches)
        let start = Instant::now();
        let stories_sequential = client.get_stories_details(story_ids.clone(), Some(1)).await.unwrap();
        let sequential_duration = start.elapsed();
        
        println!("Performance comparison:");
        println!("  Concurrent (chunk=5): {:?} for {} stories", concurrent_duration, stories_concurrent.len());
        println!("  Sequential (chunk=1): {:?} for {} stories", sequential_duration, stories_sequential.len());
        
        // The concurrent approach should generally be faster
        // This is not a strict assertion as network conditions can vary
        println!("  Speed improvement: {:.2}x", sequential_duration.as_secs_f64() / concurrent_duration.as_secs_f64());
    }
    
    #[tokio::test]
    async fn test_different_story_types() {
        let client = HnClient::new();
        
        // Test all different story types with a small count
        let top_stories = client.get_top_stories(Some(2)).await.unwrap();
        let latest_stories = client.get_latest_stories(Some(2)).await.unwrap();
        let best_stories = client.get_best_stories(Some(2)).await.unwrap();
        let ask_stories = client.get_ask_stories(Some(2)).await.unwrap();
        let show_stories = client.get_show_stories(Some(2)).await.unwrap();
        
        println!("Different story types:");
        println!("  Top stories: {:?}", top_stories);
        println!("  Latest stories: {:?}", latest_stories);
        println!("  Best stories: {:?}", best_stories);
        println!("  Ask stories: {:?}", ask_stories);
        println!("  Show stories: {:?}", show_stories);
        
        // Make sure we got results for each type
        assert!(!top_stories.is_empty());
        assert!(!latest_stories.is_empty());
        assert!(!best_stories.is_empty());
        
        // Ask and Show stories might be empty depending on content availability
        println!("  Ask stories count: {}", ask_stories.len());
        println!("  Show stories count: {}", show_stories.len());
    }
}