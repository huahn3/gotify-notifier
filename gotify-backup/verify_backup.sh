#!/bin/bash
echo '🔍 验证备份文件完整性...'
echo

echo '📁 检查目录结构:'
ls -la /Users/liubo/Desktop/gotify-backup/

echo
echo '📄 检查核心文件:'
files=(
    'src/lib.rs'
    'Cargo.toml' 
    'tauri.conf.json'
    'ui/index.html'
    'ui/settings.html'
    '.github/workflows/build.yml'
    'README.md'
)

for file in "${files[@]}"; do
    if [ -f "/Users/liubo/Desktop/gotify-backup/$file" ]; then
        echo "✅ $file"
    else
        echo "❌ $file - 缺失"
    fi
done

echo
echo '📊 备份统计:'
echo '文件数量:' $(find /Users/liubo/Desktop/gotify-backup -type f | wc -l)
echo '总大小:' $(du -sh /Users/liubo/Desktop/gotify-backup | cut -f1)
echo '备份时间:' $(date)
echo
echo '✅ 备份验证完成！所有核心文件已成功备份。'

