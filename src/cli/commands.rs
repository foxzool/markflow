use crate::{
    adapters::{PlatformAdapter, WeChatStyleAdapter, ZhihuStyleAdapter},
    cli::{args::AppConfig, ConfigAction, Platform, TemplateAction},
    core::{MarkdownProcessor, ProcessingPipeline},
    Result,
};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::PathBuf;
use tokio::{fs, sync::mpsc};
use tracing::{debug, error, info, warn};

pub async fn process_command(
    input: PathBuf,
    output: Option<PathBuf>,
    platform: Option<Platform>,
    preview: bool,
) -> Result<()> {
    info!("处理文件: {:?}", input);

    // 读取配置
    let config = AppConfig::load_from_file(&AppConfig::get_config_path())?;

    // 检查输入文件是否存在
    if !input.exists() {
        return Err(crate::error::Error::IO(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("输入文件不存在: {:?}", input),
        )));
    }

    // 读取Markdown内容
    let markdown_content = fs::read_to_string(&input).await?;

    // 处理Markdown
    let processor = MarkdownProcessor::new();
    let pipeline = ProcessingPipeline::default();

    let content = processor.process(&markdown_content)?;
    let processed_content = pipeline.process(content).await?;

    // 确定目标平台
    let target_platforms = determine_target_platforms(platform, &config);

    for target_platform in target_platforms {
        match target_platform {
            Platform::WeChat => {
                let adapter = WeChatStyleAdapter::new();
                adapter.validate_content(&processed_content)?;
                let adapted_html = adapter.adapt_html(&processed_content.html)?;

                if preview {
                    println!("=== 微信公众号 HTML 预览 ===");
                    println!("{}", adapted_html);
                } else {
                    save_output(
                        &processed_content,
                        &adapted_html,
                        &target_platform,
                        &output,
                        &config,
                    )
                    .await?;
                }
            }
            Platform::Zhihu => {
                let adapter = ZhihuStyleAdapter::new()
                    .with_math(config.zhihu.enable_math)
                    .with_code_theme(config.zhihu.code_theme.clone());
                adapter.validate_content(&processed_content)?;
                let adapted_html = adapter.adapt_html(&processed_content.html)?;

                if preview {
                    println!("=== 知乎 HTML 预览 ===");
                    println!("{}", adapted_html);
                } else {
                    save_output(
                        &processed_content,
                        &adapted_html,
                        &target_platform,
                        &output,
                        &config,
                    )
                    .await?;
                }
            }
            Platform::All => {
                // 已经在外层循环处理
                unreachable!()
            }
        }
    }

    if !preview {
        info!("处理完成！");
    }

    Ok(())
}

pub async fn watch_command(
    directory: PathBuf,
    output: Option<PathBuf>,
    pattern: String,
) -> Result<()> {
    info!("开始监控目录: {:?}", directory);
    info!("文件模式: {}", pattern);

    if !directory.exists() {
        return Err(crate::error::Error::IO(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("目录不存在: {:?}", directory),
        )));
    }

    let (tx, mut rx) = mpsc::channel(100);

    // 创建文件监控器
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| match res {
        Ok(event) => {
            if let Err(e) = tx.blocking_send(event) {
                error!("发送文件事件失败: {}", e);
            }
        }
        Err(e) => error!("文件监控错误: {}", e),
    })
    .map_err(|e| crate::error::Error::Other(format!("创建文件监控器失败: {}", e)))?;

    // 开始监控
    watcher
        .watch(&directory, RecursiveMode::Recursive)
        .map_err(|e| crate::error::Error::Other(format!("启动文件监控失败: {}", e)))?;

    info!("文件监控已启动，按 Ctrl+C 停止");

    // 处理文件事件
    while let Some(event) = rx.recv().await {
        if let EventKind::Modify(_) | EventKind::Create(_) = event.kind {
            for path in &event.paths {
                if path.extension().and_then(|s| s.to_str()) == Some("md") {
                    info!("检测到文件变化: {:?}", path);

                    // 处理文件
                    if let Err(e) =
                        process_command(path.clone(), output.clone(), Some(Platform::All), false)
                            .await
                    {
                        error!("处理文件失败: {}", e);
                    }
                }
            }
        }
    }

    Ok(())
}

pub async fn publish_command(_content: String, platform: Platform, draft: bool) -> Result<()> {
    info!("发布内容到平台: {}", platform);

    // 这里应该实现发布逻辑
    // 由于需要浏览器自动化和API集成，这里提供一个框架
    match platform {
        Platform::WeChat => {
            info!("正在发布到微信公众号...");
            if draft {
                info!("创建草稿模式");
                // TODO: 实现微信公众号草稿创建
            } else {
                warn!("微信公众号不支持直接发布，将创建草稿");
                // TODO: 实现微信公众号草稿创建
            }
        }
        Platform::Zhihu => {
            info!("正在发布到知乎...");
            // TODO: 实现知乎自动发布
            warn!("知乎发布功能正在开发中");
        }
        Platform::All => {
            return Err(crate::error::Error::Other(
                "发布时不能选择'all'平台".to_string(),
            ));
        }
    }

    Ok(())
}

