# 搜索和过滤功能实施计划

## 📋 概述

**功能目标**: 实现CSV文件的全文搜索和正则表达式过滤功能

**预计工作量**: 5-7天  
**开始时间**: 2024-12-25

## 🎯 功能需求

### 核心功能
1. **全文搜索** - 在所有列中搜索匹配的文本
2. **正则表达式** - 支持正则表达式高级搜索
3. **列过滤** - 在指定列中搜索
4. **结果高亮** - 高亮显示匹配的文本
5. **分页显示** - 搜索结果分页展示

### CLI 接口设计

```bash
# 基本搜索
csv-tool data.csv search "关键词"

# 正则表达式搜索
csv-tool data.csv search -r "正则表达式"

# 在指定列中搜索
csv-tool data.csv search "关键词" -c name

# 大小写不敏感
csv-tool data.csv search "keyword" -i

# 显示匹配行号
csv-tool data.csv search "keyword" --show-line-numbers

# 只统计匹配数量
csv-tool data.csv search "keyword" --count

# 组合使用
csv-tool data.csv search -r "^[A-Z]" -c name -i
```

## 📦 依赖添加

```toml
[dependencies]
regex = "1.10"  # 正则表达式支持
```

## 🔧 模块设计

### 1. Search 模块结构

```rust
// src/csv/search.rs

/// 搜索选项
pub struct SearchOptions {
    /// 搜索模式（文本或正则）
    pub pattern: SearchPattern,
    /// 目标列（None表示所有列）
    pub columns: Option<Vec<usize>>,
    /// 大小写敏感
    pub case_sensitive: bool,
    /// 最大结果数
    pub max_results: Option<usize>,
}

/// 搜索模式
pub enum SearchPattern {
    /// 纯文本搜索
    Text(String),
    /// 正则表达式搜索
    Regex(Regex),
}

/// 搜索结果
pub struct SearchResult {
    /// 匹配的行号
    pub row_number: usize,
    /// 匹配的列号
    pub col_numbers: Vec<usize>,
    /// 行数据
    pub record: CsvRecord<'static>,
}

/// 搜索引擎
pub struct SearchEngine<'a> {
    reader: &'a CsvReader,
    options: SearchOptions,
}
```

### 2. 主要方法

```rust
impl SearchEngine<'_> {
    /// 执行搜索
    pub fn search(&self) -> Result<Vec<SearchResult>>
    
    /// 流式搜索（大文件优化）
    pub fn search_iter(&self) -> impl Iterator<Item = SearchResult>
    
    /// 统计匹配数量
    pub fn count_matches(&self) -> Result<usize>
}
```

## 📊 实施步骤

### Phase 1: 基础搜索 ✅
- [x] 添加 regex 依赖
- [x] 创建 search.rs 模块
- [x] 实现 SearchOptions 和 SearchPattern
- [x] 实现基本文本搜索

### Phase 2: 正则表达式 ✅
- [x] 集成 regex 库
- [x] 实现正则表达式搜索
- [x] 添加大小写选项

### Phase 3: CLI 集成 ✅
- [x] 添加 search 子命令
- [x] 实现参数解析
- [x] 实现结果显示

### Phase 4: 高级功能 ✅
- [x] 搜索结果高亮
- [x] 列过滤
- [x] 结果限制
- [x] 反向匹配

### Phase 5: 测试 ✅
- [x] 单元测试 (5个)
- [x] 集成测试 (7个)

## ✅ 完成总结

搜索和过滤功能已完成！

### 实现的功能
1. **全文搜索** - 在所有列或指定列中搜索
2. **正则表达式** - 支持复杂模式匹配
3. **大小写选项** - 支持大小写敏感/不敏感搜索
4. **列过滤** - 按列名或列号指定搜索列
5. **结果高亮** - 匹配文本高亮显示
6. **反向匹配** - 显示不匹配的行
7. **结果统计** - 快速统计匹配数量
8. **结果限制** - 限制最大结果数

### 测试覆盖
- `test_text_search` - 文本搜索
- `test_regex_search` - 正则表达式搜索
- `test_search_in_column` - 列过滤
- `test_search_case_insensitive` - 大小写不敏感
- `test_search_invert_match` - 反向匹配
- `test_count_matches` - 统计匹配
- `test_search_max_results` - 结果限制

---

*创建时间: 2024-12-25*
*完成时间: 2024-12-25*

