use crate::{
    adapters::traits::{PlatformAdapter, StyleProvider, ValidationError, ValidationSeverity},
    core::content::{Content, Platform},
    error::Error,
    Result,
};
use async_trait::async_trait;
use regex::Regex;
use scraper::{Html, Selector};
use std::collections::HashMap;

pub struct WeChatStyleAdapter {
    inline_styles: HashMap<String, String>,
    max_content_length: usize,
    #[allow(dead_code)]
    allowed_tags: Vec<&'static str>,
}

impl WeChatStyleAdapter {
    pub fn new() -> Self {
        let mut inline_styles = HashMap::new();

        // 微信公众号样式规则
        inline_styles.insert("body".to_string(), 
            "font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif; color: #333; line-height: 1.6; margin: 0; padding: 20px;".to_string());

        inline_styles.insert(
            "p".to_string(),
            "font-size: 16px; line-height: 1.8; margin: 20px 0; color: #333; text-align: justify;"
                .to_string(),
        );

        inline_styles.insert("h1".to_string(), 
            "font-size: 24px; font-weight: bold; text-align: center; margin: 30px 0 20px 0; color: #2c3e50; border-bottom: 2px solid #3498db; padding-bottom: 10px;".to_string());

        inline_styles.insert("h2".to_string(), 
            "font-size: 20px; font-weight: bold; margin: 25px 0 15px 0; color: #2c3e50; border-left: 4px solid #3498db; padding-left: 15px;".to_string());

        inline_styles.insert(
            "h3".to_string(),
            "font-size: 18px; font-weight: bold; margin: 20px 0 10px 0; color: #34495e;"
                .to_string(),
        );

        inline_styles.insert(
            "h4".to_string(),
            "font-size: 16px; font-weight: bold; margin: 15px 0 8px 0; color: #34495e;".to_string(),
        );

        inline_styles.insert("blockquote".to_string(), 
            "border-left: 4px solid #ddd; margin: 20px 0; padding: 10px 20px; background-color: #f9f9f9; font-style: italic; color: #666;".to_string());

        inline_styles.insert("pre".to_string(), 
            "background-color: #f8f8f8; border: 1px solid #ddd; border-radius: 6px; padding: 15px; margin: 20px 0; overflow-x: auto; font-family: 'Consolas', 'Monaco', 'Courier New', monospace; font-size: 14px; line-height: 1.4;".to_string());

        inline_styles.insert("code".to_string(), 
            "background-color: #f1f2f3; padding: 2px 6px; border-radius: 3px; font-family: 'Consolas', 'Monaco', 'Courier New', monospace; font-size: 14px; color: #e96900;".to_string());

        inline_styles.insert("img".to_string(), 
            "max-width: 100%; height: auto; display: block; margin: 20px auto; border-radius: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.1);".to_string());

        inline_styles.insert(
            "table".to_string(),
            "width: 100%; border-collapse: collapse; margin: 20px 0; font-size: 14px;".to_string(),
        );

        inline_styles.insert("th".to_string(), 
            "background-color: #f1f2f3; padding: 12px; text-align: left; border: 1px solid #ddd; font-weight: bold;".to_string());

        inline_styles.insert(
            "td".to_string(),
            "padding: 12px; text-align: left; border: 1px solid #ddd;".to_string(),
        );

        inline_styles.insert(
            "ul".to_string(),
            "margin: 15px 0; padding-left: 30px;".to_string(),
        );

        inline_styles.insert(
            "ol".to_string(),
            "margin: 15px 0; padding-left: 30px;".to_string(),
        );

        inline_styles.insert(
            "li".to_string(),
            "margin: 8px 0; line-height: 1.6;".to_string(),
        );

        inline_styles.insert(
            "a".to_string(),
            "color: #3498db; text-decoration: none; border-bottom: 1px dotted #3498db;".to_string(),
        );

        inline_styles.insert(
            "strong".to_string(),
            "font-weight: bold; color: #2c3e50;".to_string(),
        );

        inline_styles.insert(
            "em".to_string(),
            "font-style: italic; color: #7f8c8d;".to_string(),
        );

        Self {
            inline_styles,
            max_content_length: 20000, // 微信公众号字数限制
            allowed_tags: vec![
                "p",
                "h1",
                "h2",
                "h3",
                "h4",
                "h5",
                "h6",
                "br",
                "hr",
                "strong",
                "b",
                "em",
                "i",
                "u",
                "s",
                "del",
                "ins",
                "blockquote",
                "pre",
                "code",
                "span",
                "div",
                "ul",
                "ol",
                "li",
                "dl",
                "dt",
                "dd",
                "table",
                "thead",
                "tbody",
                "tr",
                "th",
                "td",
                "img",
                "a",
                "section",
                "article",
                "aside",
                "nav",
            ],
        }
    }

