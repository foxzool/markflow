use crate::{Result, error::Error};
use crate::core::content::{Content, ContentMetadata};
use comrak::{Arena, parse_document, format_html, ComrakOptions};
use comrak::nodes::{AstNode, NodeValue};
use regex::Regex;
use std::collections::HashMap;

pub struct MarkdownProcessor {
    options: ComrakOptions,
    front_matter_regex: Regex,
}

impl MarkdownProcessor {
    pub fn new() -> Self {
        let mut options = ComrakOptions::default();
        
        // 启用GitHub Flavored Markdown扩展
        options.extension.strikethrough = true;
        options.extension.table = true;
        options.extension.autolink = true;
        options.extension.tasklist = true;
        options.extension.footnotes = true;
        options.extension.superscript = true;
        options.extension.tagfilter = false; // 允许HTML标签
        options.extension.description_lists = true;
        
        // 渲染选项
        options.render.hardbreaks = false;
        options.render.github_pre_lang = true;
        options.render.unsafe_ = true; // 允许原始HTML
        
        // 解析选项
        options.parse.smart = true;
        options.parse.default_info_string = Some("text".to_string());

        let front_matter_regex = Regex::new(r"^---\n([\s\S]*?)\n---\n").unwrap();

        Self { 
            options,
            front_matter_regex,
        }
    }

    pub fn process(&self, markdown: &str) -> Result<Content> {
        tracing::info!("开始处理Markdown内容");

        // 解析Front Matter
        let (front_matter, content_markdown) = self.parse_front_matter(markdown)?;
        
        // 从front matter创建metadata
        let metadata = self.create_metadata_from_front_matter(&front_matter)?;
        
        // 提取标题
        let title = self.extract_title(&content_markdown, &front_matter)?;
        
        // 创建内容对象
        let mut content = Content::new(title, content_markdown.clone());
        content.metadata = metadata;
        
        // 处理Markdown
        let html = self.markdown_to_html(&content_markdown)?;
        content.html = html;
        
        // 计算阅读时间
        content.calculate_reading_time();
        
        tracing::info!("Markdown处理完成，标题: {}", content.title);
        Ok(content)
    }

    fn parse_front_matter(&self, markdown: &str) -> Result<(HashMap<String, String>, String)> {
        let mut front_matter = HashMap::new();
        let content_markdown;

        if let Some(captures) = self.front_matter_regex.captures(markdown) {
            let yaml_content = captures.get(1).unwrap().as_str();
            content_markdown = self.front_matter_regex.replace(markdown, "").into_owned();
            
            // 简单的YAML解析（仅支持key: value格式）
            for line in yaml_content.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    let key = key.trim().to_string();
                    let value = value.trim().trim_matches('"').to_string();
                    front_matter.insert(key, value);
                }
            }
        } else {
            content_markdown = markdown.to_string();
        }

        Ok((front_matter, content_markdown))
    }

    fn create_metadata_from_front_matter(&self, front_matter: &HashMap<String, String>) -> Result<ContentMetadata> {
        let mut metadata = ContentMetadata::default();
        
        if let Some(author) = front_matter.get("author") {
            metadata.author = Some(author.clone());
        }
        
        if let Some(description) = front_matter.get("description") {
            metadata.description = Some(description.clone());
        }
        
        if let Some(tags_str) = front_matter.get("tags") {
            metadata.tags = tags_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        
        if let Some(cover) = front_matter.get("cover") {
            metadata.cover_image = Some(cover.clone());
        }

        // 添加自定义字段
        for (key, value) in front_matter {
            if !matches!(key.as_str(), "title" | "author" | "description" | "tags" | "cover") {
                metadata.custom_fields.insert(key.clone(), value.clone());
            }
        }
        
        Ok(metadata)
    }

    fn extract_title(&self, markdown: &str, front_matter: &HashMap<String, String>) -> Result<String> {
        // 首先检查front matter中的title
        if let Some(title) = front_matter.get("title") {
            return Ok(title.clone());
        }
        
        // 从markdown内容中提取第一个一级标题
        let title_regex = Regex::new(r"^#\s+(.+)$").unwrap();
        for line in markdown.lines() {
            if let Some(captures) = title_regex.captures(line) {
                return Ok(captures.get(1).unwrap().as_str().to_string());
            }
        }
        
        // 如果都没有找到，使用默认标题
        Ok("无标题".to_string())
    }

    fn markdown_to_html(&self, markdown: &str) -> Result<String> {
        let arena = Arena::new();
        let root = parse_document(&arena, markdown, &self.options);
        
        // 可以在这里对AST进行后处理
        self.process_ast(&arena, root)?;
        
        let mut html = vec![];
        format_html(root, &self.options, &mut html)
            .map_err(|e| Error::Markdown(format!("HTML生成失败: {}", e)))?;
        
        String::from_utf8(html)
            .map_err(|e| Error::Markdown(format!("HTML编码转换失败: {}", e)))
    }

    fn process_ast<'a>(&self, _arena: &Arena<AstNode>, root: &'a AstNode<'a>) -> Result<()> {
        // 遍历AST节点进行自定义处理
        self.iter_nodes(root, &|node| {
            match &mut node.data.borrow_mut().value {
                NodeValue::Image(ref mut image) => {
                    // 处理图片链接，为相对路径添加前缀等
                    if !image.url.starts_with("http") && !image.url.starts_with("data:") {
                        // 可以在这里转换相对路径为绝对路径
                        tracing::debug!("发现相对路径图片: {}", image.url);
                    }
                }
                NodeValue::Link(ref mut link) => {
                    // 处理链接
                    if !link.url.starts_with("http") {
                        tracing::debug!("发现相对路径链接: {}", link.url);
                    }
                }
                NodeValue::CodeBlock(ref mut code_block) => {
                    // 处理代码块
                    if code_block.info.is_empty() {
                        code_block.info = "text".to_string();
                    }
                }
                _ => {}
            }
            Ok(())
        })?;
        
        Ok(())
    }

    fn iter_nodes<'a, F>(&self, node: &'a AstNode<'a>, callback: &F) -> Result<()>
    where
        F: Fn(&AstNode) -> Result<()>,
    {
        callback(node)?;
        
        for child in node.children() {
            self.iter_nodes(child, callback)?;
        }
        
        Ok(())
    }

    pub fn extract_images(&self, markdown: &str) -> Result<Vec<String>> {
        let image_regex = Regex::new(r"!\[.*?\]\(([^)]+)\)").unwrap();
        let images: Vec<String> = image_regex
            .captures_iter(markdown)
            .map(|cap| cap[1].to_string())
            .collect();
        
        Ok(images)
    }

    pub fn extract_links(&self, markdown: &str) -> Result<Vec<String>> {
        let link_regex = Regex::new(r"\[.*?\]\(([^)]+)\)").unwrap();
        let links: Vec<String> = link_regex
            .captures_iter(markdown)
            .map(|cap| cap[1].to_string())
            .collect();
        
        Ok(links)
    }
}

