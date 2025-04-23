#[cfg(test)]
mod tests {
    use super::*;

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
}