    fn inline_all_styles(&self, html: &str) -> Result<String> {
        let _document = Html::parse_document(html);
        let mut result = html.to_string();

        // 预编译正则表达式，避免在循环中重复创建
        let style_regex = Regex::new(r#"style="([^"]*)""#)
            .map_err(|e| Error::Html(format!("正则表达式创建失败: {}", e)))?;

        for (selector_str, style) in &self.inline_styles {
            let _selector = Selector::parse(selector_str)
                .map_err(|e| Error::Html(format!("CSS选择器解析失败: {}", e)))?;

            // 使用正则表达式来替换标签，添加内联样式
            let tag_regex = Regex::new(&format!(r"<{}(\s[^>]*)?>", selector_str))
                .map_err(|e| Error::Html(format!("正则表达式创建失败: {}", e)))?;

            result = tag_regex
                .replace_all(&result, |caps: &regex::Captures| {
                    let existing_attrs = caps.get(1).map_or("", |m| m.as_str());

                    // 检查是否已有style属性
                    if existing_attrs.contains("style=") {
                        style_regex
                            .replace(existing_attrs, |style_caps: &regex::Captures| {
                                let existing_style = style_caps.get(1).map_or("", |m| m.as_str());
                                format!(r#"style="{}; {}""#, existing_style, style)
                            })
                            .to_string()
                    } else {
                        format!("<{}{} style=\"{}\">", selector_str, existing_attrs, style)
                    }
                })
                .to_string();
        }

        Ok(result)
    }

    fn convert_external_links(&self, html: &str) -> Result<String> {
        let link_regex = Regex::new(r#"<a\s+[^>]*href="([^"]*)"[^>]*>([^<]*)</a>"#)
            .map_err(|e| Error::Html(format!("链接正则表达式失败: {}", e)))?;

        let mut footnotes = Vec::new();
        let mut footnote_counter = 1;

        let result = link_regex
            .replace_all(html, |caps: &regex::Captures| {
                let url = &caps[1];
                let text = &caps[2];

                if url.starts_with("http") {
                    // 外部链接转换为脚注
                    footnotes.push(format!("[{}] {}", footnote_counter, url));
                    let footnote_mark = format!("{}[{}]", text, footnote_counter);
                    footnote_counter += 1;
                    footnote_mark
                } else {
                    // 保留内部链接
                    format!(
                        r#"<span style="color: #3498db; text-decoration: underline;">{}</span>"#,
                        text
                    )
                }
            })
            .to_string();

        // 添加脚注
        if !footnotes.is_empty() {
            let footnotes_section = format!(
                r#"
                <hr style="margin: 30px 0; border: none; border-top: 1px solid #ddd;">
                <h4 style="font-size: 14px; color: #666; margin-bottom: 10px;">参考链接：</h4>
                <div style="font-size: 12px; color: #666; line-height: 1.8;">
                    {}
                </div>
                "#,
                footnotes.join("<br>")
            );
            Ok(format!("{}{}", result, footnotes_section))
        } else {
            Ok(result)
        }
    }

    fn optimize_for_mobile(&self, html: &str) -> Result<String> {
        // 移动端优化
        let mut result = html.to_string();

        // 确保图片响应式
        let img_regex = Regex::new(r#"<img([^>]*)>"#).unwrap();
        result = img_regex.replace_all(&result, |caps: &regex::Captures| {
            let attrs = &caps[1];
            if !attrs.contains("style=") {
                format!(r#"<img{} style="max-width: 100%; height: auto; display: block; margin: 20px auto;">"#, attrs)
            } else {
                caps[0].to_string()
            }
        }).to_string();

        // 优化表格显示
        let table_regex = Regex::new(r#"<table([^>]*)>"#).unwrap();
        result = table_regex.replace_all(&result, |caps: &regex::Captures| {
            let attrs = &caps[1];
            format!(r#"<table{} style="width: 100%; border-collapse: collapse; margin: 20px 0; font-size: 14px; overflow-x: auto;">"#, attrs)
        }).to_string();

        Ok(result)
    }

    fn sanitize_html(&self, html: &str) -> Result<String> {
        let _document = Html::parse_document(html);

        // 移除不允许的标签和属性
        let mut result = html.to_string();

        // 移除script和style标签
        let script_regex = Regex::new(r"<script[^>]*>[\s\S]*?</script>").unwrap();
        result = script_regex.replace_all(&result, "").to_string();

        let style_regex = Regex::new(r"<style[^>]*>[\s\S]*?</style>").unwrap();
        result = style_regex.replace_all(&result, "").to_string();

        // 移除危险属性
        let dangerous_attrs = ["onclick", "onload", "onerror", "javascript:"];
        for attr in dangerous_attrs {
            let attr_regex = Regex::new(&format!(r#"{}="[^"]*""#, attr)).unwrap();
            result = attr_regex.replace_all(&result, "").to_string();
        }

        Ok(result)
    }
}

impl Default for WeChatStyleAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PlatformAdapter for WeChatStyleAdapter {
    fn platform(&self) -> Platform {
        Platform::WeChat
    }

    fn adapt_html(&self, html: &str) -> Result<String> {
        tracing::info!("开始适配微信公众号样式");

        // 1. 清理和消毒HTML
        let sanitized = self.sanitize_html(html)?;

        // 2. 内联所有样式
        let styled = self.inline_all_styles(&sanitized)?;

        // 3. 转换外部链接为脚注
        let with_footnotes = self.convert_external_links(&styled)?;

        // 4. 移动端优化
        let optimized = self.optimize_for_mobile(&with_footnotes)?;

        tracing::info!("微信公众号样式适配完成");
        Ok(optimized)
    }

    fn validate_content(&self, content: &Content) -> Result<()> {
        let mut errors = Vec::new();

        // 检查内容长度
        if content.markdown.len() > self.max_content_length {
            errors.push(ValidationError {
                field: "content".to_string(),
                message: format!(
                    "内容长度超过限制（当前：{}，限制：{}）",
                    content.markdown.len(),
                    self.max_content_length
                ),
                severity: ValidationSeverity::Error,
            });
        }

        // 检查标题
        if content.title.is_empty() {
            errors.push(ValidationError {
                field: "title".to_string(),
                message: "标题不能为空".to_string(),
                severity: ValidationSeverity::Error,
            });
        }

        if content.title.len() > 64 {
            errors.push(ValidationError {
                field: "title".to_string(),
                message: "标题长度不能超过64个字符".to_string(),
                severity: ValidationSeverity::Error,
            });
        }

        // 检查封面图片
        if let Some(ref cover) = content.metadata.cover_image {
            if !cover.starts_with("http") && !cover.starts_with("data:") {
                errors.push(ValidationError {
                    field: "cover_image".to_string(),
                    message: "封面图片必须是有效的URL或base64数据".to_string(),
                    severity: ValidationSeverity::Warning,
                });
            }
        }

        if !errors.is_empty() {
            let error_messages: Vec<String> = errors
                .iter()
                .filter(|e| matches!(e.severity, ValidationSeverity::Error))
                .map(|e| format!("{}: {}", e.field, e.message))
                .collect();

            if !error_messages.is_empty() {
                return Err(Error::Publishing(format!(
                    "微信公众号内容验证失败: {}",
                    error_messages.join("; ")
                )));
            }
        }

        Ok(())
    }

    async fn preprocess_images(&self, html: &str) -> Result<String> {
        // 微信公众号的图片预处理
        // 这里可以实现图片上传到微信服务器的逻辑
        tracing::debug!("预处理微信公众号图片");
        Ok(html.to_string())
    }
}

impl StyleProvider for WeChatStyleAdapter {
    fn get_styles(&self) -> &str {
        // 返回完整的CSS样式字符串
        r#"
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif; color: #333; line-height: 1.6; margin: 0; padding: 20px; }
        p { font-size: 16px; line-height: 1.8; margin: 20px 0; color: #333; text-align: justify; }
        h1 { font-size: 24px; font-weight: bold; text-align: center; margin: 30px 0 20px 0; color: #2c3e50; border-bottom: 2px solid #3498db; padding-bottom: 10px; }
        h2 { font-size: 20px; font-weight: bold; margin: 25px 0 15px 0; color: #2c3e50; border-left: 4px solid #3498db; padding-left: 15px; }
        /* ... 更多样式 ... */
        "#
    }

    fn apply_inline_styles(&self, html: &str) -> Result<String> {
        self.inline_all_styles(html)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wechat_adapter_creation() {
        let adapter = WeChatStyleAdapter::new();
        assert_eq!(adapter.platform(), Platform::WeChat);
        assert_eq!(adapter.max_content_length, 20000);
        assert!(!adapter.inline_styles.is_empty());
    }

    #[test]
    fn test_inline_styles_application() {
        let adapter = WeChatStyleAdapter::new();
        let html = "<h1>Test Title</h1><p>Test content</p>";

        let result = adapter.inline_all_styles(html).unwrap();

        assert!(result.contains("style="));
        assert!(result.contains("font-size: 24px"));
        assert!(result.contains("font-size: 16px"));
    }

    #[test]
    fn test_external_links_conversion() {
        let adapter = WeChatStyleAdapter::new();
        let html = r#"<p>Visit <a href="https://example.com">Example</a> and <a href="/internal">Internal</a>.</p>"#;

        let result = adapter.convert_external_links(html).unwrap();

        assert!(result.contains("Example[1]"));
        assert!(result.contains("参考链接"));
        assert!(result.contains("https://example.com"));
        assert!(result.contains("Internal")); // Internal link preserved
    }

    #[test]
    fn test_mobile_optimization() {
        let adapter = WeChatStyleAdapter::new();
        let html = r#"<img src="test.jpg"><table><tr><td>data</td></tr></table>"#;

        let result = adapter.optimize_for_mobile(html).unwrap();

        assert!(result.contains("max-width: 100%"));
        assert!(result.contains("overflow-x: auto"));
    }

    #[test]
    fn test_html_sanitization() {
        let adapter = WeChatStyleAdapter::new();
        let html = r#"<script>alert('test')</script><p onclick="alert('click')">Content</p><style>body{color:red}</style>"#;

        let result = adapter.sanitize_html(html).unwrap();

        assert!(!result.contains("<script>"));
        assert!(!result.contains("<style>"));
        assert!(!result.contains("onclick"));
        assert!(result.contains("Content"));
    }

    #[test]
    fn test_content_validation() {
        let adapter = WeChatStyleAdapter::new();

        // Valid content
        let content = Content::new("Valid Title".to_string(), "Short content".to_string());
        assert!(adapter.validate_content(&content).is_ok());

        // Empty title
        let mut invalid_content = Content::new("".to_string(), "Content".to_string());
        assert!(adapter.validate_content(&invalid_content).is_err());

        // Too long title
        invalid_content.title = "a".repeat(100);
        assert!(adapter.validate_content(&invalid_content).is_err());
    }

    #[test]
    fn test_full_adaptation_flow() {
        let adapter = WeChatStyleAdapter::new();
        let html = r#"<h1>Test</h1><p>Content with <a href="https://example.com">link</a></p>"#;

        let result = adapter.adapt_html(html).unwrap();

        assert!(result.contains("style="));
        assert!(result.contains("link[1]"));
        assert!(result.contains("参考链接"));
        assert!(!result.contains("<script>"));
    }
}
