# MarkFlow

[![Build Status](https://github.com/foxzool/markflow/workflows/CI/badge.svg)](https://github.com/foxzool/markflow/actions)
[![Crates.io](https://img.shields.io/crates/v/markflow.svg)](https://crates.io/crates/markflow)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

一个强大的Rust工具，用于将Markdown文档转换为适合微信公众号和知乎平台的HTML格式，并支持自动发布功能。

## ✨ 特性

- 🔄 **智能转换**: 使用comrak解析器，完全支持GitHub Flavored Markdown
- 🎨 **平台优化**: 针对微信公众号和知乎平台的样式适配
- 📱 **移动友好**: 自动优化移动端显示效果
- ⚡ **实时监控**: 文件变化自动处理功能
- 🛠️ **CLI工具**: 功能完整的命令行界面
- 🌐 **Web界面**: 可选的Web管理界面
- 📊 **多平台**: 同时支持微信公众号和知乎发布

## 🚀 快速开始

### 安装

#### 从 Crates.io 安装 (推荐)

```bash
# 安装最新版本
cargo install markflow
```

#### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/foxzool/markflow.git
cd markflow

# 构建项目
cargo build --release

# 安装到系统
cargo install --path .
```

#### 预编译二进制文件

从 [Releases](https://github.com/foxzool/markflow/releases) 页面下载适合您系统的预编译二进制文件。

### 基础使用

```bash
# 处理单个Markdown文件
markflow process -i article.md -p wechat

# 预览转换结果
markflow process -i article.md -p zhihu --preview

# 监控目录变化
markflow watch -d ./content -o ./output

# 初始化配置
markflow config init
```

## 📖 详细使用说明

### 处理文件

```bash
# 转换为微信公众号格式
markflow process -i my-article.md -p wechat -o ./output

# 转换为知乎格式
markflow process -i my-article.md -p zhihu -o ./output

# 同时转换为两种格式
markflow process -i my-article.md -p all -o ./output
```

### 监控模式

```bash
# 监控当前目录的所有Markdown文件
markflow watch -d . 

# 监控指定目录并输出到指定位置
markflow watch -d ./content -o ./dist
```

### 配置管理

```bash
# 查看当前配置
markflow config show

# 设置作者信息
markflow config set general.author "Your Name"

# 设置微信配置
markflow config set wechat.app_id "your_app_id"
markflow config set wechat.app_secret "your_app_secret"

# 设置知乎配置
markflow config set zhihu.username "your_username"
markflow config set zhihu.enable_math true
```

## 📝 Markdown支持

MarkFlow支持完整的GitHub Flavored Markdown语法：

- ✅ 标题 (H1-H6)
- ✅ 段落和换行
- ✅ **加粗** 和 *斜体*
- ✅ 代码块和行内代码
- ✅ 列表（有序和无序）
- ✅ 链接和图片
- ✅ 表格
- ✅ 引用块
- ✅ 分隔线
- ✅ 删除线
- ✅ 任务列表
- ✅ 脚注
- ✅ 数学公式（知乎）

### Front Matter支持

```yaml
---
title: "文章标题"
author: "作者名称"
description: "文章描述"
tags: "Rust, Markdown, 工具"
cover: "https://example.com/cover.jpg"
---

# 文章内容开始...
```

## 🎨 平台特性

### 微信公众号

- 📱 完全内联样式，确保兼容性
- 🔗 外部链接自动转换为脚注
- 📱 移动端优化显示
- 🎨 美观的代码高亮
- 📊 表格和列表优化

### 知乎

- 🧮 支持LaTeX数学公式渲染
- 🎨 代码块语法高亮
- 🏷️ 自动标签管理
- 📱 响应式图片处理
- 📊 表格样式优化

## ⚙️ 配置选项

配置文件位置：`~/.markflow/config.toml`

```toml
[general]
author = "Your Name"
default_platform = "all"
auto_save = true
backup_enabled = true
watch_interval = 2

[wechat]
app_id = "your_app_id"
app_secret = "your_app_secret"
auto_publish = false
draft_mode = true

[zhihu]
username = "your_username"
auto_publish = false
enable_math = true
code_theme = "github"

[output]
output_dir = "./output"
create_subdirs = true
filename_pattern = "{title}_{platform}.html"
backup_dir = "./backup"
```

## 🏗️ 项目结构

```
markflow/
├── src/
│   ├── core/           # 核心处理模块
│   │   ├── processor.rs    # Markdown处理器
│   │   ├── content.rs      # 内容数据结构
│   │   └── pipeline.rs     # 处理流水线
│   ├── adapters/       # 平台适配器
│   │   ├── wechat.rs       # 微信公众号适配
│   │   ├── zhihu.rs        # 知乎适配
│   │   └── traits.rs       # 适配器接口
│   ├── publishers/     # 发布模块
│   ├── cli/           # 命令行接口
│   ├── web/           # Web接口
│   └── lib.rs         # 库入口
├── examples/          # 示例文件
├── tests/            # 测试文件
└── README.md
```

## 🔧 开发指南

### 构建项目

```bash
# 开发模式构建
cargo build

# 发布模式构建
cargo build --release

# 运行测试
cargo test

# 生成文档
cargo doc --open
```

### 添加新的平台适配器

1. 在 `src/adapters/` 下创建新的适配器文件
2. 实现 `PlatformAdapter` trait
3. 在 `src/adapters/mod.rs` 中导出
4. 在CLI中添加相应的命令支持

## 🤝 贡献指南

欢迎贡献代码！请遵循以下流程：

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 📄 许可证

本项目采用 MIT OR Apache-2.0 许可证。详见 [LICENSE](LICENSE) 文件。

## 🆘 支持

如果遇到问题或有建议，请：

- 查看 [Issues](https://github.com/foxzool/markflow/issues)
- 创建新的 Issue
- 参考文档和示例

## 🗺️ 路线图

- [ ] 微信公众号API集成
- [ ] 知乎自动发布功能
- [ ] Web管理界面
- [ ] 模板系统
- [ ] 插件系统
- [ ] 更多平台支持（如：掘金、CSDN等）
- [ ] 图片自动压缩和优化
- [ ] 批量处理功能

---

**MarkFlow** - 让Markdown发布变得简单高效！ 🚀