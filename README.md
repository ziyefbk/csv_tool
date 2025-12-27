# CSV Tool

A high-performance CSV file viewer and processor built with Rust, supporting large files up to GB scale.

## ✨ Features

- 🚀 **High Performance**: Built with Rust, 15-100x performance improvement
- 📊 **Large File Support**: Memory mapping and sparse indexing, supports GB-level files
- ⚡ **Fast Navigation**: O(log n) complexity page jumping, millisecond-level response
- 💾 **Memory Efficient**: Memory mapping and zero-copy technology, 2-4x lower memory usage
- 🔄 **Smart Caching**: LRU page cache with index persistence
- 🎨 **Modern GUI**: Beautiful Tauri + React interface (optional)
- 🎯 **Cross-Platform**: Native support for Windows/Linux/macOS

## 🚀 Quick Start

### Windows

**Build CLI tool:**
```bash
cargo build --release
.\target\release\csv-tool.exe data.csv
```

**Build GUI app:**
```bash
# Setup environment
.\setup_gui_fixed.bat

# Build EXE
.\build.bat

# Run generated EXE
.\tauri\target\release\CSV Tool.exe
```

### Linux / macOS

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and run
git clone https://github.com/ziyefbk/csv_tool.git
cd csv_tool
cargo build --release
./target/release/csv-tool data.csv
```

## 📖 Usage

### CLI Mode

#### Basic Commands

```bash
# View first page (default)
csv-tool data.csv

# View specific page
csv-tool data.csv -p 2

# Custom page size
csv-tool data.csv -p 2 -s 50

# Custom delimiter
csv-tool data.csv -d ';'
```

#### File Information

```bash
# Show file details
csv-tool data.csv info
```

#### Search

```bash
# Basic search
csv-tool data.csv search "keyword"

# Case-insensitive search
csv-tool data.csv search "keyword" -i

# Regex search
csv-tool data.csv search "pattern" -r

# Search in specific column
csv-tool data.csv search "keyword" -c "Column Name"

# Count matches only
csv-tool data.csv search "keyword" --count

# Limit results
csv-tool data.csv search "keyword" -m 100
```

#### Sort

```bash
# Sort by column (ascending)
csv-tool data.csv sort -c "Column Name" --order asc

# Sort by column (descending)
csv-tool data.csv sort -c "Column Name" --order desc

# Auto-detect data type
csv-tool data.csv sort -c "Column Name" --data-type auto

# Case-insensitive sort
csv-tool data.csv sort -c "Column Name" --ignore-case
```

#### Export

```bash
# Export to JSON
csv-tool data.csv export output.json --format json

# Export to CSV
csv-tool data.csv export output.csv --format csv

# Export to TSV
csv-tool data.csv export output.tsv --format tsv

# Export specific columns
csv-tool data.csv export output.json --format json -c "Col1,Col2,Col3"

# Export row range
csv-tool data.csv export output.json --format json --from 10 --to 20
```

#### Edit

```bash
# Edit cell value
csv-tool data.csv edit "set 1 2 NewValue"

# Delete row
csv-tool data.csv edit "delete-row 5"

# Append row
csv-tool data.csv edit "append-row value1,value2,value3"

# Delete column
csv-tool data.csv edit "delete-col ColumnName"

# Rename column
csv-tool data.csv edit "rename-col OldName NewName"
```

#### Create New File

```bash
# Create CSV file with headers
csv-tool create new.csv --headers "Column1,Column2,Column3"

