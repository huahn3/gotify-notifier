use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager, State};
use tauri::tray::{TrayIconBuilder, TrayIcon};
use futures_util::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use image::GenericImageView;

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

#[derive(Clone, Serialize, Deserialize, Debug)]
struct WindowPosition {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

struct AppState {
    config: Arc<Mutex<Option<ConnectionConfig>>>,
    ws_running: Arc<Mutex<bool>>,
    window_position: Arc<Mutex<Option<WindowPosition>>>,
    tray_icon: Arc<Mutex<Option<TrayIcon>>>,
}

// 获取跨平台的配置目录
fn get_config_dir() -> String {
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
    }
}

#[tauri::command]
async fn save_config_to_file(
    state: State<'_, AppState>,
    server_url: String,
    client_token: String,
) -> Result<String, String> {
    let config = ConnectionConfig {
        server_url,
        client_token,
    };

    let mut state_config = state.config.lock().unwrap();
    *state_config = Some(config.clone());

    // 保存到文件
    let config_dir = get_config_dir();
    let config_path = format!("{}/.gotify_config.json", config_dir);

    match std::fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()) {
        Ok(_) => {
            println!("✅ 配置已保存到文件: {}", config_path);
            Ok("Configuration saved successfully".to_string())
        },
        Err(e) => {
            eprintln!("❌ 保存配置失败: {}", e);
            Err(format!("Failed to save config: {}", e))
        }
    }
}

#[tauri::command]
async fn test_settings_window(app: tauri::AppHandle) -> Result<(), String> {
    println!("🧪 测试调用设置窗口...");
    show_settings_window(app).await
}

#[tauri::command]
async fn show_settings_window(app: tauri::AppHandle) -> Result<(), String> {
    // 检查是否已有设置窗口
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        return Ok(());
    }

    // 创建设置窗口
    let _settings_window = tauri::WebviewWindowBuilder::new(
        &app,
        "settings",
        tauri::WebviewUrl::App("settings.html".into())
    )
    .title("Gotify 设置")
    .inner_size(800.0, 600.0)
    .min_inner_size(700.0, 500.0)
    .center()
    .build()
    .map_err(|e| format!("Failed to create settings window: {}", e))?;

    Ok(())
}

fn load_config_from_file() -> Option<ConnectionConfig> {
    let config_dir = get_config_dir();
    let config_path = format!("{}/.gotify_config.json", config_dir);

    match std::fs::read_to_string(&config_path) {
        Ok(content) => {
            match serde_json::from_str::<ConnectionConfig>(&content) {
                Ok(config) => {
                    println!("✅ 从文件加载配置成功: {}", config_path);
                    Some(config)
                },
                Err(e) => {
                    eprintln!("❌ 解析配置文件失败: {}", e);
                    // 如果配置文件解析失败，尝试从环境变量加载
                    load_config_from_env()
                }
            }
        },
        Err(_) => {
            println!("ℹ️ 未找到配置文件，尝试从环境变量加载");
            load_config_from_env()
        }
    }
}

fn load_config_from_env() -> Option<ConnectionConfig> {
    let server_url = std::env::var("GOTIFY_SERVER_URL")
        .or_else(|_| std::env::var("GOTIFY_URL"))
        .unwrap_or_else(|_| "http://localhost:8080".to_string());

    let client_token = std::env::var("GOTIFY_CLIENT_TOKEN")
        .or_else(|_| std::env::var("GOTIFY_TOKEN"))
        .unwrap_or_else(|_| "".to_string());

    // 只有当Token不为空时才返回配置
    if !client_token.is_empty() {
        println!("✅ 从环境变量加载配置成功");
        Some(ConnectionConfig {
            server_url,
            client_token,
        })
    } else {
        println!("ℹ️ 未找到有效的配置，请通过设置界面配置");
        None
    }
}

