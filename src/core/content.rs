use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    pub id: Uuid,
    pub title: String,
    pub markdown: String,
    pub html: String,
    pub metadata: ContentMetadata,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMetadata {
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub cover_image: Option<String>,
    pub reading_time: Option<u32>, // 分钟
    pub word_count: Option<u32>,
    pub custom_fields: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedContent {
    pub content: Content,
    pub wechat_html: Option<String>,
    pub zhihu_html: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Platform {
    WeChat,
    Zhihu,
    All,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::WeChat => write!(f, "wechat"),
            Platform::Zhihu => write!(f, "zhihu"),
            Platform::All => write!(f, "all"),
        }
    }
}

impl std::str::FromStr for Platform {
    type Err = crate::error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "wechat" => Ok(Platform::WeChat),
            "zhihu" => Ok(Platform::Zhihu),
            "all" => Ok(Platform::All),
            _ => Err(crate::error::Error::InvalidPlatform(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishResult {
    pub platform: Platform,
    pub url: Option<String>,
    pub draft_id: Option<String>,
    pub status: PublishStatus,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PublishStatus {
    Success,
    Draft,
    Failed,
    Pending,
}

impl Content {
    pub fn new(title: String, markdown: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            title,
            markdown,
            html: String::new(),
            metadata: ContentMetadata::default(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn calculate_reading_time(&mut self) {
        // 按平均阅读速度200字/分钟计算
        let word_count = self.markdown.chars().count() as u32;
        self.metadata.word_count = Some(word_count);
        self.metadata.reading_time = Some((word_count / 200).max(1));
    }

    pub fn from_markdown_with_front_matter(markdown: String) -> Result<Self, crate::error::Error> {
        use crate::core::processor::MarkdownProcessor;
        let processor = MarkdownProcessor::new();
        processor.process(&markdown)
    }

    pub fn update_content(&mut self, markdown: String) {
        self.markdown = markdown;
        self.updated_at = chrono::Utc::now();
        self.calculate_reading_time();
    }
}

impl Default for ContentMetadata {
    fn default() -> Self {
        Self {
            author: None,
            tags: Vec::new(),
            description: None,
            cover_image: None,
            reading_time: None,
            word_count: None,
            custom_fields: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_content_creation() {
        let content = Content::new("Test Title".to_string(), "# Test Content".to_string());
        
        assert_eq!(content.title, "Test Title");
        assert_eq!(content.markdown, "# Test Content");
        assert!(content.html.is_empty());
        assert!(content.metadata.author.is_none());
        assert!(content.metadata.tags.is_empty());
    }

    #[test]
    fn test_metadata_default() {
        let metadata = ContentMetadata::default();
        
        assert!(metadata.author.is_none());
        assert!(metadata.tags.is_empty());
        assert!(metadata.description.is_none());
        assert!(metadata.cover_image.is_none());
        assert!(metadata.reading_time.is_none());
        assert!(metadata.word_count.is_none());
        assert!(metadata.custom_fields.is_empty());
    }

    #[test]
    fn test_platform_enum() {
        assert_eq!(Platform::WeChat.to_string(), "wechat");
        assert_eq!(Platform::Zhihu.to_string(), "zhihu");
        assert_eq!(Platform::All.to_string(), "all");
    }

    #[test]
    fn test_platform_from_str() {
        assert_eq!(Platform::from_str("wechat").unwrap(), Platform::WeChat);
        assert_eq!(Platform::from_str("zhihu").unwrap(), Platform::Zhihu);
        assert_eq!(Platform::from_str("all").unwrap(), Platform::All);
        assert!(Platform::from_str("invalid").is_err());
    }

    #[test]
    fn test_front_matter_parsing() {
        let content_with_front_matter = r#"---
title: "Test Article"
author: "Test Author"
tags: "rust,programming"
description: "A test article"
---

# Main Content

This is the main content of the article."#;

        let content = Content::from_markdown_with_front_matter(content_with_front_matter.to_string()).unwrap();
        
        assert_eq!(content.title, "Test Article");
        assert_eq!(content.metadata.author, Some("Test Author".to_string()));
        assert_eq!(content.metadata.tags, vec!["rust", "programming"]);
        assert_eq!(content.metadata.description, Some("A test article".to_string()));
        assert!(content.markdown.contains("# Main Content"));
    }

    #[test]
    fn test_markdown_without_front_matter() {
        let markdown = "# Simple Title\n\nSimple content.";
        let content = Content::from_markdown_with_front_matter(markdown.to_string()).unwrap();
        
        assert_eq!(content.title, "Simple Title");
        assert_eq!(content.markdown, markdown);
        assert!(content.metadata.author.is_none());
    }
}