# 排序功能实现计划

## 状态：✅ 已完成

完成时间：2024-12-25

## 功能概述

实现CSV数据按列排序功能，支持升序/降序排序，支持多种数据类型。

## 实现的功能

### 1. 排序类型
- ✅ 升序排序 (asc)
- ✅ 降序排序 (desc)

### 2. 数据类型
- ✅ 字符串排序（字典序）
- ✅ 数字排序（数值比较）
- ✅ 自动类型检测

### 3. 排序选项
- ✅ 大小写敏感/不敏感
- ✅ 空值位置控制（排前/排后）
- ✅ 结果数量限制
- ✅ 显示原始行号
- ✅ 导出排序结果到文件

## 代码结构

### 新增文件
- `src/csv/sort.rs` - 排序核心模块
- `tests/sort_test.rs` - 排序功能测试

### 修改文件
- `src/csv/mod.rs` - 导出排序模块
- `src/main.rs` - 添加 sort 子命令

## CLI 使用方法

```bash
# 基本用法
csv-tool <FILE> sort <COLUMN>

# 按name列升序排序
csv-tool data.csv sort name

# 按age列降序排序
csv-tool data.csv sort age --order desc

# 指定数据类型为数字
csv-tool data.csv sort score -t number

# 大小写不敏感排序
csv-tool data.csv sort name -i

# 限制显示前10条
csv-tool data.csv sort score --order desc -n 10

# 显示原始行号
csv-tool data.csv sort score -l

# 导出排序结果到文件
csv-tool data.csv sort score --order desc -o sorted.csv
```

## 参数说明

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `COLUMN` | 排序列（列名或列号） | 必需 |
| `--order` | 排序方向 (asc/desc) | asc |
| `-t, --data-type` | 数据类型 (auto/string/number) | auto |
| `-n, --limit` | 结果数量限制 | 无 |
| `-i, --ignore-case` | 大小写不敏感 | false |
| `--nulls-first` | 空值排在最前 | false |
| `-l, --line-numbers` | 显示行号 | false |
| `-o, --output` | 导出文件路径 | 无 |

## API 使用

```rust
use csv_tool::csv::{
    CsvReader, SortOrder, SortKey, SortOptions, DataType, sort_csv_data
};

let reader = CsvReader::open("data.csv", true, b',', 100)?;

// 创建排序键
let key = SortKey::new(0, SortOrder::Ascending, DataType::Auto);

// 创建排序选项
let options = SortOptions::new()
    .add_key(key)
    .with_case_sensitive(false)
    .with_nulls_last(true);

// 执行排序
let sorted = sort_csv_data(&reader, &options, Some(10))?;

for record in sorted {
    println!("Row {}: {:?}", record.original_row, record.record.fields);
}
```

## 测试覆盖

- ✅ 字符串升序排序
- ✅ 字符串降序排序
- ✅ 数字升序排序
- ✅ 数字降序排序
- ✅ 自动类型检测
- ✅ 大小写不敏感
- ✅ 结果限制
- ✅ 原始行号保留
- ✅ 空值处理

