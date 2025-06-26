use markflow::core::{MarkdownProcessor, content::Platform};
use markflow::adapters::{WeChatStyleAdapter, ZhihuStyleAdapter, PlatformAdapter};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_end_to_end_processing() {
    let processor = MarkdownProcessor::new();
    let wechat_adapter = WeChatStyleAdapter::new();
    let zhihu_adapter = ZhihuStyleAdapter::new();
    
    let markdown = r#"---
title: "集成测试文章"
author: "测试作者"
description: "这是一个集成测试"
tags: "测试,集成"
---

# 测试标题

这是一个测试段落，包含**粗体**文字。

## 代码示例

```rust
fn main() {
    println!("Hello, world!");
}
```

## 列表

- 项目 1
- 项目 2
- 项目 3

## 表格

| 名称 | 年龄 |
|------|------|
| 张三 | 30   |
| 李四 | 25   |
"#;

    // 处理Markdown
    let content = processor.process(markdown).unwrap();
    
    // 验证基本内容
    assert_eq!(content.title, "集成测试文章");
    assert_eq!(content.metadata.author, Some("测试作者".to_string()));
    assert!(!content.html.is_empty());
    
    // 验证微信适配
    assert!(wechat_adapter.validate_content(&content).is_ok());
    let wechat_html = wechat_adapter.adapt_html(&content.html).unwrap();
    assert!(wechat_html.contains("style="));
    
    // 验证知乎适配
    assert!(zhihu_adapter.validate_content(&content).is_ok());
    let zhihu_html = zhihu_adapter.adapt_html(&content.html).unwrap();
    assert!(!zhihu_html.is_empty());
}

#[test]
fn test_file_processing_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.md");
    let output_dir = temp_dir.path().join("output");
    
    // 创建测试文件
    let test_content = r#"# 文件处理测试

这是一个测试文件的内容。

包含一些**重要**的信息。
"#;
    
    fs::write(&input_file, test_content).unwrap();
    fs::create_dir_all(&output_dir).unwrap();
    
    // 处理文件
    let processor = MarkdownProcessor::new();
    let content = processor.process(test_content).unwrap();
    
    let wechat_adapter = WeChatStyleAdapter::new();
    let wechat_html = wechat_adapter.adapt_html(&content.html).unwrap();
    
    // 写入输出文件
    let output_file = output_dir.join("test_wechat.html");
    fs::write(&output_file, &wechat_html).unwrap();
    
    // 验证输出文件
    assert!(output_file.exists());
    let saved_content = fs::read_to_string(&output_file).unwrap();
    assert!(saved_content.contains("style="));
    assert!(saved_content.contains("文件处理测试"));
}

#[test]
fn test_multiple_platform_processing() {
    let processor = MarkdownProcessor::new();
    let wechat_adapter = WeChatStyleAdapter::new();
    let zhihu_adapter = ZhihuStyleAdapter::new();
    
    let markdown = r#"# 多平台测试

这是一个多平台处理的测试。

包含数学公式：$E = mc^2$

和代码块：

```python
def hello():
    print("Hello, World!")
```
"#;

    let content = processor.process(markdown).unwrap();
    
    // 测试微信平台
    assert_eq!(wechat_adapter.platform(), Platform::WeChat);
    let wechat_html = wechat_adapter.adapt_html(&content.html).unwrap();
    assert!(wechat_html.contains("style="));
    
    // 测试知乎平台
    assert_eq!(zhihu_adapter.platform(), Platform::Zhihu);
    let zhihu_html = zhihu_adapter.adapt_html(&content.html).unwrap();
    assert!(!zhihu_html.is_empty());
    
    // 验证两个平台的输出不同
    assert_ne!(wechat_html, zhihu_html);
}

#[test]
fn test_error_handling() {
    let wechat_adapter = WeChatStyleAdapter::new();
    
    // 测试无效的HTML
    let invalid_html = "<invalid><script>alert('test')</script>";
    let result = wechat_adapter.adapt_html(invalid_html);
    
    // 应该能够处理并清理无效HTML
    assert!(result.is_ok());
    let cleaned = result.unwrap();
    assert!(!cleaned.contains("<script>"));
}

#[test]
fn test_large_content_handling() {
    let processor = MarkdownProcessor::new();
    let wechat_adapter = WeChatStyleAdapter::new();
    
    // 创建一个大文档
    let mut large_markdown = String::from("# 大文档测试\n\n");
    for i in 0..1000 {
        large_markdown.push_str(&format!("这是第{}段内容。", i));
        if i % 10 == 0 {
            large_markdown.push('\n');
        }
    }
    
    let content = processor.process(&large_markdown).unwrap();
    assert!(content.metadata.word_count.unwrap() > 1000);
    
    // 测试内容长度验证
    let validation_result = wechat_adapter.validate_content(&content);
    // 可能会因为内容过长而失败，这是预期的
    if validation_result.is_err() {
        // 验证错误消息包含长度限制信息
        let error_msg = format!("{}", validation_result.unwrap_err());
        assert!(error_msg.contains("长度") || error_msg.contains("限制"));
    }
}

#[test]
fn test_concurrent_processing() {
    use std::thread;
    use std::sync::Arc;
    
    let processor = Arc::new(MarkdownProcessor::new());
    let wechat_adapter = Arc::new(WeChatStyleAdapter::new());
    
    let markdown_samples = vec![
        "# 测试1\n\n内容1",
        "# 测试2\n\n内容2",
        "# 测试3\n\n内容3",
    ];
    
    let mut handles = vec![];
    
    for (i, markdown) in markdown_samples.into_iter().enumerate() {
        let processor_clone = Arc::clone(&processor);
        let adapter_clone = Arc::clone(&wechat_adapter);
        let markdown_owned = markdown.to_string();
        
        let handle = thread::spawn(move || {
            let content = processor_clone.process(&markdown_owned).unwrap();
            let html = adapter_clone.adapt_html(&content.html).unwrap();
            (i, html.len())
        });
        
        handles.push(handle);
    }
    
    // 等待所有线程完成
    for handle in handles {
        let (index, html_len) = handle.join().unwrap();
        assert!(html_len > 0, "Thread {} produced empty HTML", index);
    }
}