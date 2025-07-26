use crate::communication::{ConnectionError, ConnectionManager, SerialHandler};
use crate::models::{ConnectionConfig, ConnectionType, SerialConfig, TcpConfig, DataBits, StopBits, Parity, FlowControl, TerminalMessage};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, error, info};
use chrono::Utc;

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

// フロントエンドからの接続設定（TypeScript側との互換性）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrontendConnectionConfig {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub connection_type: String, // "serial" or "tcp"
    #[serde(rename = "serialPort")]
    pub serial_port: Option<String>,
    #[serde(rename = "baudRate")]
    pub baud_rate: Option<u32>,
    pub host: Option<String>,
    pub port: Option<u16>,
}

// 型変換関数
impl FrontendConnectionConfig {
    pub fn to_backend_config(self) -> Result<ConnectionConfig, String> {
        let now = Utc::now();
        
        match self.connection_type.as_str() {
            "serial" => {
                let serial_port = self.serial_port
                    .ok_or_else(|| "シリアルポートが指定されていません".to_string())?;
                let baud_rate = self.baud_rate.unwrap_or(115200);
                
                let serial_config = SerialConfig {
                    port: serial_port,
                    baud_rate,
                    data_bits: DataBits::Eight,
                    stop_bits: StopBits::One,
                    parity: Parity::None,
                    flow_control: FlowControl::None,
                };
                
                Ok(ConnectionConfig {
                    id: self.id,
                    name: self.name,
                    connection_type: ConnectionType::Serial,
                    serial_config: Some(serial_config),
                    tcp_config: None,
                    created_at: now,
                    updated_at: now,
                })
            },
            "tcp" => {
                let host = self.host
                    .ok_or_else(|| "ホストが指定されていません".to_string())?;
                let port = self.port
                    .ok_or_else(|| "ポートが指定されていません".to_string())?;
                
                let tcp_config = TcpConfig {
                    host,
                    port,
                    timeout: Duration::from_secs(5),
                    keep_alive: true,
                };
                
                Ok(ConnectionConfig {
                    id: self.id,
                    name: self.name,
                    connection_type: ConnectionType::Tcp,
                    serial_config: None,
                    tcp_config: Some(tcp_config),
                    created_at: now,
                    updated_at: now,
                })
            },
            _ => Err(format!("サポートされていない接続タイプです: {}", self.connection_type)),
        }
    }
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
    config: FrontendConnectionConfig,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<ApiResponse<String>, String> {
    info!("Attempting to connect with config: {:?}", config.name);
    
    // フロントエンドの設定をバックエンド形式に変換
    let backend_config = match config.to_backend_config() {
        Ok(config) => config,
        Err(e) => {
            error!("Invalid configuration: {}", e);
            return Ok(ApiResponse::error(e));
        }
    };
    
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
    match connection_manager.connect(backend_config.clone(), message_tx).await {
        Ok(_) => {
            info!("Successfully connected to device: {}", backend_config.name);
            
            // 接続成功イベントを送信
            let _ = app_handle.emit("connection-status-changed", ("connected", &backend_config.name));
            
            let info = connection_manager.get_connection_info()
                .unwrap_or_else(|| "Connected".to_string());
            
            Ok(ApiResponse::success(info))
        }
        Err(e) => {
            error!("Failed to connect to device {}: {}", backend_config.name, e);
            
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ConnectionType, DataBits, FlowControl, Parity, SerialConfig, StopBits, TcpConfig};
    use chrono::Utc;
    use std::time::Duration;

    fn create_test_serial_connection_config() -> ConnectionConfig {
        ConnectionConfig {
            id: "test-serial".to_string(),
            name: "Test Serial".to_string(),
            connection_type: ConnectionType::Serial,
            serial_config: Some(SerialConfig {
                port: "/dev/ttyUSB0".to_string(),
                baud_rate: 9600,
                data_bits: DataBits::Eight,
                stop_bits: StopBits::One,
                parity: Parity::None,
                flow_control: FlowControl::None,
            }),
            tcp_config: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn create_test_tcp_connection_config() -> ConnectionConfig {
        ConnectionConfig {
            id: "test-tcp".to_string(),
            name: "Test TCP".to_string(),
            connection_type: ConnectionType::Tcp,
            serial_config: None,
            tcp_config: Some(TcpConfig {
                host: "localhost".to_string(),
                port: 8080,
                timeout: Duration::from_secs(5),
                keep_alive: true,
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_app_state_new() {
        let _state = AppState::new();
        
        // 状態が正しく初期化されることを確認
        // 内部フィールドは直接アクセスできないが、構造体の作成は成功する
    }

    #[test]
    fn test_api_response_success() {
        let response = ApiResponse::success("test data".to_string());
        
        assert!(response.success);
        assert_eq!(response.data, Some("test data".to_string()));
        assert_eq!(response.error, None);
    }

    #[test]
    fn test_api_response_error() {
        let response: ApiResponse<String> = ApiResponse::error("test error".to_string());
        
        assert!(!response.success);
        assert_eq!(response.data, None);
        assert_eq!(response.error, Some("test error".to_string()));
    }

    #[test]
    fn test_serial_port_info_creation() {
        let info = SerialPortInfo {
            port_name: "COM1".to_string(),
            port_type: Some("USB".to_string()),
            vid: Some(0x1234),
            pid: Some(0x5678),
            serial_number: Some("SN123".to_string()),
            manufacturer: Some("Test Mfg".to_string()),
            product: Some("Test Product".to_string()),
        };
        
        assert_eq!(info.port_name, "COM1");
        assert_eq!(info.port_type, Some("USB".to_string()));
        assert_eq!(info.vid, Some(0x1234));
        assert_eq!(info.pid, Some(0x5678));
    }

    #[tokio::test]
    async fn test_get_serial_ports() {
        let result = get_serial_ports().await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        
        // レスポンスの構造を確認
        if response.success {
            assert!(response.data.is_some());
            assert!(response.error.is_none());
        } else {
            assert!(response.data.is_none());
            assert!(response.error.is_some());
        }
    }

    #[tokio::test]
    async fn test_get_serial_ports_info() {
        let result = get_serial_ports_info().await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        
        // レスポンスの構造を確認
        if response.success {
            assert!(response.data.is_some());
            assert!(response.error.is_none());
            
            if let Some(ports) = response.data {
                for port in ports {
                    assert!(!port.port_name.is_empty());
                }
            }
        } else {
            assert!(response.data.is_none());
            assert!(response.error.is_some());
        }
    }

    #[test]
    fn test_connection_config_serial() {
        let config = create_test_serial_connection_config();
        
        assert_eq!(config.connection_type, ConnectionType::Serial);
        assert!(config.serial_config.is_some());
        assert!(config.tcp_config.is_none());
        assert_eq!(config.name, "Test Serial");
    }

    #[test]
    fn test_connection_config_tcp() {
        let config = create_test_tcp_connection_config();
        
        assert_eq!(config.connection_type, ConnectionType::Tcp);
        assert!(config.serial_config.is_none());
        assert!(config.tcp_config.is_some());
        assert_eq!(config.name, "Test TCP");
    }

    #[test]
    fn test_connection_error_conversion() {
        let error = ConnectionError::NetworkTimeout;
        let string_error: String = error.into();
        assert_eq!(string_error, "Network timeout");
        
        let error = ConnectionError::PortNotFound("COM1".to_string());
        let string_error: String = error.into();
        assert_eq!(string_error, "Port not found: COM1");
    }

    #[test]
    fn test_api_response_serialization() {
        let response = ApiResponse::success(vec!["port1".to_string(), "port2".to_string()]);
        
        let json = serde_json::to_string(&response);
        assert!(json.is_ok());
        
        let json_str = json.unwrap();
        assert!(json_str.contains("\"success\":true"));
        assert!(json_str.contains("port1"));
        assert!(json_str.contains("port2"));
    }
}