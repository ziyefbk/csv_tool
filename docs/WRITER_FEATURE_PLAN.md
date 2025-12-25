# CSV写入功能实现计划

## 状态：✅ 已完成

完成时间：2024-12-25

## 功能概述

实现CSV数据的修改和保存功能，支持单元格编辑、行列操作、以及创建新文件。

## 实现的功能

### 1. 单元格编辑
- ✅ 修改任意单元格的值
- ✅ 修改表头名称
- ✅ 修改追踪（只保存修改，不全量加载）

### 2. 行操作
- ✅ 删除指定行
- ✅ 追加新行到末尾
- ✅ 在指定位置插入行
- ✅ 批量删除行

### 3. 列操作
- ✅ 删除指定列
- ✅ 重命名列
- ✅ 批量删除列

### 4. 文件操作
- ✅ 保存到新文件
- ✅ 覆盖原文件
- ✅ 创建新CSV文件
- ✅ 流式写入（支持大文件）

### 5. 写入选项
- ✅ 自定义分隔符
- ✅ 行结束符选择（LF/CRLF/CR）
- ✅ 强制引用所有字段
- ✅ 是否写入表头

## 代码结构

### 新增文件
- `src/csv/writer.rs` - 写入核心模块
- `tests/writer_test.rs` - 集成测试

### 修改文件
- `src/csv/mod.rs` - 导出写入模块
- `src/main.rs` - 添加 edit 和 create 子命令

## CLI 使用方法

### 编辑现有文件

```bash
# 修改单元格
csv-tool data.csv edit cell -r 1 -c name -v "新值"
csv-tool data.csv edit cell -r 1 -c 2 -v "新值" -o output.csv

# 删除行
csv-tool data.csv edit delete-row -r 1
csv-tool data.csv edit delete-row -r "1,3,5" -o output.csv

# 添加行
csv-tool data.csv edit add-row -d "值1,值2,值3"
csv-tool data.csv edit add-row -d "值1,值2" -p 3  # 插入到第3行

# 删除列
csv-tool data.csv edit delete-col -c "age"
csv-tool data.csv edit delete-col -c "1,3" -o output.csv

# 重命名列
csv-tool data.csv edit rename-col -c name -n full_name
```

### 创建新文件

```bash
# 创建带数据的新CSV
csv-tool dummy create output.csv -H "id,name,email" \
  -r "1,Alice,alice@example.com" \
  -r "2,Bob,bob@example.com"
```

## API 使用

### CsvEditor - 编辑现有文件

```rust
use csv_tool::csv::{CsvEditor, RowData, WriteOptions};

// 打开文件
let mut editor = CsvEditor::open("data.csv", true, b',', 100)?;

// 编辑单元格
editor.edit_cell(0, 1, "新值".to_string())?;

// 删除行
editor.delete_row(2)?;

// 追加行
let new_row = RowData::new(vec!["Alice".to_string(), "25".to_string()]);
editor.append_row(new_row)?;

// 删除列
editor.delete_col(1)?;

// 重命名列
editor.set_header(0, "full_name".to_string())?;

// 保存
let options = WriteOptions::default();
let stats = editor.save("output.csv", &options)?;

// 或覆盖原文件
editor.save_in_place(&options)?;
```

### CsvCreator - 创建新文件

```rust
use csv_tool::csv::{CsvCreator, RowData, WriteOptions};

// 创建
let headers = vec!["id".to_string(), "name".to_string()];
let mut creator = CsvCreator::new(headers);

// 添加数据
creator.add_row(RowData::new(vec!["1".to_string(), "Alice".to_string()]))?;
creator.add_row(RowData::new(vec!["2".to_string(), "Bob".to_string()]))?;

// 保存
let stats = creator.save("new.csv")?;
```

### WriteOptions - 写入选项

```rust
let options = WriteOptions::new()
    .with_delimiter(b'\t')           // 使用制表符
    .with_line_ending(LineEnding::CrLf)  // Windows换行
    .with_always_quote(true)         // 总是引用
    .with_headers(false);            // 不写表头
```

## 技术特点

### 修改追踪模式
- 不全量加载数据到内存
- 只保存修改记录（HashMap）
- 保存时流式合并原始数据和修改

### 流式写入
- 使用 `BufWriter` 缓冲写入
- 逐行处理，内存占用低
- 支持GB级大文件

### 安全保存
- 先写入临时文件
- 写入完成后原子重命名
- 避免数据丢失

## 测试覆盖

- ✅ 单元格编辑
- ✅ 行删除
- ✅ 行追加
- ✅ 列删除
- ✅ CSV创建
- ✅ 特殊字符转义
- ✅ 修改统计
- ✅ 表头修改
- ✅ 写入选项

## 参数说明

### edit cell
| 参数 | 说明 |
|------|------|
| `-r, --row` | 行号（从1开始） |
| `-c, --col` | 列名或列号 |
| `-v, --value` | 新值 |
| `-o, --output` | 输出文件（可选） |

### edit delete-row
| 参数 | 说明 |
|------|------|
| `-r, --rows` | 行号列表（逗号分隔） |
| `-o, --output` | 输出文件（可选） |

### edit add-row
| 参数 | 说明 |
|------|------|
| `-d, --data` | 行数据（逗号分隔） |
| `-p, --position` | 插入位置（可选，默认追加） |
| `-o, --output` | 输出文件（可选） |

### edit delete-col
| 参数 | 说明 |
|------|------|
| `-c, --cols` | 列名或列号列表 |
| `-o, --output` | 输出文件（可选） |

### edit rename-col
| 参数 | 说明 |
|------|------|
| `-c, --col` | 原列名或列号 |
| `-n, --name` | 新列名 |
| `-o, --output` | 输出文件（可选） |

### create
| 参数 | 说明 |
|------|------|
| `output` | 输出文件路径 |
| `-H, --headers` | 表头列表（逗号分隔） |
| `-r, --row` | 数据行（可多次使用） |

