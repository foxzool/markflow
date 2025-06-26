use crate::{Result, core::content::{Content, PublishResult, Platform}};
use async_trait::async_trait;

#[async_trait]
pub trait Publisher: Send + Sync {
    fn platform(&self) -> Platform;
    
    async fn publish(&mut self, content: &Content) -> Result<PublishResult>;
    
    async fn create_draft(&mut self, content: &Content) -> Result<PublishResult>;
    
    async fn update_content(&mut self, content_id: &str, content: &Content) -> Result<PublishResult>;
    
    async fn delete_content(&mut self, content_id: &str) -> Result<()>;
    
    async fn get_publish_status(&self, content_id: &str) -> Result<PublishResult>;
}

#[async_trait]
pub trait AuthProvider: Send + Sync {
    async fn authenticate(&mut self) -> Result<()>;
    
    async fn refresh_auth(&mut self) -> Result<()>;
    
    fn is_authenticated(&self) -> bool;
}

pub trait ConfigProvider {
    type Config;
    
    fn get_config(&self) -> &Self::Config;
    
    fn update_config(&mut self, config: Self::Config);
}