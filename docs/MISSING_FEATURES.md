# CSV工具缺失功能分析

## 📋 功能缺失概览

基于当前项目状态和README中的路线图，以下是当前缺失的功能清单。

## 🔴 高优先级缺失功能

### 1. 索引持久化 ⚠️ 重要

**当前状态**: ❌ 未实现  
**优先级**: 🔴 高  
**预计工作量**: 2-3天

**功能描述**:
- 将构建的行索引保存到 `.csv.idx` 文件
- 下次打开相同文件时，如果文件未修改，直接加载索引
- 避免每次打开文件都重新构建索引

**实现要点**:
```rust
// 需要实现的功能
pub fn save_index(&self, index_path: &Path) -> Result<()>
pub fn load_index(index_path: &Path) -> Result<RowIndex>
pub fn is_index_valid(csv_path: &Path, index_path: &Path) -> bool
```

**技术细节**:
- 使用 `serde` + `bincode` 序列化索引
- 检查CSV文件的修改时间（mtime）
- 如果CSV文件被修改，索引失效，需要重新构建

**预期收益**:
- 打开已索引的文件从2秒降至<100ms
- 提升用户体验，特别是重复打开同一文件

---

### 2. CLI界面优化 ⚠️ 重要

**当前状态**: ❌ 基础实现，功能有限  
**优先级**: 🔴 高  
**预计工作量**: 3-5天

**当前问题**:
- 命令行参数解析简单（使用 `env::args()`）
- 缺少参数验证和帮助信息
- 不支持可选参数（分隔符、编码等）
- 没有进度条显示

**需要实现的功能**:
```rust
// 使用 clap 改进命令行接口
csv-tool [OPTIONS] <FILE> [PAGE]

Options:
  -d, --delimiter <CHAR>    分隔符（默认: ,）
  -e, --encoding <ENCODING>  文件编码（默认: UTF-8）
  -h, --no-headers          无表头
  -p, --page-size <SIZE>    每页行数（默认: 20）
  -i, --index-granularity   索引粒度（默认: 1000）
  --show-stats              显示统计信息
  --version                 显示版本信息
  --help                    显示帮助信息
```

**实现要点**:
- 集成 `clap` 库进行参数解析
- 添加参数验证和错误提示
- 支持配置文件（可选）
- 添加进度条（使用 `indicatif`）

**预期收益**:
- 更友好的用户体验
- 支持更多使用场景
- 更好的错误提示

---

### 3. CSV写入功能 ❌ 缺失

**当前状态**: ❌ 完全未实现  
**优先级**: 🔴 高（如果要做编辑器）  
**预计工作量**: 5-7天

**功能描述**:
- 支持修改CSV单元格
- 支持添加/删除行
- 支持添加/删除列
- 支持保存修改

**实现挑战**:
- 内存映射文件是只读的，修改需要特殊处理
- 大文件修改需要流式写入
- 需要处理索引更新

**实现方案**:
```rust
pub struct CsvWriter {
    // 方案1: 使用临时文件
    // 方案2: 使用内存缓冲区（小文件）
    // 方案3: 流式写入（大文件）
}

pub fn edit_cell(&mut self, row: usize, col: usize, value: &str) -> Result<()>
pub fn add_row(&mut self, row: Vec<String>) -> Result<()>
pub fn delete_row(&mut self, row: usize) -> Result<()>
pub fn save(&self, path: &Path) -> Result<()>
```

**预期收益**:
- 从查看器升级为编辑器
- 支持数据修改需求

---

## 🟡 中优先级缺失功能

### 4. 异步索引构建

**当前状态**: ❌ 未实现  
**优先级**: 🟡 中  
**预计工作量**: 3-4天

**功能描述**:
- 使用 `tokio` 在后台线程构建索引
- 文件打开立即返回，不阻塞
- 索引构建完成后通知用户

