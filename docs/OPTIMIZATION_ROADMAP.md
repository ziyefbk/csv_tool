# CSV Tool - 进一步优化路线图

## 📊 当前性能状态

### 已实现的优化 ✅
- ✅ 内存映射 (mmap) - 支持 GB 级文件
- ✅ 稀疏索引 - O(log n) 快速定位
- ✅ 快速打开模式 - 采样估算 + 渐进式索引
- ✅ 索引持久化 - 20-40x 再次打开速度
- ✅ LRU 缓存 - 重复访问优化
- ✅ 零拷贝解析 - 减少内存分配
- ✅ 并行索引构建 - 大文件多线程处理

### 当前性能指标
- 打开 500k 行文件：2.5ms (fast mode)
- 页面读取：37-63 µs
- 内存占用：<50MB (1GB 文件)

---

## 🚀 性能优化方向

### 1. 更激进的快速打开优化

**当前状态**：
- 采样大小：1MB
- 初始索引行数：2000 行

**优化建议**：

```rust
// 优化 1: 减少采样大小到 256KB
const SAMPLE_SIZE: usize = 256 * 1024;  // 从 1MB 降到 256KB

// 优化 2: 减少初始索引行数到 500 行
const INITIAL_ROWS: usize = 500;  // 从 2000 降到 500

// 优化 3: 智能采样策略
// - 小文件 (<10MB): 完整索引
// - 中文件 (10-100MB): 采样 256KB
// - 大文件 (>100MB): 采样 128KB
```

**预期效果**：
- 打开时间：2.5ms → **<1ms**
- 内存占用：进一步降低

### 2. 搜索性能优化

**当前状态**：
- 文本搜索：逐行扫描
- 正则搜索：使用 `regex` crate

**优化建议**：

```rust
// 优化 1: 使用 SIMD 加速文本搜索
use memchr::memmem;  // SIMD 加速的字符串搜索

// 优化 2: 构建搜索索引（倒排索引）
// 对于频繁搜索的场景，可以预先构建索引
pub struct SearchIndex {
    // 列索引：列名 -> 行号列表
    column_index: HashMap<String, Vec<usize>>,
    // 全文索引：关键词 -> 行号列表
    fulltext_index: HashMap<String, Vec<usize>>,
}

// 优化 3: 并行搜索
use rayon::prelude::*;

pub fn search_parallel(&self, options: &SearchOptions) -> Result<Vec<SearchResult>> {
    // 将文件分块，并行搜索
    let chunks: Vec<_> = self.split_into_chunks(num_cpus::get());
    chunks.par_iter()
        .flat_map(|chunk| self.search_chunk(chunk, options))
        .collect()
}
```

**预期效果**：
- 搜索速度：提升 **5-10x**
- 大文件搜索：从秒级降到毫秒级

### 3. 排序性能优化

**当前状态**：
- 单线程排序
- 需要加载所有数据到内存

**优化建议**：

```rust
// 优化 1: 外部排序（External Sort）
// 对于超大文件，使用临时文件进行排序
pub fn sort_external(
    &self,
    options: &SortOptions,
    temp_dir: &Path,
) -> Result<Vec<SortedRecord>> {
    // 1. 分块读取并排序
    // 2. 写入临时文件
    // 3. 多路归并
}

// 优化 2: 并行排序
use rayon::prelude::*;

pub fn sort_parallel(&self, options: &SortOptions) -> Result<Vec<SortedRecord>> {
    // 分块并行排序，然后归并
    let chunks = self.split_into_chunks(num_cpus::get());
    let sorted_chunks: Vec<_> = chunks
        .par_iter()
        .map(|chunk| self.sort_chunk(chunk, options))
        .collect();
    self.merge_sorted_chunks(sorted_chunks)
}

// 优化 3: 流式排序（只排序前 N 行）
pub fn sort_top_n(
    &self,
    options: &SortOptions,
    limit: usize,
) -> Result<Vec<SortedRecord>> {
    // 使用堆排序，只保留 top N
}
```

