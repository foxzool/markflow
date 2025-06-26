pub mod core;
pub mod adapters;
pub mod publishers;
pub mod cli;
pub mod web;

pub use core::*;
pub use adapters::{WeChatStyleAdapter, ZhihuStyleAdapter, PlatformAdapter};
pub use publishers::Publisher;

pub type Result<T> = std::result::Result<T, crate::error::Error>;

pub mod error {
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum Error {
        #[error("IO error: {0}")]
        IO(#[from] std::io::Error),
        
        #[error("Markdown processing error: {0}")]
        Markdown(String),
        
        #[error("HTML processing error: {0}")]
        Html(String),
        
        #[error("HTTP error: {0}")]
        Http(#[from] reqwest::Error),
        
        #[error("Serialization error: {0}")]
        Serde(#[from] serde_json::Error),
        
        #[error("Browser automation error: {0}")]
        Browser(String),
        
        #[error("Publishing error: {0}")]
        Publishing(String),
        
        #[error("Configuration error: {0}")]
        Config(String),
        
        #[error("Template error: {0}")]
        Template(#[from] tera::Error),
        
        #[error("Invalid platform: {0}")]
        InvalidPlatform(String),
        
        #[error("Other error: {0}")]
        Other(String),
    }
}