use crate::{
    adapters::traits::{PlatformAdapter, StyleProvider, ValidationError, ValidationSeverity},
    core::content::{Content, Platform},
    error::Error,
    Result,
};
use async_trait::async_trait;
use regex::Regex;

pub struct ZhihuStyleAdapter {
    math_enabled: bool,
    code_highlight_theme: String,
    max_content_length: usize,
    forbidden_tags: Vec<&'static str>,
}

impl ZhihuStyleAdapter {
    pub fn new() -> Self {
        Self {
            math_enabled: true,
            code_highlight_theme: "github".to_string(),
            max_content_length: 30000, // 知乎字数限制相对宽松
            forbidden_tags: vec![
                "script", "style", "iframe", "object", "embed", "form", "input", "button", "meta",
                "link",
            ],
        }
    }

    pub fn with_math(mut self, enabled: bool) -> Self {
        self.math_enabled = enabled;
        self
    }

    pub fn with_code_theme(mut self, theme: String) -> Self {
        self.code_highlight_theme = theme;
        self
    }

    fn render_math_expressions(&self, html: &str) -> Result<String> {
        if !self.math_enabled {
            return Ok(html.to_string());
        }

        tracing::debug!("渲染数学公式");

        // 处理行内数学公式 $...$
        static INLINE_MATH_REGEX: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
        let inline_math_regex =
            INLINE_MATH_REGEX.get_or_init(|| Regex::new(r"\$([^\$\n]+)\$").unwrap());

        let mut result = inline_math_regex
            .replace_all(html, |caps: &regex::Captures| {
                let formula = &caps[1];
                self.render_katex_inline(formula)
            })
            .to_string();

        // 处理块级数学公式 $$...$$
        static BLOCK_MATH_REGEX: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
        let block_math_regex =
            BLOCK_MATH_REGEX.get_or_init(|| Regex::new(r"\$\$([\s\S]*?)\$\$").unwrap());

        result = block_math_regex
            .replace_all(&result, |caps: &regex::Captures| {
                let formula = &caps[1].trim();
                self.render_katex_block(formula)
            })
            .to_string();

        Ok(result)
    }

    fn render_katex_inline(&self, formula: &str) -> String {
        // 在实际应用中，这里应该调用KaTeX库来渲染数学公式
        // 这里提供一个简化的实现
        format!(
            r#"<span class="ztext-math" data-tex="{}" data-mode="inline">{}</span>"#,
            html_escape::encode_text(formula),
            html_escape::encode_text(formula)
        )
    }

    fn render_katex_block(&self, formula: &str) -> String {
        // 块级数学公式渲染
        format!(
            r#"<div class="ztext-math" data-tex="{}" data-mode="display">{}</div>"#,
            html_escape::encode_text(formula),
            html_escape::encode_text(formula)
        )
    }

