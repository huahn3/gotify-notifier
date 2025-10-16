// 调试模式
const DEBUG = true;

let debugLogs = [];
let debugConsole, clearDebugBtn;
let serverUrlInput, clientTokenInput, saveConfigBtn, loadConfigBtn;
let connectBtn, disconnectBtn, statusDot, statusText, messageList;
let messages = [];

function addDebugLog(message, type = 'info') {
    const timestamp = new Date().toLocaleTimeString('zh-CN');
    const logEntry = { timestamp, message, type };
    debugLogs.push(logEntry);
    
    // 保持最多 100 条日志
    if (debugLogs.length > 100) {
        debugLogs.shift();
    }
    
    if (debugConsole) {
        renderDebugLogs();
    }
}

function renderDebugLogs() {
    if (!debugConsole) return;
    
    if (debugLogs.length === 0) {
        debugConsole.innerHTML = '<div class="debug-log">等待日志...</div>';
        return;
    }
    
    debugConsole.innerHTML = debugLogs.map(log => {
        return `<div class="debug-log ${log.type}">[${log.timestamp}] ${log.message}</div>`;
    }).reverse().join('');
    
    // 自动滚动到底部
    debugConsole.scrollTop = debugConsole.scrollHeight;
}

function debugLog(...args) {
    const message = args.join(' ');
    console.log('[Gotify Debug]', ...args);
    if (DEBUG) {
        addDebugLog(message, 'info');
    }
}

function debugError(...args) {
    const message = args.join(' ');
    console.error('[Gotify Error]', ...args);
    addDebugLog('❌ ' + message, 'error');
}

function debugWarn(...args) {
    const message = args.join(' ');
    console.warn('[Gotify Warn]', ...args);
    addDebugLog('⚠️ ' + message, 'warn');
}

// 早期日志记录
debugLog('🚀 脚本开始加载...');

