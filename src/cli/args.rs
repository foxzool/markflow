use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub wechat: WeChatConfig,
    pub zhihu: ZhihuConfig,
    pub templates: TemplateConfig,
    pub output: OutputConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub author: Option<String>,
    pub default_platform: Option<String>,
    pub auto_save: bool,
    pub backup_enabled: bool,
    pub watch_interval: u64, // 秒
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatConfig {
    pub app_id: Option<String>,
    pub app_secret: Option<String>,
    pub access_token: Option<String>,
    pub default_thumb_media_id: Option<String>,
    pub auto_publish: bool,
    pub draft_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZhihuConfig {
    pub username: Option<String>,
    pub cookies_file: Option<PathBuf>,
    pub auto_publish: bool,
    pub default_column: Option<String>,
    pub enable_math: bool,
    pub code_theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub templates_dir: PathBuf,
    pub default_template: Option<String>,
    pub custom_templates: HashMap<String, PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub output_dir: PathBuf,
    pub create_subdirs: bool, // 是否为每个平台创建子目录
    pub filename_pattern: String, // 文件名模式，如 "{title}_{platform}.html"
    pub backup_dir: Option<PathBuf>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            wechat: WeChatConfig::default(),
            zhihu: ZhihuConfig::default(),
            templates: TemplateConfig::default(),
            output: OutputConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            author: None,
            default_platform: Some("all".to_string()),
            auto_save: true,
            backup_enabled: true,
            watch_interval: 2,
        }
    }
}

impl Default for WeChatConfig {
    fn default() -> Self {
        Self {
            app_id: None,
            app_secret: None,
            access_token: None,
            default_thumb_media_id: None,
            auto_publish: false,
            draft_mode: true,
        }
    }
}

impl Default for ZhihuConfig {
    fn default() -> Self {
        Self {
            username: None,
            cookies_file: None,
            auto_publish: false,
            default_column: None,
            enable_math: true,
            code_theme: "github".to_string(),
        }
    }
}

impl Default for TemplateConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            templates_dir: home_dir.join(".markflow").join("templates"),
            default_template: None,
            custom_templates: HashMap::new(),
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("./output"),
            create_subdirs: true,
            filename_pattern: "{title}_{platform}.html".to_string(),
            backup_dir: Some(PathBuf::from("./backup")),
        }
    }
}

impl AppConfig {
    pub fn load_from_file(path: &PathBuf) -> crate::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        
        let content = std::fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)
            .map_err(|e| crate::error::Error::Config(format!("配置文件解析失败: {}", e)))?;
        
        Ok(config)
    }
    
    pub fn save_to_file(&self, path: &PathBuf) -> crate::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)
            .map_err(|e| crate::error::Error::Config(format!("配置序列化失败: {}", e)))?;
        
        std::fs::write(path, content)?;
        Ok(())
    }
    
    pub fn get_config_path() -> PathBuf {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home_dir.join(".markflow").join("config.toml")
    }
    
    pub fn set_value(&mut self, key: &str, value: &str) -> crate::Result<()> {
        match key {
            "general.author" => self.general.author = Some(value.to_string()),
            "general.default_platform" => self.general.default_platform = Some(value.to_string()),
            "general.auto_save" => self.general.auto_save = value.parse().unwrap_or(true),
            "general.backup_enabled" => self.general.backup_enabled = value.parse().unwrap_or(true),
            "general.watch_interval" => self.general.watch_interval = value.parse().unwrap_or(2),
            
            "wechat.app_id" => self.wechat.app_id = Some(value.to_string()),
            "wechat.app_secret" => self.wechat.app_secret = Some(value.to_string()),
            "wechat.auto_publish" => self.wechat.auto_publish = value.parse().unwrap_or(false),
            "wechat.draft_mode" => self.wechat.draft_mode = value.parse().unwrap_or(true),
            
            "zhihu.username" => self.zhihu.username = Some(value.to_string()),
            "zhihu.auto_publish" => self.zhihu.auto_publish = value.parse().unwrap_or(false),
            "zhihu.enable_math" => self.zhihu.enable_math = value.parse().unwrap_or(true),
            "zhihu.code_theme" => self.zhihu.code_theme = value.to_string(),
            
            "output.output_dir" => self.output.output_dir = PathBuf::from(value),
            "output.create_subdirs" => self.output.create_subdirs = value.parse().unwrap_or(true),
            "output.filename_pattern" => self.output.filename_pattern = value.to_string(),
            
            _ => return Err(crate::error::Error::Config(format!("未知的配置键: {}", key))),
        }
        Ok(())
    }
    
    pub fn get_value(&self, key: &str) -> Option<String> {
        match key {
            "general.author" => self.general.author.clone(),
            "general.default_platform" => self.general.default_platform.clone(),
            "general.auto_save" => Some(self.general.auto_save.to_string()),
            "general.backup_enabled" => Some(self.general.backup_enabled.to_string()),
            "general.watch_interval" => Some(self.general.watch_interval.to_string()),
            
            "wechat.app_id" => self.wechat.app_id.clone(),
            "wechat.app_secret" => self.wechat.app_secret.clone(),
            "wechat.auto_publish" => Some(self.wechat.auto_publish.to_string()),
            "wechat.draft_mode" => Some(self.wechat.draft_mode.to_string()),
            
            "zhihu.username" => self.zhihu.username.clone(),
            "zhihu.auto_publish" => Some(self.zhihu.auto_publish.to_string()),
            "zhihu.enable_math" => Some(self.zhihu.enable_math.to_string()),
            "zhihu.code_theme" => Some(self.zhihu.code_theme.clone()),
            
            "output.output_dir" => Some(self.output.output_dir.display().to_string()),
            "output.create_subdirs" => Some(self.output.create_subdirs.to_string()),
            "output.filename_pattern" => Some(self.output.filename_pattern.clone()),
            
            _ => None,
        }
    }
}