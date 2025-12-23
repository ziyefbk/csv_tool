# CSV Tool GUI - 快速开始指南

## 🚀 快速开始

### 1. 安装依赖

**Windows:**
```bash
setup_gui.bat
```

**Linux/macOS:**
```bash
chmod +x setup_gui.sh
./setup_gui.sh
```

### 2. 运行开发模式

```bash
cd tauri
cargo tauri dev
```

首次运行会下载依赖并编译，可能需要几分钟时间。

### 3. 使用应用

1. 应用启动后，点击"打开CSV文件"按钮
2. 选择要查看的CSV文件
3. 使用分页控件浏览数据
4. 使用搜索框过滤数据

## 📋 系统要求

- **Rust**: 最新稳定版
- **Node.js**: v18 或更高版本
- **系统依赖**:
  - Windows: Microsoft C++ Build Tools
  - macOS: Xcode Command Line Tools
  - Linux: libwebkit2gtk-4.0-dev 等（见 README_GUI.md）

## 🛠️ 故障排除

### 问题：`cargo tauri dev` 失败

**解决方案：**
1. 确保已安装所有系统依赖
2. 检查 Rust 版本：`rustc --version`
3. 检查 Node.js 版本：`node --version`
4. 清理并重新构建：
   ```bash
   cargo clean
   cd ../frontend && rm -rf node_modules && npm install
   ```

### 问题：端口 5173 被占用

**解决方案：**
1. 修改 `frontend/vite.config.ts` 中的端口号
2. 同步修改 `tauri/tauri.conf.json` 中的 `devPath`

### 问题：前端无法连接后端

**解决方案：**
1. 确保前端开发服务器正在运行（`npm run dev`）
2. 检查 `tauri.conf.json` 配置
3. 查看浏览器控制台错误信息

## 📚 更多信息

详细文档请参阅 [README_GUI.md](./README_GUI.md)