#[tauri::command]
async fn get_config(state: State<'_, AppState>) -> Result<Option<ConnectionConfig>, String> {
    let config = state.config.lock().unwrap();
    Ok(config.clone())
}

#[tauri::command]
async fn save_config(
    state: State<'_, AppState>,
    server_url: String,
    client_token: String,
) -> Result<String, String> {
    save_config_to_file(state, server_url, client_token).await
}

#[tauri::command]
async fn test_connection(
    server_url: String,
    client_token: String,
) -> Result<String, String> {
    // 构建 WebSocket URL
    let ws_url = {
        let server_url = server_url.trim_end_matches('/');
        let protocol = if server_url.starts_with("https://") {
            "wss://"
        } else {
            "ws://"
        };
        let host = server_url
            .trim_start_matches("https://")
            .trim_start_matches("http://");
        format!("{}{}/stream?token={}", protocol, host, client_token)
    };

    println!("🧪 测试连接到: {}", ws_url);

    // 尝试解析 URL
    let url = url::Url::parse(&ws_url)
        .map_err(|e| format!("URL 解析失败: {}", e))?;

    // 尝试连接 WebSocket
    match tokio::time::timeout(
        std::time::Duration::from_secs(10),
        connect_async(url)
    ).await {
        Ok(Ok((ws_stream, _))) => {
            println!("✅ WebSocket 连接测试成功");
            drop(ws_stream); // 立即关闭测试连接
            Ok("连接成功".to_string())
        }
        Ok(Err(e)) => {
            eprintln!("❌ WebSocket 连接失败: {}", e);
            Err(format!("连接失败: {}。请检查服务器地址和 Token 是否正确", e))
        }
        Err(_) => {
            eprintln!("❌ 连接超时");
            Err("连接超时，请检查服务器地址是否可访问".to_string())
        }
    }
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

#[tauri::command]
async fn save_window_position_auto(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let window = app.get_webview_window("main").unwrap();
    
    // 直接从窗口获取位置和大小
    let position = window.outer_position().map_err(|e| format!("获取位置失败: {}", e))?;
    let size = window.outer_size().map_err(|e| format!("获取大小失败: {}", e))?;
    
    let window_pos = WindowPosition {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
    };
    
    let mut window_position = state.window_position.lock().unwrap();
    *window_position = Some(window_pos.clone());
    
    // 保存到文件
    let config_dir = get_config_dir();
    let config_path = format!("{}/.gotify_window_position.json", config_dir);
    
    match std::fs::write(&config_path, serde_json::to_string_pretty(&window_pos).unwrap()) {
        Ok(_) => {
            println!("✅ 窗口位置已自动保存: x={}, y={}, w={}, h={}", window_pos.x, window_pos.y, window_pos.width, window_pos.height);
            Ok(())
        },
        Err(e) => {
            eprintln!("❌ 保存窗口位置失败: {}", e);
            Err(format!("保存失败: {}", e))
        }
    }
}

#[tauri::command]
async fn save_window_position(
    state: State<'_, AppState>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let position = WindowPosition { x, y, width, height };
    let mut window_position = state.window_position.lock().unwrap();
    *window_position = Some(position.clone());
    
    // 保存到文件
    let config_dir = get_config_dir();
    let config_path = format!("{}/.gotify_window_position.json", config_dir);
    
    match std::fs::write(&config_path, serde_json::to_string_pretty(&position).unwrap()) {
        Ok(_) => {
            println!("✅ 窗口位置已保存: x={}, y={}, w={}, h={}", x, y, width, height);
            Ok(())
        },
        Err(e) => {
            eprintln!("❌ 保存窗口位置失败: {}", e);
            Err(format!("保存失败: {}", e))
        }
    }
}

#[tauri::command]
async fn load_window_position(state: State<'_, AppState>) -> Result<Option<WindowPosition>, String> {
    let config_dir = get_config_dir();
    let config_path = format!("{}/.gotify_window_position.json", config_dir);
    
    match std::fs::read_to_string(&config_path) {
        Ok(content) => {
            match serde_json::from_str::<WindowPosition>(&content) {
                Ok(position) => {
                    let mut window_position = state.window_position.lock().unwrap();
                    *window_position = Some(position.clone());
                    println!("✅ 窗口位置已加载: x={}, y={}, w={}, h={}", position.x, position.y, position.width, position.height);
                    Ok(Some(position))
                },
                Err(e) => {
                    eprintln!("❌ 解析窗口位置失败: {}", e);
                    Ok(None)
                }
            }
        },
        Err(_) => {
            println!("ℹ️ 未找到保存的窗口位置");
            Ok(None)
        }
    }
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
            config: Arc::new(Mutex::new(load_config_from_file())),
            ws_running: Arc::new(Mutex::new(false)),
            window_position: Arc::new(Mutex::new(None)),
            tray_icon: Arc::new(Mutex::new(None)),
        })
        .setup(|app| {
            let app_handle = app.handle().clone();
            let main_window = app.get_webview_window("main").unwrap();
            
            // 恢复窗口位置
            tauri::async_runtime::spawn(async move {
                let state: tauri::State<AppState> = app_handle.state();
                
                // 加载保存的窗口位置
                let config_dir = get_config_dir();
                let config_path = format!("{}/.gotify_window_position.json", config_dir);
                
                // 确保窗口可见且未最小化（Windows 兼容性）
                let _ = main_window.unminimize();
                let _ = main_window.show();
                let _ = main_window.set_focus();
                
                if let Ok(content) = std::fs::read_to_string(&config_path) {
                    if let Ok(position) = serde_json::from_str::<WindowPosition>(&content) {
                        println!("🪟 恢复窗口位置: x={}, y={}, w={}, h={}", position.x, position.y, position.width, position.height);
                        
                        // 设置窗口位置和大小
                        let _ = main_window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                            x: position.x,
                            y: position.y,
                        }));
                        let _ = main_window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                            width: position.width,
                            height: position.height,
                        }));
                        
                        // 更新状态
                        let mut window_position = state.window_position.lock().unwrap();
                        *window_position = Some(position);
                    }
                } else {
                    println!("ℹ️ 未找到保存的窗口位置，使用默认位置");
                    // 如果没有保存的位置，居中显示
                    let _ = main_window.center();
                }

                // 等待 1 秒后自动连接
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

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

            // 创建系统托盘
            use tauri::{menu::{Menu, MenuItem}};

            println!("🔧 开始创建系统托盘...");
            let settings_item = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&settings_item, &quit_item])?;

            // 加载托盘图标 - 解码 PNG 以获得 RGBA 数据
            let icon_png = include_bytes!("../icons/icon.png");
            let icon_image = image::load_from_memory(icon_png)
                .map_err(|e| format!("Failed to decode tray icon: {}", e))?;
            let (icon_width, icon_height) = icon_image.dimensions();
            let icon_rgba = icon_image.into_rgba8().into_raw();
            let icon = tauri::image::Image::new_owned(icon_rgba, icon_width, icon_height);
            println!("✅ 托盘图标加载成功");

            let tray = TrayIconBuilder::new()
                .icon(icon)
                .menu(&tray_menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "settings" => {
                            println!("🖱️ 用户点击了设置菜单");
                            let _ = app.emit("show-settings", ());
                        }
                        "quit" => {
                            println!("🚪 用户点击了退出菜单");
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;
            println!("✅ 系统托盘创建成功");

            // 保存tray到状态
            let state: tauri::State<AppState> = app.state();
            let mut tray_icon = state.tray_icon.lock().unwrap();
            *tray_icon = Some(tray);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            save_config,
            save_config_to_file,
            get_config,
            test_connection,
            start_websocket,
            stop_websocket,
            is_websocket_running,
            save_window_position,
            save_window_position_auto,
            load_window_position,
            show_settings_window,
            test_settings_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
