#!/bin/bash

# CSV Tool GUI 设置脚本

echo "🚀 开始设置 CSV Tool GUI..."

# 检查 Node.js
if ! command -v node &> /dev/null; then
    echo "❌ 未找到 Node.js，请先安装 Node.js 18 或更高版本"
    exit 1
fi

NODE_VERSION=$(node -v | cut -d'v' -f2 | cut -d'.' -f1)
if [ "$NODE_VERSION" -lt 18 ]; then
    echo "❌ Node.js 版本过低，需要 18 或更高版本"
    exit 1
fi

echo "✅ Node.js 版本: $(node -v)"

# 检查 Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ 未找到 Rust，请先安装 Rust"
    exit 1
fi

echo "✅ Rust 版本: $(rustc --version)"

# 安装前端依赖
echo "📦 安装前端依赖..."
cd frontend
npm install

if [ $? -ne 0 ]; then
    echo "❌ 前端依赖安装失败"
    exit 1
fi

echo "✅ 前端依赖安装完成"

# 返回根目录
cd ..

echo "✨ 设置完成！"
echo ""
echo "运行开发模式:"
echo "  cd tauri && cargo tauri dev"
echo ""
echo "构建生产版本:"
echo "  cd tauri && cargo tauri build"