impl Default for MarkdownProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_creation() {
        let processor = MarkdownProcessor::new();
        assert!(processor.options.extension.table);
        assert!(processor.options.extension.strikethrough);
        assert!(processor.options.extension.footnotes);
    }

    #[test]
    fn test_simple_markdown_processing() {
        let processor = MarkdownProcessor::new();
        let markdown = "# Test Title\n\nThis is a **bold** text.";
        
        let content = processor.process(markdown).unwrap();
        
        assert_eq!(content.title, "Test Title");
        assert!(content.html.contains("<h1>"));
        assert!(content.html.contains("<strong>"));
        assert!(content.html.contains("bold"));
    }

    #[test]
    fn test_front_matter_parsing() {
        let processor = MarkdownProcessor::new();
        let markdown_with_front_matter = r#"---
title: "Custom Title"
author: "Test Author"
tags: "rust,markdown"
description: "Test description"
---

# Heading

Content here."#;

        let content = processor.process(markdown_with_front_matter).unwrap();
        
        assert_eq!(content.title, "Custom Title");
        assert_eq!(content.metadata.author, Some("Test Author".to_string()));
        assert_eq!(content.metadata.tags, vec!["rust", "markdown"]);
        assert_eq!(content.metadata.description, Some("Test description".to_string()));
    }

    #[test]
    fn test_table_rendering() {
        let processor = MarkdownProcessor::new();
        let markdown = r#"
| Name | Age |
|------|-----|
| John | 30  |
| Jane | 25  |
"#;

        let content = processor.process(markdown).unwrap();
        
        assert!(content.html.contains("<table>"));
        assert!(content.html.contains("<th>"));
        assert!(content.html.contains("<td>"));
        assert!(content.html.contains("John"));
        assert!(content.html.contains("Jane"));
    }

    #[test]
    fn test_code_block_rendering() {
        let processor = MarkdownProcessor::new();
        let markdown = r#"
```rust
fn main() {
    println!("Hello, world!");
}
```
"#;

        let content = processor.process(markdown).unwrap();
        
        // 检查代码块渲染
        assert!(content.html.contains("<code") || content.html.contains("<pre>"));
        assert!(content.html.contains("fn main"));
    }

    #[test]
    fn test_title_extraction() {
        let processor = MarkdownProcessor::new();
        
        // Test H1 extraction
        let markdown1 = "# First Heading\n\nContent.";
        let content1 = processor.process(markdown1).unwrap();
        assert_eq!(content1.title, "First Heading");
        
        // Test no heading
        let markdown2 = "Just some content without heading.";
        let content2 = processor.process(markdown2).unwrap();
        assert_eq!(content2.title, "无标题");
    }

    #[test]
    fn test_word_count_calculation() {
        let processor = MarkdownProcessor::new();
        let markdown = "This is a test content with exactly ten words in it.";
        
        let content = processor.process(markdown).unwrap();
        
        // 字数统计基于字符数，不是单词数
        assert!(content.metadata.word_count.is_some());
        assert!(content.metadata.word_count.unwrap() > 0);
    }

    #[test]
    fn test_reading_time_calculation() {
        let processor = MarkdownProcessor::new();
        let words = vec!["word"; 300]; // 300 words
        let markdown = format!("# Title\n\n{}", words.join(" "));
        
        let content = processor.process(&markdown).unwrap();
        
        // 阅读时间基于字符数，300个单词约1500字符，应该大于1分钟
        assert!(content.metadata.reading_time.unwrap() >= 1);
    }

    #[test]
    fn test_empty_markdown() {
        let processor = MarkdownProcessor::new();
        let content = processor.process("").unwrap();
        
        assert_eq!(content.title, "无标题");
        assert_eq!(content.markdown, "");
        assert_eq!(content.metadata.word_count, Some(0));
        assert_eq!(content.metadata.reading_time, Some(1)); // 最小1分钟
    }
}