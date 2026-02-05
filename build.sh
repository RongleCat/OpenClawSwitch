#!/bin/bash
set -e

echo "========================================"
echo "OpenClaw Manager - 构建脚本"
echo "========================================"
echo ""

# 检查 Node.js
if ! command -v node &> /dev/null; then
    echo "[错误] 未检测到 Node.js,请先安装 Node.js 18+"
    echo "下载地址: https://nodejs.org/"
    exit 1
fi

# 检查 Rust
if ! command -v cargo &> /dev/null; then
    echo "[错误] 未检测到 Rust,请先安装 Rust"
    echo "下载地址: https://rustup.rs/"
    exit 1
fi

echo "[1/4] 检查环境..."
node --version
cargo --version

echo ""
echo "[2/4] 安装 npm 依赖..."
npm install

echo ""
echo "[3/4] 构建应用..."
echo "提示: 首次构建可能需要较长时间 (10-20分钟)"
npm run tauri build

echo ""
echo "[4/4] 构建完成!"
echo ""
echo "========================================"
echo "可执行文件位置:"
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "src-tauri/target/release/bundle/macos/OpenClaw Manager.app"
else
    echo "src-tauri/target/release/openclaw-manager"
fi
echo "========================================"
echo ""
