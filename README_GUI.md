# CSV Tool GUI - 现代化图形界面

基于 Tauri + React + TypeScript 构建的现代化 CSV 查看器 GUI 应用。

## ✨ 特性

- 🎨 **现代化UI设计** - 使用 Tailwind CSS 构建的美观界面
- ⚡ **高性能** - 复用 Rust 核心库，保持原有的高性能特性
- 📊 **实时搜索** - 支持在表格中实时搜索过滤
- 📄 **分页浏览** - 支持自定义每页显示行数
- 🎯 **响应式设计** - 适配不同窗口大小
- 🌙 **深色主题** - 护眼的深色界面

## 🛠️ 技术栈

### 前端
- **React 18** - UI 框架
- **TypeScript** - 类型安全
- **Tailwind CSS** - 样式框架
- **Vite** - 构建工具
- **Lucide React** - 图标库

### 后端
- **Tauri 1.5** - 桌面应用框架
- **Rust** - 核心逻辑（复用 csv-tool 库）

## 📦 安装和运行

### 前置要求

1. **Rust** (最新稳定版)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Node.js** (v18 或更高版本)
   ```bash
   # 使用 nvm 安装
   nvm install 18
   nvm use 18
   ```

3. **系统依赖**

   **Windows:**
   - 安装 [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

   **macOS:**
   ```bash
   xcode-select --install
   ```

   **Linux (Ubuntu/Debian):**
   ```bash
   sudo apt update
   sudo apt install libwebkit2gtk-4.0-dev \
       build-essential \
       curl \
       wget \
       libssl-dev \
       libgtk-3-dev \
       libayatana-appindicator3-dev \
       librsvg2-dev
   ```

### 开发模式运行

1. **安装前端依赖**
   ```bash
   cd frontend
   npm install
   ```

2. **运行开发服务器**
   ```bash
   # 在项目根目录
   cd tauri
   cargo tauri dev
   ```

   这将同时启动：
   - Vite 开发服务器 (http://localhost:5173)
   - Tauri 应用窗口

### 构建生产版本

```bash
cd tauri
cargo tauri build
```

构建产物位于 `tauri/target/release/` 目录。

## 📁 项目结构

```
csv-tool/
├── frontend/              # React 前端应用
│   ├── src/
│   │   ├── components/    # React 组件
│   │   │   ├── CSVTable.tsx
│   │   │   ├── FileInfo.tsx
│   │   │   └── Pagination.tsx
│   │   ├── App.tsx        # 主应用组件
│   │   ├── main.tsx       # 入口文件
│   │   └── index.css      # 全局样式
│   ├── package.json
│   └── vite.config.ts
│
├── tauri/                 # Tauri 后端
│   ├── src/
│   │   └── main.rs        # Tauri 应用入口和 API
│   ├── Cargo.toml
│   └── tauri.conf.json    # Tauri 配置
│
└── src/                   # Rust 核心库（复用）
    └── csv/
        └── ...
```

## 🎯 功能说明

### 1. 打开文件
- 点击"打开CSV文件"按钮
- 选择 CSV 文件
- 自动检测表头和分隔符

### 2. 浏览数据
- 使用分页控件切换页面
- 调整每页显示行数（20/50/100/200）
- 支持键盘导航

### 3. 搜索功能
- 在搜索框中输入关键词
- 实时过滤表格数据
- 高亮显示匹配内容

### 4. 文件信息
- 显示文件大小、总行数、总列数
- 显示列名（如果有表头）

## 🔧 开发指南

### 添加新的 Tauri 命令

1. **在 `tauri/src/main.rs` 中添加命令函数：**
   ```rust
   #[tauri::command]
   fn my_command(param: String) -> Result<String, String> {
       // 实现逻辑
       Ok("result".to_string())
   }
   ```

2. **注册命令：**
   ```rust
   .invoke_handler(tauri::generate_handler![
       open_csv_file,
       read_page,
       my_command  // 添加新命令
   ])
   ```

3. **在前端调用：**
   ```typescript
   import { invoke } from "@tauri-apps/api/tauri";
   
   const result = await invoke<string>("my_command", {
     param: "value"
   });
   ```

### 添加新的 React 组件

1. 在 `frontend/src/components/` 中创建组件文件
2. 使用 TypeScript 和 Tailwind CSS
3. 在 `App.tsx` 中导入和使用

## 🐛 故障排除

### 问题：`cargo tauri dev` 失败

**解决方案：**
- 确保已安装所有系统依赖
- 检查 Rust 版本：`rustc --version`
- 检查 Node.js 版本：`node --version`

### 问题：前端无法连接到后端

**解决方案：**
- 确保端口 5173 未被占用
- 检查 `tauri.conf.json` 中的 `devPath` 配置

### 问题：构建失败

**解决方案：**
- 清理构建缓存：`cargo clean`
- 删除 `node_modules` 并重新安装：`rm -rf node_modules && npm install`

## 📝 TODO

- [ ] 支持多文件标签页
- [ ] 添加数据导出功能
- [ ] 支持列排序
- [ ] 添加列过滤功能
- [ ] 支持单元格编辑
- [ ] 添加主题切换（浅色/深色）
- [ ] 性能优化（虚拟滚动）

## 📄 许可证

MIT License

---

*使用 Tauri + React 构建的现代化 CSV 查看器*

