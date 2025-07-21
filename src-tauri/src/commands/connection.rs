use crate::communication::{ConnectionError, ConnectionManager, SerialHandler};
use crate::models::{ConnectionConfig, TerminalMessage};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, error, info};

// アプリケーション状態
pub struct AppState {
    pub connection_manager: Arc<Mutex<ConnectionManager>>,
    pub message_receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<TerminalMessage>>>>,
    pub message_sender: Arc<Mutex<Option<mpsc::UnboundedSender<TerminalMessage>>>>,
}

impl AppState {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            connection_manager: Arc::new(Mutex::new(ConnectionManager::new())),
            message_receiver: Arc::new(Mutex::new(Some(rx))),
            message_sender: Arc::new(Mutex::new(Some(tx))),
        }
    }
}

// API応答型
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

// シリアルポート情報
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SerialPortInfo {
    pub port_name: String,
    pub port_type: Option<String>,
    pub vid: Option<u16>,
    pub pid: Option<u16>,
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
}

// Tauri コマンド

#[tauri::command]
pub async fn get_serial_ports() -> Result<ApiResponse<Vec<String>>, String> {
    debug!("Getting available serial ports");
    
    match SerialHandler::list_available_ports().await {
        Ok(ports) => {
            info!("Found {} serial ports", ports.len());
            Ok(ApiResponse::success(ports))
        }
        Err(e) => {
            error!("Failed to get serial ports: {}", e);
            Ok(ApiResponse::error(e.to_string()))
        }
    }
}

#[tauri::command]
pub async fn get_serial_ports_info() -> Result<ApiResponse<Vec<SerialPortInfo>>, String> {
    debug!("Getting detailed serial port information");
    
    match SerialHandler::get_port_info().await {
        Ok(info) => {
            let port_info: Vec<SerialPortInfo> = info
                .into_iter()
                .map(|port| SerialPortInfo {
                    port_name: port.port_name,
                    port_type: port.port_type,
                    vid: port.vid,
                    pid: port.pid,
                    serial_number: port.serial_number,
                    manufacturer: port.manufacturer,
                    product: port.product,
                })
                .collect();
            
            info!("Found detailed info for {} serial ports", port_info.len());
            Ok(ApiResponse::success(port_info))
        }
        Err(e) => {
            error!("Failed to get serial port info: {}", e);
            Ok(ApiResponse::error(e.to_string()))
        }
    }
}

#[tauri::command]
pub async fn connect_device(
    config: ConnectionConfig,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<ApiResponse<String>, String> {
    info!("Attempting to connect with config: {:?}", config.name);
    
    let mut connection_manager = state.connection_manager.lock().await;
    
    // メッセージチャンネルを取得
    let message_tx = {
        let sender_guard = state.message_sender.lock().await;
        match sender_guard.as_ref() {
            Some(tx) => tx.clone(),
            None => {
                error!("Message sender not available");
                return Ok(ApiResponse::error("Internal error: message sender not available".to_string()));
            }
        }
    };

    // 受信メッセージ処理を開始（初回のみ）
    start_message_handling(app_handle.clone(), state.message_receiver.clone()).await;

    // 接続実行
    match connection_manager.connect(config.clone(), message_tx).await {
        Ok(_) => {
            info!("Successfully connected to device: {}", config.name);
            
            // 接続成功イベントを送信
            let _ = app_handle.emit("connection-status-changed", ("connected", &config.name));
            
            let info = connection_manager.get_connection_info()
                .unwrap_or_else(|| "Connected".to_string());
            
            Ok(ApiResponse::success(info))
        }
        Err(e) => {
            error!("Failed to connect to device {}: {}", config.name, e);
            
            // 接続失敗イベントを送信
            let _ = app_handle.emit("connection-status-changed", ("error", e.to_string()));
            
            Ok(ApiResponse::error(e.to_string()))
        }
    }
}

#[tauri::command]
pub async fn disconnect_device(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<ApiResponse<String>, String> {
    info!("Attempting to disconnect device");
    
    let mut connection_manager = state.connection_manager.lock().await;
    
    match connection_manager.disconnect().await {
        Ok(_) => {
            info!("Successfully disconnected device");
            
            // 切断イベントを送信
            let _ = app_handle.emit("connection-status-changed", ("disconnected", ""));
            
            Ok(ApiResponse::success("Disconnected".to_string()))
        }
        Err(e) => {
            error!("Failed to disconnect device: {}", e);
            Ok(ApiResponse::error(e.to_string()))
        }
    }
}

#[tauri::command]
pub async fn send_message(
    message: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<String>, String> {
    debug!("Sending message: {}", message);
    
    let mut connection_manager = state.connection_manager.lock().await;
    
    match connection_manager.send_message(message).await {
        Ok(_) => {
            debug!("Message sent successfully");
            Ok(ApiResponse::success("Message sent".to_string()))
        }
        Err(e) => {
            error!("Failed to send message: {}", e);
            Ok(ApiResponse::error(e.to_string()))
        }
    }
}

#[tauri::command]
pub async fn get_connection_status(
    state: State<'_, AppState>,
) -> Result<ApiResponse<bool>, String> {
    let connection_manager = state.connection_manager.lock().await;
    let is_connected = connection_manager.is_connected();
    
    debug!("Connection status: {}", is_connected);
    Ok(ApiResponse::success(is_connected))
}

#[tauri::command]
pub async fn get_connection_info(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Option<String>>, String> {
    let connection_manager = state.connection_manager.lock().await;
    let info = connection_manager.get_connection_info();
    
    debug!("Connection info: {:?}", info);
    Ok(ApiResponse::success(info))
}

// メッセージハンドリングの開始（一度だけ実行される）
async fn start_message_handling(
    app_handle: AppHandle,
    message_receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<TerminalMessage>>>>
) {
    let mut receiver_guard = message_receiver.lock().await;
    
    if let Some(mut rx) = receiver_guard.take() {
        tokio::spawn(async move {
            info!("Starting message handling loop");
            
            while let Some(message) = rx.recv().await {
                debug!("Received message: {:?}", message);
                
                // フロントエンドにメッセージを送信
                if let Err(e) = app_handle.emit("terminal-message-received", &message) {
                    error!("Failed to emit terminal message: {}", e);
                }
            }
            
            info!("Message handling loop ended");
        });
    }
}

// エラー変換
impl From<ConnectionError> for String {
    fn from(error: ConnectionError) -> Self {
        error.to_string()
    }
}