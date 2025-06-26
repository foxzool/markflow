use crate::Result;
use crate::core::content::Content;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait ProcessingStage: Send + Sync {
    async fn process(&self, content: &mut Content) -> Result<()>;
    fn name(&self) -> &'static str;
}

pub struct ProcessingPipeline {
    stages: Vec<Arc<dyn ProcessingStage>>,
}

impl ProcessingPipeline {
    pub fn new() -> Self {
        Self {
            stages: Vec::new(),
        }
    }

    pub fn add_stage<T: ProcessingStage + 'static>(mut self, stage: T) -> Self {
        self.stages.push(Arc::new(stage));
        self
    }

    pub async fn process(&self, mut content: Content) -> Result<Content> {
        tracing::info!("开始处理流水线，包含 {} 个阶段", self.stages.len());

        for (i, stage) in self.stages.iter().enumerate() {
            tracing::debug!("执行阶段 {}: {}", i + 1, stage.name());
            
            match stage.process(&mut content).await {
                Ok(_) => {
                    tracing::debug!("阶段 {} 完成", stage.name());
                }
                Err(e) => {
                    tracing::error!("阶段 {} 失败: {}", stage.name(), e);
                    return Err(e);
                }
            }
        }

        tracing::info!("处理流水线完成");
        Ok(content)
    }
}

// 图片处理阶段
pub struct ImageProcessingStage;

#[async_trait]
impl ProcessingStage for ImageProcessingStage {
    async fn process(&self, content: &mut Content) -> Result<()> {
        // 提取并处理图片
        let image_regex = regex::Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)").unwrap();
        
        for capture in image_regex.captures_iter(&content.markdown.clone()) {
            let alt = &capture[1];
            let src = &capture[2];
            
            tracing::debug!("处理图片: {} ({})", src, alt);
            
            // 这里可以添加图片处理逻辑：
            // - 下载远程图片
            // - 压缩图片
            // - 上传到CDN
            // - 生成不同尺寸版本
        }
        
        Ok(())
    }

    fn name(&self) -> &'static str {
        "图片处理"
    }
}

// 链接验证阶段
pub struct LinkValidationStage;

#[async_trait]
impl ProcessingStage for LinkValidationStage {
    async fn process(&self, content: &mut Content) -> Result<()> {
        let link_regex = regex::Regex::new(r"\[([^\]]*)\]\(([^)]+)\)").unwrap();
        
        for capture in link_regex.captures_iter(&content.markdown.clone()) {
            let text = &capture[1];
            let url = &capture[2];
            
            if url.starts_with("http") {
                tracing::debug!("验证外部链接: {} ({})", url, text);
                // 这里可以添加链接验证逻辑
                // - 检查链接是否可访问
                // - 获取链接标题
                // - 检查链接是否安全
            }
        }
        
        Ok(())
    }

    fn name(&self) -> &'static str {
        "链接验证"
    }
}

// 内容增强阶段
pub struct ContentEnhancementStage;

#[async_trait]
impl ProcessingStage for ContentEnhancementStage {
    async fn process(&self, content: &mut Content) -> Result<()> {
        // 自动生成摘要
        if content.metadata.description.is_none() {
            let summary = self.generate_summary(&content.markdown);
            content.metadata.description = Some(summary);
        }
        
        // 自动提取标签
        if content.metadata.tags.is_empty() {
            content.metadata.tags = self.extract_tags(&content.markdown);
        }
        
        Ok(())
    }

    fn name(&self) -> &'static str {
        "内容增强"
    }
}

impl ContentEnhancementStage {
    fn generate_summary(&self, markdown: &str) -> String {
        // 简单的摘要生成：取第一段非标题内容
        let lines: Vec<&str> = markdown.lines()
            .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
            .take(3)
            .collect();
        
        let summary = lines.join(" ");
        if summary.len() > 200 {
            format!("{}...", &summary[0..197])
        } else {
            summary
        }
    }
    
    fn extract_tags(&self, markdown: &str) -> Vec<String> {
        // 简单的标签提取：基于关键词
        let keywords = [
            "Rust", "JavaScript", "Python", "TypeScript", "React", "Vue", "Node.js",
            "前端", "后端", "全栈", "微服务", "数据库", "算法", "设计模式",
            "性能优化", "安全", "测试", "部署", "Docker", "Kubernetes",
        ];
        
        let markdown_lower = markdown.to_lowercase();
        keywords.iter()
            .filter(|&keyword| markdown_lower.contains(&keyword.to_lowercase()))
            .map(|&keyword| keyword.to_string())
            .collect()
    }
}

impl Default for ProcessingPipeline {
    fn default() -> Self {
        Self::new()
            .add_stage(ImageProcessingStage)
            .add_stage(LinkValidationStage)
            .add_stage(ContentEnhancementStage)
    }
}