pub async fn serve_command(port: u16, host: String, _static_dir: Option<PathBuf>) -> Result<()> {
    info!("启动Web服务器 {}:{}", host, port);

    // TODO: 实现Web服务器
    warn!("Web服务器功能正在开发中");

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    Ok(())
}

pub async fn config_command(action: ConfigAction) -> Result<()> {
    let config_path = AppConfig::get_config_path();

    match action {
        ConfigAction::Show => {
            let config = AppConfig::load_from_file(&config_path)?;
            println!("当前配置:");
            println!("{}", toml::to_string_pretty(&config).unwrap());
        }
        ConfigAction::Set { key, value } => {
            let mut config = AppConfig::load_from_file(&config_path)?;
            config.set_value(&key, &value)?;
            config.save_to_file(&config_path)?;
            info!("配置已更新: {} = {}", key, value);
        }
        ConfigAction::Get { key } => {
            let config = AppConfig::load_from_file(&config_path)?;
            if let Some(value) = config.get_value(&key) {
                println!("{}", value);
            } else {
                error!("配置键不存在: {}", key);
            }
        }
        ConfigAction::Init => {
            let config = AppConfig::default();
            config.save_to_file(&config_path)?;
            info!("已初始化默认配置到: {:?}", config_path);
        }
    }

    Ok(())
}

pub async fn template_command(action: TemplateAction) -> Result<()> {
    match action {
        TemplateAction::List => {
            info!("列出模板功能正在开发中");
        }
        TemplateAction::Create { name, file } => {
            info!("创建模板 '{}' from {:?}", name, file);
        }
        TemplateAction::Delete { name } => {
            info!("删除模板 '{}'", name);
        }
        TemplateAction::Apply {
            name,
            input,
            output: _,
        } => {
            info!("应用模板 '{}' 到 {:?}", name, input);
        }
    }

    Ok(())
}

// 辅助函数
fn determine_target_platforms(platform: Option<Platform>, config: &AppConfig) -> Vec<Platform> {
    match platform {
        Some(Platform::All) => vec![Platform::WeChat, Platform::Zhihu],
        Some(platform) => vec![platform],
        None => {
            // 使用配置中的默认平台
            match config.general.default_platform.as_deref() {
                Some("wechat") => vec![Platform::WeChat],
                Some("zhihu") => vec![Platform::Zhihu],
                Some("all") | None => vec![Platform::WeChat, Platform::Zhihu],
                _ => vec![Platform::WeChat, Platform::Zhihu],
            }
        }
    }
}

async fn save_output(
    content: &crate::core::Content,
    html: &str,
    platform: &Platform,
    output_override: &Option<PathBuf>,
    config: &AppConfig,
) -> Result<()> {
    let output_dir = output_override
        .as_ref()
        .unwrap_or(&config.output.output_dir);

    // 创建输出目录
    if !output_dir.exists() {
        fs::create_dir_all(output_dir).await?;
    }

    // 生成文件名
    let filename = generate_filename(&content.title, platform, &config.output.filename_pattern);

    let output_path = if config.output.create_subdirs {
        let platform_dir = output_dir.join(platform.to_string());
        if !platform_dir.exists() {
            fs::create_dir_all(&platform_dir).await?;
        }
        platform_dir.join(filename)
    } else {
        output_dir.join(filename)
    };

    // 写入文件
    fs::write(&output_path, html).await?;

    info!("已保存到: {:?}", output_path);

    // 备份功能
    if config.general.backup_enabled {
        if let Some(backup_dir) = &config.output.backup_dir {
            backup_file(&output_path, backup_dir).await?;
        }
    }

    Ok(())
}

fn generate_filename(title: &str, platform: &Platform, pattern: &str) -> String {
    // 清理标题作为文件名
    let safe_title = title
        .chars()
        .map(|c| match c {
            '/' | '\\' | '?' | '%' | '*' | ':' | '|' | '"' | '<' | '>' => '_',
            c => c,
        })
        .collect::<String>();

    // 应用模式
    pattern
        .replace("{title}", &safe_title)
        .replace("{platform}", &platform.to_string())
        .replace(
            "{timestamp}",
            &chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string(),
        )
}

async fn backup_file(source: &PathBuf, backup_dir: &PathBuf) -> Result<()> {
    if !backup_dir.exists() {
        fs::create_dir_all(backup_dir).await?;
    }

    let filename = source
        .file_name()
        .ok_or_else(|| crate::error::Error::Other("无法获取文件名".to_string()))?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_filename = format!("{}_{}", timestamp, filename.to_string_lossy());
    let backup_path = backup_dir.join(backup_filename);

    fs::copy(source, &backup_path).await?;
    debug!("已备份到: {:?}", backup_path);

    Ok(())
}
