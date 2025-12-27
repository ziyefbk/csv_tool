# CSV Tool 使用说明

## 快速开始

### 基本查看

```bash
# 查看第1页（默认）
csv-tool data.csv

# 查看第2页
csv-tool data.csv -p 2

# 查看第5页，每页50行
csv-tool data.csv -p 5 -s 50
```

### 文件信息

```bash
# 显示文件详细信息（行数、列数、大小等）
csv-tool data.csv info
```

## 搜索功能

### 文本搜索

```bash
# 基本搜索
csv-tool data.csv search "关键词"

# 大小写不敏感搜索
csv-tool data.csv search "关键词" -i

# 显示行号
csv-tool data.csv search "关键词" -l

# 只统计匹配数量
csv-tool data.csv search "关键词" --count

# 限制结果数量
csv-tool data.csv search "关键词" -m 100
```

### 正则表达式搜索

```bash
# 使用正则表达式
csv-tool data.csv search "正则.*表达式" -r

# 在指定列中搜索
csv-tool data.csv search "关键词" -c "列名"
csv-tool data.csv search "关键词" -c "1"  # 使用列号（从1开始）

# 反向匹配（显示不匹配的行）
csv-tool data.csv search "关键词" -V
```

## 导出功能

### 导出为JSON

```bash
# 导出为JSON格式
csv-tool data.csv export output.json --format json

# 美化JSON输出
csv-tool data.csv export output.json --format json --pretty

# 导出为JSONL（每行一个JSON对象）
csv-tool data.csv export output.jsonl --format jsonl
```

### 导出为CSV/TSV

```bash
# 导出为CSV
csv-tool data.csv export output.csv --format csv

# 导出为TSV（制表符分隔）
csv-tool data.csv export output.tsv --format tsv
```

### 导出指定列

```bash
# 导出指定列（使用列名）
csv-tool data.csv export output.json --format json -c "列1,列2,列3"

# 导出指定列（使用列号）
csv-tool data.csv export output.json --format json -c "1,2,3"
```

### 导出指定行范围

```bash
# 导出第10-20行
csv-tool data.csv export output.json --format json --from 10 --to 20
```

### 导出搜索结果

```bash
# 只导出匹配搜索条件的行
csv-tool data.csv export output.json --format json --search "关键词"
```

## 排序功能

### 基本排序

```bash
# 按列升序排序
csv-tool data.csv sort -c "列名" --order asc

# 按列降序排序
csv-tool data.csv sort -c "列名" --order desc

# 使用列号排序
csv-tool data.csv sort -c "1" --order asc
```

### 数据类型

```bash
# 自动检测数据类型
csv-tool data.csv sort -c "列名" --data-type auto

# 强制按字符串排序
csv-tool data.csv sort -c "列名" --data-type string

# 强制按数字排序
csv-tool data.csv sort -c "列名" --data-type number
```

### 高级选项

```bash
# 大小写不敏感排序
csv-tool data.csv sort -c "列名" --ignore-case

# 空值排在前面
csv-tool data.csv sort -c "列名" --nulls-first

# 限制结果数量
csv-tool data.csv sort -c "列名" --limit 100

# 显示行号
csv-tool data.csv sort -c "列名" --line-numbers

# 保存到文件
csv-tool data.csv sort -c "列名" --order asc -o sorted.csv
```

## 编辑功能

### 编辑单元格

```bash
# 设置第1行第2列的值
csv-tool data.csv edit "set 1 2 新值"

# 设置第5行"列名"的值
csv-tool data.csv edit "set 5 列名 新值"
```

### 行操作

```bash
# 删除第5行
csv-tool data.csv edit "delete-row 5"

# 在第3行后插入新行
csv-tool data.csv edit "insert-row 3 值1,值2,值3"

# 在末尾添加新行
csv-tool data.csv edit "append-row 值1,值2,值3"
```

### 列操作