# Create with initial rows
csv-tool create new.csv --headers "Col1,Col2,Col3" --rows "val1,val2,val3"
```

### GUI Mode

1. **Build the application** (see Quick Start above)
2. **Run the EXE**: Double-click `CSV Tool.exe`
3. **Open CSV file**: Click "Open CSV File" button
4. **Browse data**: Use pagination controls to navigate
5. **Search**: Use the search box to filter data in real-time

## 📊 Performance

### Benchmark Results

| File Size | Standard Open | Fast Open | Improvement |
|-----------|--------------|-----------|-------------|
| 10k rows (~1MB) | 3.6 ms | 2.6 ms | 1.4x |
| 100k rows (~10MB) | 23 ms | 19 ms | 1.2x |
| **500k rows (~50MB)** | **96 ms** | **2.5 ms** | **38x** 🚀 |

### Page Reading Performance

| Operation | Time |
|-----------|------|
| Read first page | 37 µs |
| Read middle page | 40 µs |
| Read last page | 63 µs |

### Memory Usage

| File Size | Before | After | Reduction |
|-----------|--------|-------|-----------|
| 1 GB | 1 GB+ | <50 MB | **20x** |

## 🏗️ Project Structure

```
csv-tool/
├── src/                        # Rust core library
│   ├── main.rs                 # CLI entry point
│   ├── lib.rs                  # Library entry
│   ├── error.rs                # Error types
│   └── csv/                    # Core modules
│       ├── reader.rs           # High-performance reader (mmap + index)
│       ├── index.rs            # Sparse row index + sampling
│       ├── cache.rs            # LRU page cache
│       ├── search.rs           # Search functionality
│       ├── sort.rs             # Sort functionality
│       ├── export.rs           # Export functionality
│       ├── writer.rs           # Edit/write functionality
│       └── utils.rs            # Utility functions
│
├── frontend/                   # React frontend
│   └── src/
│       ├── App.tsx
│       ├── components/         # UI components
│       ├── api/                # Tauri API calls
│       └── stores/             # State management
│
├── tauri/                      # Tauri backend
│   └── src/main.rs             # GUI API
│
├── tests/                      # Integration tests (40+ tests)
├── benches/                    # Performance benchmarks
└── docs/                       # Documentation
```

## 🔧 Technology Stack

### Core Dependencies

```toml
memmap2 = "0.9"      # Memory mapping (core optimization)
memchr = "2.7"       # SIMD-accelerated string search
rayon = "1.8"        # Parallel processing
csv = "1.3"          # CSV parsing
lru = "0.12"         # LRU cache
bincode = "1.3"      # Index serialization
regex = "1.10"       # Regular expressions
clap = "4.5"         # CLI argument parsing
thiserror = "1.0"    # Error types
```

### Key Technologies

- **Memory Mapping (mmap)**: OS-level file mapping, on-demand loading
- **Sparse Indexing**: Record byte offset every N rows, O(log n) fast location
- **Zero-Copy Parsing**: Fields directly reference mmap data, reducing allocations
- **Index Persistence**: Auto-save index to `.csv.idx`, 20-40x faster on reopen
- **Fast Open Mode**: Row sampling estimation, progressive indexing, async build support

## 💡 Key Optimizations

### Fast Open Mode (`open_fast`)

For large files, the tool uses smart sampling and progressive indexing:

1. **Row Sampling**: Sample first 1MB to estimate total rows
2. **Progressive Index**: Only index first 2000 rows initially
3. **Async Build**: Background thread continues building full index
4. **Result**: <100ms response time for files of any size!

### Index Persistence

Indexes are automatically saved to `.csv.idx` files:
- Validated against file size and modification time
- Loaded automatically on next open
- 20-40x faster than rebuilding

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run integration tests
cargo test --test integration_test

# Run benchmarks
cargo bench
```

## 📚 Documentation

Detailed documentation in `docs/`:
- [USAGE.md](./docs/USAGE.md) - Complete usage guide
- [PERFORMANCE.md](./docs/PERFORMANCE.md) - Performance analysis
- [TECHNICAL_ASSESSMENT.md](./docs/TECHNICAL_ASSESSMENT.md) - Technical details
- [QUICK_REFERENCE.md](./docs/QUICK_REFERENCE.md) - Quick reference

## 🎯 Features Status

### ✅ Completed

- [x] High-performance CSV reading (mmap + sparse index)
- [x] Fast open mode (sampling + progressive indexing)
- [x] Index persistence (.csv.idx files)
- [x] LRU page cache
- [x] Zero-copy parsing
- [x] Modern GUI (Tauri + React)
- [x] Search (text, regex, column filter)
- [x] Sort (multiple data types)
- [x] Export (JSON, CSV, TSV)
- [x] Edit (cells, rows, columns)
- [x] Create new files
- [x] Comprehensive tests (40+ tests)
- [x] Performance benchmarks

### 🚧 Future Plans

- [ ] Virtual scrolling for very large tables
- [ ] Multi-file tab support
- [ ] Column statistics
- [ ] Data visualization
- [ ] Plugin system

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

MIT License

---

*Built with ❤️ using Rust*