**预期效果**：
- 排序速度：提升 **3-5x**（多核 CPU）
- 内存占用：降低（外部排序）

### 4. 内存映射预热

**当前状态**：
- mmap 按需加载

**优化建议**：

```rust
// 优化：预热 mmap（后台预读取前几 MB）
pub fn warmup_mmap(&self, size: usize) {
    // 后台线程预读取文件的前 N MB
    // 使用 madvise(MADV_WILLNEED) 提示操作系统
    #[cfg(unix)]
    unsafe {
        libc::madvise(
            self.mmap.as_ptr() as *mut _,
            size.min(self.mmap.len()),
            libc::MADV_WILLNEED,
        );
    }
}
```

**预期效果**：
- 首次页面读取：从 40µs → **<20µs**

### 5. 索引压缩

**当前状态**：
- 索引使用 bincode 序列化

**优化建议**：

```rust
// 优化：使用压缩算法压缩索引
use flate2::Compression;
use flate2::write::GzEncoder;

pub fn save_compressed(&self, path: &Path) -> Result<()> {
    // 使用 gzip 压缩索引
    // 索引大小：减少 50-70%
}
```

**预期效果**：
- 索引文件大小：减少 **50-70%**
- 加载速度：可能略有提升（I/O 减少）

---

## 🎨 用户体验优化

### 1. GUI 虚拟滚动

**当前状态**：
- 渲染所有可见行

**优化建议**：

```typescript
// 使用 react-window 或 react-virtualized
import { FixedSizeList } from 'react-window';

function VirtualizedTable({ rows, headers }) {
  return (
    <FixedSizeList
      height={600}
      itemCount={rows.length}
      itemSize={35}
      width="100%"
    >
      {({ index, style }) => (
        <div style={style}>
          <TableRow row={rows[index]} />
        </div>
      )}
    </FixedSizeList>
  );
}
```

**预期效果**：
- 支持显示 **百万行** 而不卡顿
- 内存占用：恒定（只渲染可见行）

### 2. 多文件标签页

**优化建议**：

```typescript
// 标签页管理
interface TabManager {
  tabs: Tab[];
  activeTab: number;
  openFile(path: string): void;
  closeTab(index: number): void;
  switchTab(index: number): void;
}
```

**预期效果**：
- 同时打开多个 CSV 文件
- 快速切换和对比

### 3. 列统计信息

**优化建议**：

```rust
pub struct ColumnStats {
    pub name: String,
    pub data_type: DataType,
    pub null_count: usize,
    pub unique_count: usize,
    pub min: Option<String>,
    pub max: Option<String>,
    pub mean: Option<f64>,
    pub median: Option<f64>,
}

pub fn analyze_columns(&self) -> Result<Vec<ColumnStats>> {
    // 并行分析各列
    // 支持数值、日期、文本等类型
}
```

**预期效果**：
- 快速了解数据质量
- 发现异常值

### 4. 数据可视化

**优化建议**：

```typescript
// 集成图表库（如 Chart.js 或 Recharts）
import { LineChart, BarChart } from 'recharts';

// 支持：
// - 数值列的直方图
// - 时间序列图
// - 相关性热力图
```

**预期效果**：
- 直观理解数据分布
- 发现数据模式

---

## 🔧 代码质量优化

### 1. 性能分析工具

**优化建议**：

```rust
// 添加性能分析宏
#[cfg(feature = "profiling")]
macro_rules! profile {
    ($name:expr, $block:block) => {
        let start = std::time::Instant::now();
        let result = $block;
        eprintln!("[PROFILE] {}: {:?}", $name, start.elapsed());
        result
    };
}

// 使用示例
let rows = profile!("read_page", {
    reader.read_page(page, page_size)?
});
```

### 2. 更完善的错误处理

**优化建议**：