```bash
# 删除列（使用列名）
csv-tool data.csv edit "delete-col 列名"

# 删除列（使用列号）
csv-tool data.csv edit "delete-col 1"

# 重命名列
csv-tool data.csv edit "rename-col 旧列名 新列名"
```

### 保存选项

```bash
# 默认保存到新文件（原文件.csv -> 原文件_edited.csv）
csv-tool data.csv edit "set 1 2 新值"

# 保存到指定文件
csv-tool data.csv edit "set 1 2 新值" -o output.csv

# 覆盖原文件（谨慎使用）
csv-tool data.csv edit "set 1 2 新值" --overwrite
```

## 创建新文件

```bash
# 创建带表头的CSV文件
csv-tool create new.csv --headers "列1,列2,列3"

# 创建并添加初始数据行
csv-tool create new.csv --headers "列1,列2,列3" --rows "值1,值2,值3" "值4,值5,值6"

# 使用分号作为分隔符
csv-tool create new.csv --headers "列1;列2;列3" -d ';'
```

## 高级选项

### 分隔符

```bash
# 使用分号作为分隔符
csv-tool data.csv -d ';'

# 使用制表符作为分隔符
csv-tool data.csv -d $'\t'  # Linux/macOS
csv-tool data.csv -d "`t"   # PowerShell
```

### 索引选项

```bash
# 指定索引粒度（每N行记录一次索引点）
csv-tool data.csv -g 500

# 强制重建索引
csv-tool data.csv --rebuild-index
```

### 输出选项

```bash
# 安静模式（减少输出信息）
csv-tool data.csv -q

# 详细模式（显示更多信息）
csv-tool data.csv -v
```

## Windows PowerShell 特殊说明

### 中文路径问题

如果文件路径包含中文，建议：

1. **使用引号包裹路径**：
```powershell
cargo run --release -- "E:\路径\文件.csv" -p 2
```

2. **先切换到文件所在目录**：
```powershell
cd "E:\路径"
cargo run --release -- 文件.csv -p 2
```

3. **使用短路径名**（如果可用）：
```powershell
# 获取短路径名
cmd /c for %I in ("E:\长路径名\文件.csv") do @echo %~sI
```

### 特殊字符转义

在 PowerShell 中，某些特殊字符需要转义：

```powershell
# 单引号内的内容会被原样处理
csv-tool data.csv -d ';'

# 双引号内需要转义
csv-tool data.csv search "包含\"引号\"的文本"
```

## 完整示例

```bash
# 1. 查看大文件的基本信息
csv-tool large_file.csv info

# 2. 搜索包含"error"的行
csv-tool large_file.csv search "error" -i -l

# 3. 导出前1000行到JSON
csv-tool large_file.csv export output.json --format json --from 1 --to 1000

# 4. 按时间列降序排序，只显示前100条
csv-tool large_file.csv sort -c "时间" --order desc --data-type auto --limit 100

# 5. 编辑文件：修改第10行第3列的值
csv-tool large_file.csv edit "set 10 3 新值" -o edited.csv

# 6. 创建新文件并添加数据
csv-tool create new.csv --headers "姓名,年龄,城市" --rows "张三,25,北京" "李四,30,上海"
```

## 常见问题

### Q: 如何查看帮助信息？

```bash
csv-tool --help
csv-tool search --help
csv-tool export --help
```

### Q: 索引文件在哪里？

索引文件保存在CSV文件同目录下，文件名为 `原文件名.csv.idx`。

### Q: 如何删除索引文件？

直接删除 `.idx` 文件即可，程序会在下次打开时自动重建。

### Q: 支持哪些文件编码？

目前支持 UTF-8 编码的CSV文件。如果文件包含 BOM 标记，程序会自动处理。

### Q: 内存占用如何？

程序使用内存映射技术，不会将整个文件加载到内存。内存占用主要取决于：
- 页面大小（默认20行）
- 缓存大小（默认10页）
- 索引大小（取决于文件大小和索引粒度）

对于GB级文件，内存占用通常小于100MB。

