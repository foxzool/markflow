# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

MarkFlow 是一个 Rust CLI 工具，用于将 Markdown 文档转换为适合微信公众号和知乎平台的 HTML 格式，并支持自动发布功能。

## 开发命令

### 构建项目
```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 运行测试
cargo test

# 生成文档
cargo doc --open
```

### 运行 CLI
```bash
# 处理单个 Markdown 文件
cargo run -- process -i article.md -p wechat

# 监控目录变化
cargo run -- watch -d ./content -o ./output

# 查看配置
cargo run -- config show

# 初始化默认配置
cargo run -- config init
```

## 架构设计

### 核心模块

- **`src/core/`** - 核心处理引擎
  - `processor.rs` - 使用 comrak 解析和处理 Markdown
  - `content.rs` - 内容数据结构和元数据处理
  - `pipeline.rs` - 多阶段处理流水线

- **`src/adapters/`** - 平台适配器
  - `traits.rs` - PlatformAdapter trait 定义适配器接口
  - `wechat.rs` - 微信公众号 HTML/CSS 适配
  - `zhihu.rs` - 知乎平台优化，支持数学公式

- **`src/cli/`** - 命令行接口
  - `mod.rs` - 使用 clap 的主 CLI 结构
  - `commands.rs` - 命令实现（process、watch、publish 等）
  - `args.rs` - 配置管理和参数解析

### 关键设计模式

1. **适配器模式**: 通过 PlatformAdapter trait 实现平台特定的 HTML/CSS 转换
2. **管道处理**: 多阶段内容转换和验证
3. **配置驱动**: 基于 TOML 的配置文件，支持运行时覆盖
4. **异步处理**: 基于 Tokio 的异步运行时，用于文件 I/O 和网络操作

### 平台适配

- **微信公众号**: 内联 CSS 样式、外部链接脚注、移动端优化
- **知乎**: LaTeX 数学渲染、语法高亮、响应式图片

### 关键依赖

- `comrak` - GitHub Flavored Markdown 解析
- `tokio` - 异步运行时
- `clap` - CLI 参数解析
- `notify` - 文件系统监控
- `scraper` - HTML 操作
- `thirtyfour` - 浏览器自动化（用于发布）

## 配置文件

配置文件位置：`~/.markflow/config.toml`

配置系统支持：
- 通用设置（作者、默认平台、备份）
- 平台特定设置（微信应用凭据、知乎偏好）
- 输出设置（目录结构、文件名模式）

## 工作流程

1. Markdown → Content 结构体（包含 front matter 解析）
2. Content → HTML（通过 comrak 处理器）
3. HTML → 平台适配的 HTML（通过适配器）
4. 输出 → 文件系统（支持备份）
5. 未来功能：浏览器自动化直接发布