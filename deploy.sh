#!/bin/bash

# GitHub Actions 部署脚本

echo "🚀 Gotify Notifier - GitHub Actions 部署助手"
echo "==========================================="
echo ""

# 检查是否已经初始化 git
if [ ! -d ".git" ]; then
    echo "❌ 未找到 .git 目录，请先初始化 Git 仓库"
    echo ""
    echo "运行以下命令初始化："
    echo "  git init"
    echo "  git add ."
    echo "  git commit -m 'Initial commit'"
    echo "  git remote add origin <你的GitHub仓库地址>"
    echo ""
    exit 1
fi

# 检查是否有未提交的改动
if [ -n "$(git status --porcelain)" ]; then
    echo "📝 检测到未提交的改动，正在提交..."
    git add .
    read -p "请输入提交信息（直接回车使用默认）: " commit_msg
    if [ -z "$commit_msg" ]; then
        commit_msg="更新代码并添加 GitHub Actions 配置"
    fi
    git commit -m "$commit_msg"
    echo "✅ 改动已提交"
    echo ""
fi

# 询问是否推送
read -p "是否推送到 GitHub？(y/n): " push_confirm
if [ "$push_confirm" = "y" ] || [ "$push_confirm" = "Y" ]; then
    # 获取当前分支
    current_branch=$(git branch --show-current)
    echo "📤 正在推送到分支: $current_branch"
    git push origin $current_branch
    echo "✅ 代码已推送"
    echo ""
fi

# 询问是否创建 tag 并发布
read -p "是否创建版本 tag 并触发构建？(y/n): " tag_confirm
if [ "$tag_confirm" = "y" ] || [ "$tag_confirm" = "Y" ]; then
    echo ""
    echo "📋 现有的 tags:"
    git tag -l
    echo ""
    read -p "请输入新版本号（例如 v0.1.0）: " version
    
    if [ -z "$version" ]; then
        echo "❌ 版本号不能为空"
        exit 1
    fi
    
    # 确保版本号以 v 开头
    if [[ ! $version == v* ]]; then
        version="v$version"
    fi
    
    # 检查 tag 是否已存在
    if git rev-parse "$version" >/dev/null 2>&1; then
        echo "❌ Tag $version 已存在"
        exit 1
    fi
    
    read -p "请输入 tag 描述（可选，直接回车跳过）: " tag_desc
    
    if [ -z "$tag_desc" ]; then
        git tag $version
    else
        git tag -a $version -m "$tag_desc"
    fi
    
    echo "✅ Tag $version 已创建"
    echo ""
    
    read -p "是否推送 tag 到 GitHub？这将触发自动构建。(y/n): " push_tag_confirm
    if [ "$push_tag_confirm" = "y" ] || [ "$push_tag_confirm" = "Y" ]; then
        git push origin $version
        echo "✅ Tag 已推送"
        echo ""
        echo "🎉 完成！GitHub Actions 正在构建多平台安装包..."
        echo ""
        echo "你可以在以下位置查看构建进度："
        
        # 尝试获取远程仓库 URL
        remote_url=$(git config --get remote.origin.url)
        if [[ $remote_url == git@github.com:* ]]; then
            # SSH URL
            repo_path=${remote_url#git@github.com:}
            repo_path=${repo_path%.git}
            echo "  https://github.com/$repo_path/actions"
        elif [[ $remote_url == https://github.com/* ]]; then
            # HTTPS URL
            repo_path=${remote_url#https://github.com/}
            repo_path=${repo_path%.git}
            echo "  https://github.com/$repo_path/actions"
        else
            echo "  GitHub 仓库的 Actions 页面"
        fi
        echo ""
        echo "构建完成后，前往 Releases 页面发布新版本："
        if [[ $remote_url == git@github.com:* ]] || [[ $remote_url == https://github.com/* ]]; then
            echo "  https://github.com/$repo_path/releases"
        else
            echo "  GitHub 仓库的 Releases 页面"
        fi
    fi
fi

echo ""
echo "🎊 所有操作完成！"
