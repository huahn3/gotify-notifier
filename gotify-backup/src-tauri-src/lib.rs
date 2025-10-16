use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager, State};
use futures_util::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[derive(Clone, Serialize, Deserialize, Debug)]
struct GotifyMessage {
    id: u64,
    appid: u64,
    message: String,
    title: String,
    priority: u32,
    date: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct ConnectionConfig {
    server_url: String,
    client_token: String,
}

struct AppState {
    config: Arc<Mutex<Option<ConnectionConfig>>>,
    ws_running: Arc<Mutex<bool>>,
}

#[tauri::command]
async fn save_config(
    state: State<'_, AppState>,
    server_url: String,
    client_token: String,
) -> Result<String, String> {
    let config = ConnectionConfig {
        server_url,
        client_token,
    };
    
    let mut state_config = state.config.lock().unwrap();
    *state_config = Some(config);
    
    Ok("Configuration saved successfully".to_string())
}

#[tauri::command]
async fn get_config(state: State<'_, AppState>) -> Result<Option<ConnectionConfig>, String> {
    let config = state.config.lock().unwrap();
    Ok(config.clone())
}

#[tauri::command]
async fn start_websocket(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let config = {
        let cfg = state.config.lock().unwrap();
        cfg.clone()
    };

    let config = config.ok_or("No configuration found. Please configure first.")?;
    
    let mut ws_running = state.ws_running.lock().unwrap();
    if *ws_running {
        return Err("WebSocket is already running".to_string());
    }
    *ws_running = true;
    drop(ws_running);

    let ws_running_clone = state.ws_running.clone();
    
    tokio::spawn(async move {
        if let Err(e) = run_websocket(app, config, ws_running_clone).await {
            eprintln!("WebSocket error: {}", e);
        }
    });

    Ok("WebSocket connection started".to_string())
}

#[tauri::command]
async fn stop_websocket(state: State<'_, AppState>) -> Result<String, String> {
    let mut ws_running = state.ws_running.lock().unwrap();
    *ws_running = false;
    Ok("WebSocket connection stopped".to_string())
}

#[tauri::command]
async fn is_websocket_running(state: State<'_, AppState>) -> Result<bool, String> {
    let ws_running = state.ws_running.lock().unwrap();
    Ok(*ws_running)
}



async fn run_websocket(
    app: tauri::AppHandle,
    config: ConnectionConfig,
    ws_running: Arc<Mutex<bool>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // 构建 WebSocket URL
    let ws_url = {
        let server_url = config.server_url.trim_end_matches('/');
        let protocol = if server_url.starts_with("https://") {
            "wss://"
        } else {
            "ws://"
        };
        let host = server_url
            .trim_start_matches("https://")
            .trim_start_matches("http://");
        format!("{}{}/stream?token={}", protocol, host, config.client_token)
    };

    println!("Connecting to: {}", ws_url);

    let url = url::Url::parse(&ws_url)?;
    let (ws_stream, _) = connect_async(url).await?;
    println!("WebSocket connected successfully");

    let (_write, mut read) = ws_stream.split();

    // 发送连接成功消息到前端
    println!("📤 发送连接状态到前端...");
    match app.emit("websocket-status", "connected") {
        Ok(_) => println!("✅ websocket-status 已推送"),
        Err(e) => eprintln!("❌ websocket-status 推送失败: {}", e),
    }

    while *ws_running.lock().unwrap() {
        tokio::select! {
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        println!("Received message: {}", text);
                        
                        // 解析 Gotify 消息
                        if let Ok(gotify_msg) = serde_json::from_str::<GotifyMessage>(&text) {
                            println!("📨 收到 Gotify 消息:");
                            println!("   标题: {}", gotify_msg.title);
                            println!("   内容: {}", gotify_msg.message);
                            println!("   优先级: {}", gotify_msg.priority);
                            
                            // 发送 macOS 系统通知
                            use tauri_plugin_notification::NotificationExt;
                            
                            println!("🔔 准备发送系统通知...");
                            println!("   标题: {}", gotify_msg.title);
                            println!("   优先级: {}", gotify_msg.priority);
                            
                            // 构建通知
                            let mut notification = app.notification().builder();
                            notification = notification
                                .title(&gotify_msg.title)
                                .body(&gotify_msg.message);
                            
                            // 高优先级消息添加声音
                            if gotify_msg.priority >= 5 {
                                println!("   添加默认系统声音");
                                notification = notification.sound("default");
                            }
                            
                            match notification.show() {
                                Ok(_) => println!("✅ macOS 系统通知已成功发送"),
                                Err(e) => eprintln!("❌ 发送通知失败: {:?}", e),
                            }
                            
                            // 发送消息到前端
                            println!("📤 发送消息到前端...");
                            
                            // 发送到前端
                            match app.emit("gotify-message", &gotify_msg) {
                                Ok(_) => println!("✅ 消息已推送到前端"),
                                Err(e) => eprintln!("❌ 推送失败: {}", e),
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        println!("WebSocket closed");
                        break;
                    }
                    Some(Err(e)) => {
                        eprintln!("WebSocket error: {}", e);
                        break;
                    }
                    None => {
                        println!("WebSocket stream ended");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    app.emit("websocket-status", "disconnected").ok();
    let mut running = ws_running.lock().unwrap();
    *running = false;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            config: Arc::new(Mutex::new(Some(ConnectionConfig {
                server_url: "http://192.168.31.88:7777".to_string(),
                client_token: "CDEtcxPRxdQM1qf".to_string(),
            }))),
            ws_running: Arc::new(Mutex::new(false)),
        })
        .setup(|app| {
            // 应用启动时自动连接
            let app_handle = app.handle().clone();
            
            tauri::async_runtime::spawn(async move {
                // 等待 1 秒后自动连接
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                
                let state: tauri::State<AppState> = app_handle.state();
                
                let config = {
                    let cfg = state.config.lock().unwrap();
                    cfg.clone()
                };
                
                if let Some(config) = config {
                    println!("🚀 自动连接到 Gotify 服务器...");
                    
                    let mut ws_running = state.ws_running.lock().unwrap();
                    if !*ws_running {
                        *ws_running = true;
                        drop(ws_running);
                        
                        let ws_running_clone = state.ws_running.clone();
                        tokio::spawn(async move {
                            if let Err(e) = run_websocket(app_handle, config, ws_running_clone).await {
                                eprintln!("WebSocket error: {}", e);
                            }
                        });
                    }
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            save_config,
            get_config,
            start_websocket,
            stop_websocket,
            is_websocket_running,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
