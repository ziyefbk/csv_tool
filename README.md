# CSV Tool

一款使用Rust开发的高性能大型CSV文件查看和编辑工具。

## ✨ 特性

- 🚀 **高性能**: 使用Rust开发，零开销抽象
- 📊 **大文件支持**: 采用分页加载，支持GB级文件  
- 🎨 **现代GUI**: 基于egui的原生跨平台图形界面
- �� **实时编辑**: 快速编辑CSV单元格（开发中）
- 🔍 **搜索过滤**: 强大的数据查询功能（开发中）

## 🛠️ 安装

### Windows

1. 安装 Rust: https://rustup.rs/
2. 克隆项目:
```bash
git clone https://github.com/ziyefbk/csv_tool.git
cd csv_tool
```

3. 编译运行:
```bash
cargo build --release
.\target\release\csv-tool.exe
```

或直接运行:
```bash
cargo run --release
```

### Linux / macOS

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆并运行
git clone https://github.com/ziyefbk/csv_tool.git
cd csv_tool
cargo run --release
```

## 📖 使用方法

1. **启动程序**: 运行后会打开GUI窗口
2. **打开文件**: 点击"📂 打开文件"按钮选择CSV文件
3. **浏览数据**: 使用"⬅ 上一页"/"下一页 ➡"按钮翻页查看
4. **查看信息**: 顶部面板显示文件元数据（行数/列数/大小）

## 🎯 GUI 界面

程序提供完整的图形界面：
- 文件选择对话框
- 数据表格展示（支持滚动）
- 分页浏览控制
- 文件信息面板

## 🏗️ 项目结构

```
csv-tool/
├── src/
│   ├── main.rs              # 应用入口
│   ├── csv_engine/          # CSV处理核心
│   │   ├── mod.rs          # 模块定义
│   │   ├── reader.rs        # 读取器
│   │   ├── writer.rs        # 写入器
│   │   └── preview.rs       # 预览助手
│   └── ui/                  # 用户界面
│       ├── mod.rs          # UI模块
│       └── app.rs           # 主窗口
├── docs/                    # 开发文档
├── examples/                # 示例代码
└── Cargo.toml              # 项目配置
```

## 🔧 技术栈

- **Rust 2021**: 核心语言
- **egui/eframe**: GUI框架
- **csv**: CSV解析
- **polars**: 数据处理
- **tokio**: 异步运行时
- **memmap2**: 内存映射（大文件优化）

## 💡 性能特点

- ✅ 分页加载，不会一次性加载整个文件
- ✅ 支持 GB 级大文件
- ✅ 低内存占用
- ✅ 跨平台支持（Windows/Linux/macOS）

## 📋 开发路线图

- [x] 基础文件读取
- [x] 分页预览
- [x] 元数据显示
- [x] GUI界面
- [ ] 单元格编辑
- [ ] 搜索过滤
- [ ] 数据排序
- [ ] 导出功能

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

MIT License

---

*Built with ❤️ using Rust*