**实现要点**:
```rust
pub async fn open_async<P>(path: P, ...) -> Result<CsvReader>
pub fn is_index_ready(&self) -> bool
pub fn wait_for_index(&self) -> Result<()>
```

**预期收益**:
- 提升用户体验，文件打开更快
- 适合GUI应用

---

### 5. 搜索和过滤功能

**当前状态**: ❌ 未实现  
**优先级**: 🟡 中  
**预计工作量**: 5-7天

**功能描述**:
- 全文搜索（支持正则表达式）
- 列过滤（等于、包含、大于、小于等）
- 多条件组合过滤
- 搜索结果高亮

**实现要点**:
```rust
pub fn search(&self, query: &str, regex: bool) -> Result<Vec<usize>>
pub fn filter(&self, filters: Vec<Filter>) -> Result<Vec<usize>>
pub fn read_filtered_page(&mut self, page: usize, filtered_rows: &[usize]) -> Result<Vec<CsvRecord>>
```

**技术细节**:
- 使用 `regex` 库支持正则表达式
- 需要构建搜索索引（可选）
- 支持增量搜索

**预期收益**:
- 提升数据查找效率
- 支持复杂查询需求

---

### 6. 数据排序功能

**当前状态**: ❌ 未实现  
**优先级**: 🟡 中  
**预计工作量**: 4-5天

**功能描述**:
- 按列排序（升序/降序）
- 多列排序
- 数值/日期/字符串智能识别

**实现要点**:
```rust
pub fn sort_by_column(&mut self, col: usize, ascending: bool) -> Result<()>
pub fn sort_by_columns(&mut self, cols: Vec<(usize, bool)>) -> Result<()>
```

**技术细节**:
- 需要读取所有数据到内存（或使用外部排序）
- 大文件需要流式排序
- 排序后需要更新索引

**预期收益**:
- 方便数据分析和查看

---

### 7. 导出功能

**当前状态**: ❌ 未实现  
**优先级**: 🟡 中  
**预计工作量**: 3-4天

**功能描述**:
- 导出为其他格式（JSON, Excel, Parquet等）
- 导出筛选后的数据
- 导出指定列
- 批量导出

**实现要点**:
```rust
pub fn export_json(&self, path: &Path, filters: Option<&Filters>) -> Result<()>
pub fn export_excel(&self, path: &Path) -> Result<()>
pub fn export_parquet(&self, path: &Path) -> Result<()>
```

**技术细节**:
- JSON: 使用 `serde_json`
- Excel: 使用 `calamine` 或 `xlsxwriter`
- Parquet: 使用 `parquet` crate

**预期收益**:
- 支持数据格式转换
- 方便数据共享

---

## 🟢 低优先级缺失功能

### 8. GUI界面

**当前状态**: ❌ 未实现  
**优先级**: 🟢 低  
**预计工作量**: 2-3周

**功能描述**:
- 使用 `egui` 实现图形界面
- 文件选择对话框
- 表格展示（支持滚动、调整列宽）
- 分页控制
- 搜索框
- 编辑功能

**实现要点**:
```rust
// 需要创建新的GUI模块
src/ui/
├── mod.rs
├── app.rs          # 主应用窗口
├── table.rs        # 表格组件
├── search.rs       # 搜索组件
└── editor.rs       # 编辑组件
```

**技术栈**:
- `eframe` + `egui` - GUI框架
- `rfd` - 文件对话框

**预期收益**:
- 更友好的用户界面
- 适合非技术用户

---

### 9. 并行处理优化

**当前状态**: ❌ 未实现  
**优先级**: 🟢 低  
**预计工作量**: 3-5天

**功能描述**:
- 使用 `rayon` 并行解析多行
- 并行构建索引
- 并行搜索

**实现要点**:
```rust
use rayon::prelude::*;

// 并行解析
lines.par_iter()
    .map(|line| CsvRecord::parse_line(line, delimiter))
    .collect()
```

**预期收益**:
- 在多核CPU上提升性能
- 索引构建速度提升2-4倍

