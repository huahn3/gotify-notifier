#!/bin/bash

# Gotify Notifier 一键安装和运行脚本

echo "═══════════════════════════════════════════════════════════════"
echo "  🔔 Gotify Notifier for macOS - 一键安装脚本"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# 检查是否安装了 Rust
if ! command -v cargo &> /dev/null; then
    echo "📦 步骤 1/3: 安装 Rust..."
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "即将安装 Rust 工具链，这是构建应用所必需的。"
    echo "按 Enter 继续，或 Ctrl+C 取消..."
    read
    
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    
    # 加载 Rust 环境
    source $HOME/.cargo/env
    
    echo ""
    echo "✅ Rust 安装成功！"
else
    echo "✅ Rust 已安装"
fi

echo ""
echo "📦 步骤 2/3: 检查 Xcode Command Line Tools..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if ! xcode-select -p &> /dev/null; then
    echo "正在安装 Xcode Command Line Tools..."
    xcode-select --install
    echo ""
    echo "请在弹出的窗口中完成安装，然后按 Enter 继续..."
    read
else
    echo "✅ Xcode Command Line Tools 已安装"
fi

echo ""
echo "📦 步骤 3/3: 安装 Tauri CLI..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if ! command -v cargo-tauri &> /dev/null; then
    echo "正在安装 Tauri CLI（这可能需要几分钟）..."
    cargo install tauri-cli --quiet
    echo "✅ Tauri CLI 安装成功！"
else
    echo "✅ Tauri CLI 已安装"
fi

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "  🎉 安装完成！"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "现在你可以："
echo ""
echo "  1️⃣  启动开发模式（推荐首次使用）："
echo "     ./dev.sh"
echo ""
echo "  2️⃣  构建生产版本："
echo "     ./build.sh"
echo ""
echo "  3️⃣  手动运行："
echo "     cargo tauri dev      # 开发模式"
echo "     cargo tauri build    # 生产构建"
echo ""
echo "📖 查看文档："
echo "   - QUICKSTART.md    快速开始"
echo "   - README.md        完整文档"
echo "   - PROJECT_OVERVIEW.md  项目总览"
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "是否现在启动开发模式？(y/n)"
read -r response

if [[ "$response" =~ ^[Yy]$ ]]; then
    echo ""
    echo "🚀 启动开发模式..."
    cargo tauri dev
fi