// 页面加载时检查连接状态
window.addEventListener('DOMContentLoaded', async () => {
    debugLog('🏁 DOM 加载完成');
    
    // 初始化 DOM 元素
    debugConsole = document.getElementById('debugConsole');
    clearDebugBtn = document.getElementById('clearDebug');
    serverUrlInput = document.getElementById('serverUrl');
    clientTokenInput = document.getElementById('clientToken');
    saveConfigBtn = document.getElementById('saveConfig');
    loadConfigBtn = document.getElementById('loadConfig');
    connectBtn = document.getElementById('connectBtn');
    disconnectBtn = document.getElementById('disconnectBtn');
    statusDot = document.getElementById('statusDot');
    statusText = document.getElementById('statusText');
    messageList = document.getElementById('messageList');
    
    // 渲染已有的日志
    renderDebugLogs();
    
    debugLog('✅ DOM 元素已初始化');
    
    // 检查 Tauri 环境
    debugLog('🔍 检查 Tauri 环境...');
    debugLog('window.__TAURI_INTERNALS__:', typeof window.__TAURI_INTERNALS__);
    debugLog('window.ipc:', typeof window.ipc);
    
    // Tauri v2 使用 window.ipc 而不是 window.__TAURI__
    if (typeof window.__TAURI_INTERNALS__ === 'undefined' && typeof window.ipc === 'undefined') {
        debugError('Tauri API 未找到！应用可能未在 Tauri 环境中运行');
        debugError('请确保应用在 Tauri 环境中运行，而非浏览器中打开');
        alert('错误：Tauri API 未初始化\n\n请使用 cargo tauri dev 启动应用');
        return;
    }
    
    debugLog('✅ Tauri 环境已确认');
    
    // 使用 Tauri v2 的 invoke 和 listen
    const invoke = window.__TAURI_INTERNALS__.invoke || window.ipc?.postMessage;
    const listen = window.__TAURI_INTERNALS__.listen || window.ipc?.listen;
    
    if (!invoke) {
        debugError('invoke 函数不可用');
        return;
    }
    
    debugLog('✅ Tauri API 可用');
    
    // 绑定事件
    debugLog('🔗 绑定事件监听器...');
    
    // 清空调试日志
    clearDebugBtn.addEventListener('click', () => {
        debugLogs = [];
        renderDebugLogs();
        debugLog('调试日志已清空');
    });
    
    // 保存配置
    saveConfigBtn.addEventListener('click', async () => {
        debugLog('🔧 点击保存配置按钮');
        const serverUrl = serverUrlInput.value.trim();
        const clientToken = clientTokenInput.value.trim();

        debugLog('服务器地址:', serverUrl);
        debugLog('Token 长度:', clientToken.length, '字符');

        if (!serverUrl || !clientToken) {
            const msg = '请填写服务器地址和 Client Token';
            alert(msg);
            debugError(msg);
            return;
        }

        try {
            debugLog('📤 调用 save_config 命令...');
            const result = await invoke('save_config', {
                serverUrl,
                clientToken
            });
            debugLog('✅ 配置保存成功:', result);
            alert('配置保存成功！');
        } catch (error) {
            debugError('保存配置失败:', error);
            alert('保存配置失败: ' + error);
        }
    });

    // 加载配置
    loadConfigBtn.addEventListener('click', async () => {
        debugLog('📂 点击加载配置按钮');
        try {
            debugLog('📥 调用 get_config 命令...');
            const config = await invoke('get_config');
            debugLog('✅ 配置加载结果:', config);
            if (config) {
                serverUrlInput.value = config.server_url || '';
                clientTokenInput.value = config.client_token || '';
                debugLog('配置已填充到表单');
                alert('配置加载成功！');
            } else {
                const msg = '暂无保存的配置';
                alert(msg);
                debugWarn(msg);
            }
        } catch (error) {
            debugError('加载配置失败:', error);
            alert('加载配置失败: ' + error);
        }
    });

    // 连接 WebSocket
    connectBtn.addEventListener('click', async () => {
        debugLog('🔌 点击连接按钮');
        try {
            debugLog('📤 调用 start_websocket 命令...');
            const result = await invoke('start_websocket');
            debugLog('✅ WebSocket 启动成功:', result);
            connectBtn.disabled = true;
            disconnectBtn.disabled = false;
        } catch (error) {
            debugError('WebSocket 启动失败:', error);
            alert('连接失败: ' + error);
        }
    });

    // 断开 WebSocket
    disconnectBtn.addEventListener('click', async () => {
        debugLog('🔌 点击断开按钮');
        try {
            debugLog('📤 调用 stop_websocket 命令...');
            const result = await invoke('stop_websocket');
            debugLog('✅ WebSocket 已停止:', result);
            updateStatus('disconnected');
            connectBtn.disabled = false;
            disconnectBtn.disabled = true;
        } catch (error) {
            debugError('WebSocket 停止失败:', error);
            alert('断开失败: ' + error);
        }
    });
    
    // 监听 WebSocket 状态变化
    debugLog('👂 注册事件监听器...');
    listen('websocket-status', (event) => {
        debugLog('🔔 收到状态事件:', event.payload);
        updateStatus(event.payload);
    });

    // 监听 Gotify 消息
    listen('gotify-message', (event) => {
        debugLog('🔔 收到 Gotify 消息事件');
        addMessage(event.payload);
    });
    
    debugLog('✅ 事件监听器已注册');
    
    try {
        debugLog('🔍 检查 WebSocket 状态...');
        const isRunning = await invoke('is_websocket_running');
        debugLog('WebSocket 运行状态:', isRunning);
        if (isRunning) {
            updateStatus('connected');
        }
        
        // 尝试加载配置
        debugLog('🔍 加载保存的配置...');
        const config = await invoke('get_config');
        if (config) {
            debugLog('✅ 发现已保存配置');
            serverUrlInput.value = config.server_url || '';
            clientTokenInput.value = config.client_token || '';
        } else {
            debugLog('ℹ️ 未发现已保存配置');
        }
        
        debugLog('🎉 应用初始化完成！');
    } catch (error) {
        debugError('初始化失败:', error);
    }
});
