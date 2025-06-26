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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Platform {
    WeChat,
    Zhihu,
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