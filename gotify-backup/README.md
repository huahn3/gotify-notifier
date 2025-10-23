# Gotify Notifier

一个使用 Rust + Tauri 构建的 Gotify 通知客户端，支持 Windows、macOS 和 Linux。

## 功能特性

- ✅ 实时接收 Gotify 服务器推送的消息
- ✅ macOS 系统原生通知
- ✅ 系统托盘图标，后台运行
- ✅ 优雅的用户界面
- ✅ 消息历史记录
- ✅ 消息优先级显示
- ✅ WebSocket 自动重连

## 安装要求

在构建应用之前，确保你的系统已安装：

1. **Rust** (最新稳定版)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Node.js** (可选，如果需要使用前端构建工具)

3. **Xcode Command Line Tools** (macOS)
   ```bash
   xcode-select --install
   ```

## 构建步骤

### 开发模式

```bash
# 安装 Tauri CLI
cargo install tauri-cli

# 运行开发模式
cargo tauri dev
```

### 生产构建

#### 本地构建（当前平台）

```bash
# 构建当前平台的应用
cargo tauri build

# macOS: target/release/bundle/macos/Gotify Notifier.app
# Windows: target/release/bundle/msi/*.msi
# Linux: target/release/bundle/deb/*.deb
```

#### 使用 GitHub Actions 多平台构建（推荐）

项目已配置 GitHub Actions，可自动构建所有平台的安装包：

**方法 1：手动触发构建**
1. 将代码推送到 GitHub
2. 进入仓库的 Actions 页面
3. 选择 "Build and Release" 工作流
4. 点击 "Run workflow" 手动触发
5. 构建完成后在 Artifacts 中下载安装包

**方法 2：自动发布（推荐）**
```bash
# 使用提供的部署脚本
./deploy.sh

# 或手动执行
git tag v0.1.0
git push origin v0.1.0
```

构建将自动开始，完成后会创建 GitHub Release。

详细说明请查看 [GitHub Actions 使用指南](./GITHUB_ACTIONS_GUIDE.md)

## 使用说明

### 1. 获取 Client Token

⚠️ **重要提示**：必须使用 **Client Token**，而非 Application Token！

1. 登录你的 Gotify 服务器 Web 界面
2. 进入 **Clients** 页面（不是 Applications 页面）
3. 点击 "Create Client" 创建新客户端
4. 复制生成的 Client Token

### 2. 配置应用

1. 启动 Gotify Notifier
2. 在配置区域填写：
   - **Gotify 服务器地址**：例如 `https://gotify.example.com`
   - **Client Token**：从 Clients 页面获取的 token
3. 点击 "保存配置"

### 3. 连接服务器

1. 点击 "连接" 按钮
2. 状态指示器变为绿色表示连接成功
3. 应用会自动接收并显示消息

### 4. 后台运行

- 关闭窗口后，应用会继续在系统托盘运行
- 点击托盘图标可重新打开窗口
- 右键托盘图标选择 "Quit" 完全退出应用

## 项目结构

```
gotify/
├── src/
│   └── main.rs           # Rust 后端代码
├── ui/
│   ├── index.html        # 前端界面
│   ├── styles.css        # 样式文件
│   └── app.js            # 前端逻辑
├── icons/                # 应用图标
├── Cargo.toml            # Rust 依赖配置
├── tauri.conf.json       # Tauri 配置
└── build.rs              # 构建脚本
```

## 技术栈

- **后端**: Rust
- **框架**: Tauri 1.5
- **WebSocket**: tokio-tungstenite
- **前端**: 原生 HTML/CSS/JavaScript
- **异步运行时**: Tokio

## 核心功能说明

### WebSocket 连接

应用使用 WebSocket 连接到 Gotify 服务器的 `/stream` 端点：

```
wss://your-server.com/stream?token=YOUR_CLIENT_TOKEN
```

### 系统通知

收到消息后会自动触发 macOS 系统通知，显示：
- 消息标题
- 消息内容

### 消息优先级

- 🔴 高优先级 (8-10)：红色标记
- 🟡 中优先级 (5-7)：黄色标记
- 🟢 低优先级 (0-4)：绿色标记

## 常见问题

### Q: 连接显示 403 错误？

**A**: 你可能使用了错误的 Token 类型。请确保：
- 使用 **Client Token**（从 Clients 页面创建）
- 不要使用 Application Token（从 Applications 页面创建）

### Q: 收不到通知？

**A**: 请检查：
1. macOS 系统设置中是否允许应用发送通知
2. WebSocket 连接状态是否为 "已连接"
3. Gotify 服务器是否正常运行

### Q: 如何让应用开机自启动？

**A**: 
1. 打开 "系统偏好设置" > "用户与群组"
2. 选择 "登录项"
3. 点击 "+" 添加 Gotify Notifier.app

## 开发说明

### 添加新功能

1. Rust 后端：修改 `src/main.rs`
2. 前端界面：修改 `ui/` 目录下的文件
3. 配置更改：修改 `tauri.conf.json`

### 调试

```bash
# 查看控制台输出
cargo tauri dev

# Rust 日志会在终端显示
# 前端日志在开发者工具中查看 (Cmd+Opt+I)
```

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！

## 联系方式

如有问题或建议，欢迎创建 Issue。