```rust
// 添加错误恢复机制
pub enum CsvError {
    // ... existing errors
    /// 索引损坏，但可以重建
    IndexCorrupted {
        path: PathBuf,
        reason: String,
    },
    /// 部分数据损坏，但可以继续
    PartialDataCorruption {
        row: usize,
        reason: String,
    },
}

// 自动恢复
impl CsvReader {
    pub fn open_with_recovery(path: &Path) -> Result<Self> {
        match Self::open(path, ...) {
            Ok(reader) => Ok(reader),
            Err(CsvError::IndexCorrupted { .. }) => {
                // 自动重建索引
                Self::rebuild_index(path)?;
                Self::open(path, ...)
            }
            Err(e) => Err(e),
        }
    }
}
```

### 3. 更多测试覆盖

**优化建议**：

```rust
// 压力测试
#[test]
fn test_large_file_10gb() {
    // 测试 10GB 文件处理
}

// 并发测试
#[test]
fn test_concurrent_reads() {
    // 测试多线程同时读取
}

// 边界条件测试
#[test]
fn test_malformed_csv() {
    // 测试各种格式错误的 CSV
}
```

---

## 📦 功能扩展

### 1. 数据验证

**优化建议**：

```rust
pub struct ValidationRule {
    pub column: String,
    pub rule_type: RuleType,
    pub message: String,
}

pub enum RuleType {
    Required,
    MinLength(usize),
    MaxLength(usize),
    Range(f64, f64),
    Regex(String),
    Custom(Box<dyn Fn(&str) -> bool>),
}

pub fn validate(&self, rules: &[ValidationRule]) -> Vec<ValidationError> {
    // 并行验证所有规则
}
```

### 2. 数据转换

**优化建议**：

```rust
pub enum Transform {
    // 列操作
    RenameColumn { from: String, to: String },
    AddColumn { name: String, formula: String },
    DeleteColumn(String),
    
    // 数据转换
    ConvertType { column: String, to: DataType },
    FormatDate { column: String, format: String },
    Round { column: String, decimals: usize },
    
    // 行操作
    Filter { condition: String },
    Sort { column: String, order: SortOrder },
}
```

### 3. 插件系统

**优化建议**：

```rust
// 插件接口
pub trait CsvPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn process(&self, data: &mut Vec<CsvRecord>) -> Result<()>;
}

// 插件管理器
pub struct PluginManager {
    plugins: Vec<Box<dyn CsvPlugin>>,
}

// 支持动态加载插件（使用 libloading）
```

---

## 📈 优化优先级

### 高优先级（立即实施）

1. **更激进的快速打开** ⭐⭐⭐
   - 影响：用户体验
   - 难度：低
   - 时间：1-2 天

2. **GUI 虚拟滚动** ⭐⭐⭐
   - 影响：大文件浏览体验
   - 难度：中
   - 时间：2-3 天

3. **搜索性能优化** ⭐⭐
   - 影响：搜索功能
   - 难度：中
   - 时间：3-5 天

### 中优先级（近期实施）

4. **排序性能优化** ⭐⭐
   - 影响：排序功能
   - 难度：中高
   - 时间：5-7 天

5. **多文件标签页** ⭐⭐
   - 影响：多任务处理
   - 难度：中
   - 时间：3-4 天

6. **列统计信息** ⭐
   - 影响：数据分析
   - 难度：低
   - 时间：2-3 天

### 低优先级（长期规划）

7. **数据可视化** ⭐
   - 影响：数据分析
   - 难度：高
   - 时间：1-2 周

8. **插件系统** ⭐
   - 影响：扩展性
   - 难度：高
   - 时间：2-3 周

---

## 🎯 预期总体效果

实施高优先级优化后：

| 指标 | 当前 | 优化后 | 提升 |
|------|------|--------|------|
| 打开时间 | 2.5ms | <1ms | **2.5x** |
| 搜索速度 | 秒级 | 毫秒级 | **10x** |
| 排序速度 | 秒级 | 毫秒级 | **5x** |
| GUI 支持行数 | 10k | 1M+ | **100x** |
| 内存占用 | <50MB | <30MB | **1.7x** |

---

**最后更新**: 2025-12-27  
**状态**: 规划中