---

### 10. 统计信息功能

**当前状态**: ❌ 未实现  
**优先级**: 🟢 低  
**预计工作量**: 2-3天

**功能描述**:
- 列统计（最小值、最大值、平均值、中位数等）
- 数据类型检测
- 空值统计
- 唯一值统计

**实现要点**:
```rust
pub fn column_stats(&self, col: usize) -> Result<ColumnStats>
pub fn detect_column_types(&self) -> Result<Vec<ColumnType>>
```

**预期收益**:
- 方便数据探索
- 数据质量检查

---

### 11. 编码检测和处理

**当前状态**: ⚠️ 部分支持（仅UTF-8）  
**优先级**: 🟢 低  
**预计工作量**: 2-3天

**功能描述**:
- 自动检测文件编码（GBK, GB2312, Big5等）
- 支持多种编码格式
- 编码转换

**实现要点**:
```rust
use encoding_rs::Encoding;

pub fn detect_encoding(&self) -> Result<&'static Encoding>
pub fn convert_encoding(&self, target: &Encoding) -> Result<()>
```

**技术细节**:
- 使用 `encoding_rs` 库
- 检测BOM标记
- 使用统计方法检测编码

**预期收益**:
- 支持中文CSV文件
- 国际化支持

---

### 12. 性能基准测试

**当前状态**: ⚠️ 框架已创建，但未运行  
**优先级**: 🟢 低  
**预计工作量**: 1-2天

**功能描述**:
- 使用 `criterion` 进行性能基准测试
- 对比不同实现的性能
- 生成性能报告

**当前状态**:
- ✅ `benches/benchmark.rs` 已创建
- ❌ 未实际运行基准测试
- ❌ 未生成性能报告

**需要完成**:
```bash
# 运行基准测试
cargo bench

# 生成HTML报告
# 报告位于 target/criterion/
```

---

## 📊 功能优先级总结

### 高优先级（建议优先实现）
1. ✅ **索引持久化** - 显著提升重复打开文件的速度
2. ✅ **CLI界面优化** - 提升用户体验，支持更多场景
3. ⚠️ **CSV写入功能** - 如果要做编辑器，这是必需的

### 中优先级（根据需求实现）
4. 异步索引构建
5. 搜索和过滤功能
6. 数据排序功能
7. 导出功能

### 低优先级（可选功能）
8. GUI界面
9. 并行处理优化
10. 统计信息功能
11. 编码检测和处理
12. 性能基准测试

## 🎯 推荐实施顺序

### 第一阶段（1-2周）
1. **索引持久化** - 快速提升用户体验
2. **CLI界面优化** - 完善基础功能

### 第二阶段（2-4周）
3. **搜索和过滤** - 核心功能增强
4. **数据排序** - 数据分析支持

### 第三阶段（1-2月）
5. **CSV写入功能** - 升级为编辑器
6. **导出功能** - 数据格式转换

### 第四阶段（长期）
7. **GUI界面** - 图形化界面
8. **其他优化** - 性能、统计等

## 💡 实施建议

### 快速提升用户体验
优先实现：
- ✅ 索引持久化（2-3天）
- ✅ CLI界面优化（3-5天）

这两个功能可以快速提升用户体验，工作量相对较小。

### 功能完整性
如果需要完整的CSV编辑器：
- CSV写入功能（必需）
- 搜索和过滤（重要）
- 数据排序（重要）

### 性能优化
如果需要进一步提升性能：
- 异步索引构建
- 并行处理优化

## 📝 总结

当前项目**核心读取功能已完整**，主要缺失的是：

1. **用户体验优化**（索引持久化、CLI优化）
2. **功能增强**（搜索、排序、导出）
3. **编辑器功能**（写入、编辑）
4. **图形界面**（GUI）

建议根据实际需求，按优先级逐步实现这些功能。

---

*最后更新: 2024*  
*文档版本: 1.0*

