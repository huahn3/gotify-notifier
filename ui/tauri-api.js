// Tauri API 初始化脚本
// 该脚本会在页面加载时被 Tauri 注入，提供 window.__TAURI__ 对象

console.log('🚀 Tauri API 初始化脚本加载');

// 等待 Tauri 运行时就绪
if (window.__TAURI_INTERNALS__) {
    console.log('✅ Tauri Internals 可用');
    console.log('__TAURI_INTERNALS__ 属性:', Object.keys(window.__TAURI_INTERNALS__));
    
    // 检查是否有 invoke 函数
    if (window.__TAURI_INTERNALS__.invoke) {
        console.log('✅ invoke 函数可用');
        
        // 创建 __TAURI__ 对象
        window.__TAURI__ = window.__TAURI__ || {};
        window.__TAURI__.event = window.__TAURI__.event || {};
        
        // 手动实现 listen 函数
        window.__TAURI__.event.listen = async function(eventName, handler) {
            console.log(`📡 注册事件监听: ${eventName}`);
            
            const { transformCallback, invoke } = window.__TAURI_INTERNALS__;
            const callbackId = transformCallback(handler);
            
            // 使用 Tauri 内部的消息系统
            // Tauri v2 需要 target 参数来指定事件目标
            await invoke('plugin:event|listen', {
                event: eventName,
                target: { kind: 'Any' },  // 监听来自任何源的事件
                handler: callbackId
            });
            
            console.log(`✅ 事件 ${eventName} 监听器已注册`);
            
            // 返回取消监听的函数
            return async function unlisten() {
                await invoke('plugin:event|unlisten', {
                    event: eventName,
                    eventId: callbackId
                });
            };
        };
        
        console.log('✅ window.__TAURI__.event.listen 已创建');
    } else {
        console.error('❌ __TAURI_INTERNALS__.invoke 不可用，无法创建事件监听器');
        console.error('请检查 Tauri 版本和配置');
    }
} else {
    console.error('❌ window.__TAURI_INTERNALS__ 不可用');
}
