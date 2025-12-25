# CLI界面优化实施计划

## 📋 概述

**功能目标**: 使用 `clap` 库改进命令行参数解析，添加进度条显示，支持更多配置选项。

**预计工作量**: 3-5天  
**开始时间**: 2024-12-25

## 🎯 实施目标

### 核心功能
1. ✅ 使用 `clap` derive 模式定义命令行参数
2. ✅ 支持更多可配置选项
3. ✅ 添加进度条显示索引构建进度
4. ✅ 改进帮助信息和错误提示
5. ✅ 支持子命令（info, view, export等）

### 命令行接口设计

```bash
csv-tool [OPTIONS] <FILE> [COMMAND]

Commands:
  info    显示文件信息
  view    查看数据（默认）
  help    显示帮助

Arguments:
  <FILE>  CSV文件路径

Options:
  -d, --delimiter <CHAR>      分隔符 [default: ,]
  -p, --page <PAGE>           页码（从1开始）[default: 1]
  -s, --page-size <SIZE>      每页行数 [default: 20]
  -n, --no-headers            文件无表头
  -g, --granularity <N>       索引粒度 [default: 1000]
  -q, --quiet                 安静模式（减少输出）
  -v, --verbose               详细模式
      --no-cache              禁用索引缓存
      --rebuild-index         强制重建索引
  -h, --help                  显示帮助
  -V, --version               显示版本
```

## 📦 依赖添加

```toml
# Cargo.toml
[dependencies]
clap = { version = "4.5", features = ["derive", "color"] }
indicatif = "0.17"  # 进度条
```

## 🔧 实施步骤

### Phase 1: 基础CLI结构 ✅
- [x] 添加依赖
- [ ] 定义 Args 结构体
- [ ] 实现基础参数解析

### Phase 2: 参数实现
- [ ] 分隔符参数
- [ ] 页码和页面大小参数
- [ ] 表头参数
- [ ] 索引相关参数

### Phase 3: 进度条
- [ ] 索引构建进度条
- [ ] 文件加载进度指示

### Phase 4: 子命令
- [ ] info 子命令
- [ ] view 子命令（默认）
- [ ] 更多子命令（可选）

### Phase 5: 测试和文档
- [ ] 命令行参数测试
- [ ] 更新 README
- [ ] 更新帮助信息

## 📝 代码设计

### Args 结构体

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "csv-tool")]
#[command(author, version, about = "高性能CSV文件查看工具")]
#[command(long_about = "
CSV Tool - 高性能CSV文件查看和处理工具

特性:
  ✨ 使用内存映射技术，支持GB级大文件
  ⚡ 稀疏行索引，快速页面跳转
  💾 索引持久化，重复打开更快
  🔄 LRU页面缓存，提升访问性能
")]
pub struct Args {
    /// CSV文件路径
    #[arg(value_name = "FILE")]
    pub file: String,

    /// 分隔符
    #[arg(short, long, default_value = ",")]
    pub delimiter: char,

    /// 页码（从1开始）
    #[arg(short, long, default_value = "1")]
    pub page: usize,

    /// 每页行数
    #[arg(short = 's', long, default_value = "20")]
    pub page_size: usize,

    /// 文件无表头
    #[arg(short = 'n', long)]
    pub no_headers: bool,

    /// 索引粒度（每N行记录一次索引）
    #[arg(short, long, default_value = "1000")]
    pub granularity: usize,

    /// 安静模式
    #[arg(short, long)]
    pub quiet: bool,

    /// 详细模式
    #[arg(short, long)]
    pub verbose: bool,

    /// 禁用索引缓存
    #[arg(long)]
    pub no_cache: bool,

    /// 强制重建索引
    #[arg(long)]
    pub rebuild_index: bool,

    /// 子命令
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 显示文件详细信息
    Info,
    /// 查看数据（默认行为）
    View {
        /// 起始行
        #[arg(long)]
        from_row: Option<usize>,
        /// 结束行
        #[arg(long)]
        to_row: Option<usize>,
    },
}
```

## 🧪 测试计划

1. 参数解析测试
2. 默认值测试
3. 错误参数处理测试
4. 帮助信息显示测试

## 📊 进度跟踪

| 任务 | 状态 | 完成时间 |
|------|------|----------|
| 添加依赖 | ✅ | 2024-12-25 |
| Args 结构体 | ✅ | 2024-12-25 |
| 基础参数解析 | ✅ | 2024-12-25 |
| 分隔符支持 | ✅ | 2024-12-25 |
| 进度条 | ✅ | 2024-12-25 |
| 子命令 (info, view) | ✅ | 2024-12-25 |
| 测试 | ✅ | 2024-12-25 |
| Bug修复（分页逻辑）| ✅ | 2024-12-25 |

## ✅ 完成总结

CLI界面优化已完成！主要改进：

1. **使用 `clap` 库**：专业的命令行参数解析
2. **加载动画**：使用 `indicatif` 显示加载进度
3. **子命令支持**：`info` 和 `view` 命令
4. **丰富的选项**：分隔符、页码、页面大小、安静模式等
5. **美化输出**：表格格式、导航提示、文件信息展示
6. **Bug修复**：修复了分页逻辑中的索引查找问题

---

*创建时间: 2024-12-25*  
*完成时间: 2024-12-25*

