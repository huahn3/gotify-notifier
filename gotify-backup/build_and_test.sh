#!/bin/bash

echo "🔨 构建 Gotify Notifier..."
echo ""

cd src-tauri
cargo tauri build --debug

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ 构建成功！"
    echo ""
    echo "📦 应用位置:"
    echo "   src-tauri/target/debug/bundle/macos/Gotify Notifier.app"
    echo ""
    echo "🚀 启动应用..."
    open "target/debug/bundle/macos/Gotify Notifier.app"
else
    echo ""
    echo "❌ 构建失败"
    exit 1
fi