    fn enhance_code_blocks(&self, html: &str) -> Result<String> {
        tracing::debug!("增强代码块样式");

        // 为代码块添加知乎样式
        static PRE_REGEX: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
        let pre_regex = PRE_REGEX.get_or_init(|| {
            Regex::new(r#"<pre><code(?:\s+class="language-([^"]*)")?>([^<]*?)</code></pre>"#)
                .unwrap()
        });

        let result = pre_regex.replace_all(html, |caps: &regex::Captures| {
            let language = caps.get(1).map_or("text", |m| m.as_str());
            let code = &caps[2];

            format!(
                r#"<div class="highlight"><pre><code class="language-{}" data-lang="{}">{}</code></pre></div>"#,
                language, language, code
            )
        }).to_string();

        // 增强行内代码样式
        static INLINE_CODE_REGEX: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
        let inline_code_regex =
            INLINE_CODE_REGEX.get_or_init(|| Regex::new(r#"<code>([^<]+)</code>"#).unwrap());

        let result = inline_code_regex
            .replace_all(&result, |caps: &regex::Captures| {
                let code = &caps[1];
                format!(r#"<code class="inline-code">{}</code>"#, code)
            })
            .to_string();

        Ok(result)
    }

    fn sanitize_html(&self, html: &str) -> Result<String> {
        let mut result = html.to_string();

        // 移除禁用的标签
        for tag in &self.forbidden_tags {
            let tag_regex = Regex::new(&format!(r"<{}[^>]*>[\s\S]*?</{}>", tag, tag))
                .map_err(|e| Error::Html(format!("清理标签正则表达式失败: {}", e)))?;
            result = tag_regex.replace_all(&result, "").to_string();

            // 也移除自闭合标签
            let self_closing_regex = Regex::new(&format!(r"<{}\s*[^>]*/>", tag))
                .map_err(|e| Error::Html(format!("清理自闭合标签正则表达式失败: {}", e)))?;
            result = self_closing_regex.replace_all(&result, "").to_string();
        }

        // 清理危险属性
        let dangerous_attrs = ["onclick", "onload", "onerror", "onmouseover", "onfocus"];
        for attr in dangerous_attrs {
            let attr_regex = Regex::new(&format!(r#"{}="[^"]*""#, attr))
                .map_err(|e| Error::Html(format!("清理属性正则表达式失败: {}", e)))?;
            result = attr_regex.replace_all(&result, "").to_string();
        }

        Ok(result)
    }

    fn optimize_images(&self, html: &str) -> Result<String> {
        tracing::debug!("优化图片显示");

        // 为图片添加知乎样式类
        let img_regex = Regex::new(r#"<img([^>]*?)>"#)
            .map_err(|e| Error::Html(format!("图片正则表达式失败: {}", e)))?;

        let result = img_regex
            .replace_all(html, |caps: &regex::Captures| {
                let attrs = &caps[1];

                // 检查是否已有class属性
                if attrs.contains("class=") {
                    let class_regex = Regex::new(r#"class="([^"]*)""#).unwrap();
                    class_regex
                        .replace(attrs, |class_caps: &regex::Captures| {
                            let existing_classes = class_caps.get(1).map_or("", |m| m.as_str());
                            format!(r#"class="{} ztext-image""#, existing_classes)
                        })
                        .to_string()
                } else {
                    format!(r#"<img{} class="ztext-image">"#, attrs)
                }
            })
            .to_string();

        Ok(result)
    }

    fn enhance_tables(&self, html: &str) -> Result<String> {
        tracing::debug!("增强表格样式");

        // 为表格添加知乎样式
        let table_regex = Regex::new(r#"<table([^>]*)>"#).unwrap();
        let result = table_regex
            .replace_all(html, |caps: &regex::Captures| {
                let attrs = &caps[1];
                format!(r#"<table{} class="ztext-table">"#, attrs)
            })
            .to_string();

        Ok(result)
    }

    fn process_lists(&self, html: &str) -> Result<String> {
        // 处理列表，确保知乎格式兼容
        let mut result = html.to_string();

        // 为有序列表添加样式
        let ol_regex = Regex::new(r#"<ol([^>]*)>"#).unwrap();
        result = ol_regex
            .replace_all(&result, |caps: &regex::Captures| {
                let attrs = &caps[1];
                format!(r#"<ol{} class="ztext-list">"#, attrs)
            })
            .to_string();

        // 为无序列表添加样式
        let ul_regex = Regex::new(r#"<ul([^>]*)>"#).unwrap();
        result = ul_regex
            .replace_all(&result, |caps: &regex::Captures| {
                let attrs = &caps[1];
                format!(r#"<ul{} class="ztext-list">"#, attrs)
            })
            .to_string();

        Ok(result)
    }

    #[allow(dead_code)]
    fn add_zhihu_meta(&self, html: &str, content: &Content) -> Result<String> {
        // 添加知乎特定的元数据
        let meta_section = if !content.metadata.tags.is_empty() {
            let tags_html = content
                .metadata
                .tags
                .iter()
                .map(|tag| format!(r#"<span class="ztext-tag">#{}</span>"#, tag))
                .collect::<Vec<_>>()
                .join(" ");

            format!(
                r#"<div class="ztext-meta">
                    <div class="ztext-tags">{}</div>
                </div>"#,
                tags_html
            )
        } else {
            String::new()
        };

        Ok(format!("{}{}", html, meta_section))
    }
}

impl Default for ZhihuStyleAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PlatformAdapter for ZhihuStyleAdapter {
    fn platform(&self) -> Platform {
        Platform::Zhihu
    }

    fn adapt_html(&self, html: &str) -> Result<String> {
        tracing::info!("开始适配知乎样式");

        // 1. 清理和消毒HTML
        let sanitized = self.sanitize_html(html)?;

        // 2. 渲染数学公式
        let with_math = self.render_math_expressions(&sanitized)?;

        // 3. 增强代码块
        let enhanced_code = self.enhance_code_blocks(&with_math)?;

        // 4. 优化图片
        let optimized_images = self.optimize_images(&enhanced_code)?;

        // 5. 增强表格
        let enhanced_tables = self.enhance_tables(&optimized_images)?;

        // 6. 处理列表
        let processed_lists = self.process_lists(&enhanced_tables)?;

        tracing::info!("知乎样式适配完成");
        Ok(processed_lists)
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

        if content.title.len() > 100 {
            errors.push(ValidationError {
                field: "title".to_string(),
                message: "标题长度不能超过100个字符".to_string(),
                severity: ValidationSeverity::Warning,
            });
        }

        // 检查标签数量
        if content.metadata.tags.len() > 5 {
            errors.push(ValidationError {
                field: "tags".to_string(),
                message: "标签数量不能超过5个".to_string(),
                severity: ValidationSeverity::Warning,
            });
        }

        // 检查是否包含禁用内容
        let forbidden_keywords = ["广告", "推广", "联系方式"];
        for keyword in forbidden_keywords {
            if content.markdown.contains(keyword) {
                errors.push(ValidationError {
                    field: "content".to_string(),
                    message: format!("内容包含可能被禁止的关键词: {}", keyword),
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
                    "知乎内容验证失败: {}",
                    error_messages.join("; ")
                )));
            }
        }

        Ok(())
    }

    async fn preprocess_images(&self, html: &str) -> Result<String> {
        // 知乎的图片预处理
        // 可以实现图片压缩、格式转换等
        tracing::debug!("预处理知乎图片");
        self.optimize_images(html)
    }
}

impl StyleProvider for ZhihuStyleAdapter {
    fn get_styles(&self) -> &str {
        r#"
        .ztext-image { max-width: 100%; height: auto; display: block; margin: 20px auto; }
        .ztext-table { width: 100%; border-collapse: collapse; margin: 20px 0; }
        .ztext-list { margin: 15px 0; padding-left: 30px; }
        .ztext-math { font-family: 'Times New Roman', serif; }
        .ztext-tag { 
            display: inline-block; 
            background: #f0f0f0; 
            color: #666; 
            padding: 2px 8px; 
            border-radius: 12px; 
            font-size: 12px; 
            margin: 2px; 
        }
        .ztext-meta { margin-top: 30px; padding-top: 20px; border-top: 1px solid #eee; }
        .highlight { background: #f8f8f8; border-radius: 4px; padding: 16px; margin: 16px 0; }
        .inline-code { 
            background: #f0f0f0; 
            color: #d73a49; 
            padding: 2px 4px; 
            border-radius: 3px; 
            font-family: 'SFMono-Regular', Consolas, monospace; 
        }
        "#
    }

    fn apply_inline_styles(&self, html: &str) -> Result<String> {
        // 知乎支持CSS类，不需要完全内联化
        Ok(html.to_string())
    }
}
