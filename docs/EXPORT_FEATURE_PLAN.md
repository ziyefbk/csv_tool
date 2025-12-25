# 导出功能实施计划

## 📋 概述

**功能目标**: 支持将CSV数据导出为多种格式

**预计工作量**: 3-4天  
**开始时间**: 2024-12-25

## 🎯 功能需求

### 支持的导出格式
1. **JSON** - 标准JSON数组格式
2. **JSON Lines** - 每行一个JSON对象（适合大文件）
3. **TSV** - 制表符分隔
4. **自定义分隔符CSV** - 支持不同分隔符

### CLI 接口设计

```bash
# 导出为JSON
csv-tool data.csv export output.json

# 导出为JSON Lines格式
csv-tool data.csv export output.jsonl --format jsonl

# 导出指定列
csv-tool data.csv export output.json -c id,name,age

# 导出指定行范围
csv-tool data.csv export output.json --from 100 --to 200

# 导出搜索结果
csv-tool data.csv export output.json --search "Beijing"

# 导出为TSV
csv-tool data.csv export output.tsv --format tsv

# 导出并压缩（可选）
csv-tool data.csv export output.json.gz --compress
```

## 📦 模块设计

### Export 模块结构

```rust
// src/csv/export.rs

/// 导出格式
pub enum ExportFormat {
    Json,           // 标准JSON数组
    JsonLines,      // 每行一个JSON对象
    Csv,            // CSV格式（可自定义分隔符）
    Tsv,            // 制表符分隔
}

/// 导出选项
pub struct ExportOptions {
    pub format: ExportFormat,
    pub columns: Option<Vec<usize>>,    // 导出的列
    pub row_range: Option<(usize, usize)>, // 行范围
    pub search_filter: Option<SearchOptions>, // 搜索筛选
    pub pretty: bool,                   // JSON美化输出
}

/// 导出器
pub struct Exporter<'a> {
    reader: &'a CsvReader,
    options: ExportOptions,
}
```

## 📊 实施步骤

### Phase 1: 基础导出 ✅
- [x] 创建 export.rs 模块
- [x] 实现 ExportFormat 和 ExportOptions
- [x] 实现 JSON 导出

### Phase 2: 更多格式 ✅
- [x] 实现 JSON Lines 导出
- [x] 实现 TSV 导出
- [x] 实现自定义分隔符CSV导出

### Phase 3: 高级功能 ✅
- [x] 列选择导出
- [x] 行范围导出
- [x] 搜索结果导出

### Phase 4: CLI集成 ✅
- [x] 添加 export 子命令
- [x] 实现参数解析
- [x] 进度显示

### Phase 5: 测试 ✅
- [x] 单元测试 (4个)
- [x] 集成测试 (6个)

## ✅ 完成总结

导出功能已完成！

### 已实现的功能
1. **JSON导出** - 标准JSON数组格式，支持美化输出
2. **JSON Lines导出** - 每行一个JSON对象，适合大文件
3. **TSV导出** - 制表符分隔格式
4. **CSV导出** - 自定义分隔符
5. **列选择** - 导出指定列
6. **行范围** - 导出指定行范围
7. **搜索筛选** - 导出搜索匹配的行

### CLI命令示例
```bash
csv-tool data.csv export output.json           # 导出为JSON
csv-tool data.csv export output.jsonl          # 导出为JSON Lines
csv-tool data.csv export output.tsv            # 导出为TSV
csv-tool data.csv export out.json -c id,name   # 导出指定列
csv-tool data.csv export out.json --from 1 --to 100  # 导出指定行
csv-tool data.csv export out.json --search "Beijing" # 导出搜索结果
csv-tool data.csv export out.json --pretty     # JSON美化输出
```

---

*创建时间: 2024-12-25*
*完成时间: 2024-12-25*

