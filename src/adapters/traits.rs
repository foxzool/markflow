use crate::{
    core::content::{Content, Platform},
    Result,
};
use async_trait::async_trait;

#[async_trait]
pub trait PlatformAdapter: Send + Sync {
    fn platform(&self) -> Platform;
    fn adapt_html(&self, html: &str) -> Result<String>;
    fn validate_content(&self, content: &Content) -> Result<()>;
    async fn preprocess_images(&self, html: &str) -> Result<String>;
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub severity: ValidationSeverity,
}

#[derive(Debug, Clone)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

pub trait StyleProvider {
    fn get_styles(&self) -> &str;
    fn apply_inline_styles(&self, html: &str) -> Result<String>;
}
