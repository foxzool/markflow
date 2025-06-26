pub mod args;
pub mod commands;

use crate::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;

pub use args::*;
pub use commands::*;

#[derive(Parser)]
#[command(
    name = "markflow",
    version = "0.1.0",
    author = "MarkFlow Team",
    about = "Markdown转HTML并发布到微信公众号和知乎的工具"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// 启用调试日志
    #[arg(short, long, global = true)]
    pub debug: bool,

    /// 配置文件路径
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 处理Markdown文件
    Process {
        /// 输入的Markdown文件路径
        #[arg(short, long)]
        input: PathBuf,

        /// 输出目录（可选）
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// 目标平台
        #[arg(short, long)]
        platform: Option<Platform>,

        /// 预览模式（不写入文件）
        #[arg(long)]
        preview: bool,
    },

    /// 监控目录变化并自动处理
    Watch {
        /// 要监控的目录
        #[arg(short, long)]
        directory: PathBuf,

        /// 输出目录
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// 文件模式匹配（如 "*.md"）
        #[arg(short, long, default_value = "*.md")]
        pattern: String,
    },

    /// 发布内容到平台
    Publish {
        /// 内容ID或文件路径
        #[arg(short, long)]
        content: String,

        /// 目标平台
        #[arg(short, long)]
        platform: Platform,

        /// 是否为草稿模式
        #[arg(long)]
        draft: bool,
    },

    /// 启动Web服务器
    Serve {
        /// 服务器端口
        #[arg(short, long, default_value = "8080")]
        port: u16,

        /// 绑定地址
        #[arg(short, long, default_value = "127.0.0.1")]
        host: String,

        /// 静态文件目录
        #[arg(long)]
        static_dir: Option<PathBuf>,
    },

    /// 配置管理
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// 模板管理
    Template {
        #[command(subcommand)]
        action: TemplateAction,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// 显示当前配置
    Show,

    /// 设置配置项
    Set {
        /// 配置键
        key: String,
        /// 配置值
        value: String,
    },

    /// 获取配置项
    Get {
        /// 配置键
        key: String,
    },

    /// 初始化默认配置
    Init,
}

#[derive(Subcommand)]
pub enum TemplateAction {
    /// 列出所有模板
    List,

    /// 创建新模板
    Create {
        /// 模板名称
        name: String,
        /// 模板文件路径
        file: PathBuf,
    },

    /// 删除模板
    Delete {
        /// 模板名称
        name: String,
    },

    /// 应用模板
    Apply {
        /// 模板名称
        name: String,
        /// 输入文件
        input: PathBuf,
        /// 输出文件
        output: Option<PathBuf>,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Platform {
    WeChat,
    Zhihu,
    All,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::WeChat => write!(f, "wechat"),
            Platform::Zhihu => write!(f, "zhihu"),
            Platform::All => write!(f, "all"),
        }
    }
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    // 初始化日志
    init_logging(cli.debug)?;

    info!("MarkFlow 启动中...");

    match cli.command {
        Commands::Process {
            input,
            output,
            platform,
            preview,
        } => commands::process_command(input, output, platform, preview).await,
        Commands::Watch {
            directory,
            output,
            pattern,
        } => commands::watch_command(directory, output, pattern).await,
        Commands::Publish {
            content,
            platform,
            draft,
        } => commands::publish_command(content, platform, draft).await,
        Commands::Serve {
            port,
            host,
            static_dir,
        } => commands::serve_command(port, host, static_dir).await,
        Commands::Config { action } => commands::config_command(action).await,
        Commands::Template { action } => commands::template_command(action).await,
    }
}

fn init_logging(debug: bool) -> Result<()> {
    use tracing_subscriber::{fmt, EnvFilter};

    let level = if debug { "debug" } else { "info" };

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("markflow={}", level)));

    fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(debug)
        .with_line_number(debug)
        .init();

    Ok(())
}
