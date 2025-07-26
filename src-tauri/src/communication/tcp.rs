use super::{ConnectionError, ConnectionHandler, ConnectionResult};
use crate::models::{ConnectionConfig, TcpConfig, TerminalMessage};
use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};

pub struct TcpHandler {
    config: TcpConfig,
    stream: Arc<Mutex<Option<TcpStream>>>,
    is_connected: Arc<AtomicBool>,
}

impl TcpHandler {
    pub fn new(config: TcpConfig) -> Self {
        Self {
            config,
            stream: Arc::new(Mutex::new(None)),
            is_connected: Arc::new(AtomicBool::new(false)),
        }
    }

    async fn create_connection(&self) -> ConnectionResult<TcpStream> {
        let address = format!("{}:{}", self.config.host, self.config.port);
        
        debug!("Attempting TCP connection to: {}", address);

        match timeout(self.config.timeout, TcpStream::connect(&address)).await {
            Ok(Ok(stream)) => {
                info!("TCP connection established to: {}", address);
                
                // Keep-alive設定
                if self.config.keep_alive {
                    if let Err(e) = stream.set_nodelay(true) {
                        warn!("Failed to set TCP_NODELAY: {}", e);
                    }
                }
                
                Ok(stream)
            }
            Ok(Err(e)) => {
                let detailed_error = match e.kind() {
                    std::io::ErrorKind::ConnectionRefused => {
                        format!("接続が拒否されました（{}）。サーバーが起動していない可能性があります", address)
                    }
                    std::io::ErrorKind::TimedOut => {
                        format!("接続がタイムアウトしました（{}）。ネットワークまたはファイアウォールの問題の可能性があります", address)
                    }
                    std::io::ErrorKind::NotFound => {
                        format!("ホストが見つかりません（{}）。アドレスを確認してください", address)
                    }
                    std::io::ErrorKind::PermissionDenied => {
                        format!("接続が許可されていません（{}）。ポートアクセス権限を確認してください", address)
                    }
                    _ => {
                        format!("TCP接続エラー（{}）: {}", address, e)
                    }
                };
                error!("{}", detailed_error);
                Err(ConnectionError::IoError(std::io::Error::new(e.kind(), detailed_error)))
            }
            Err(_) => {
                let timeout_error = format!("TCP接続タイムアウト（{}）: {}ms以内に接続できませんでした", address, self.config.timeout.as_millis());
                error!("{}", timeout_error);
                Err(ConnectionError::NetworkTimeout)
            }
        }
    }
}

#[async_trait]
impl ConnectionHandler for TcpHandler {
    async fn connect(&mut self, _config: &ConnectionConfig) -> ConnectionResult<()> {
        info!("開始: TCP接続 - {}:{} (タイムアウト: {}ms, keep-alive: {})", 
              self.config.host, self.config.port, self.config.timeout.as_millis(), self.config.keep_alive);
        debug!("Attempting to connect to TCP: {}:{}", self.config.host, self.config.port);

        // 既存の接続があれば閉じる
        {
            let mut stream_guard = self.stream.lock().await;
            if let Some(mut stream) = stream_guard.take() {
                debug!("既存のTCP接続を切断中: {}:{}", self.config.host, self.config.port);
                let _ = stream.shutdown().await;
            }
        }

        // 新しい接続を作成
        debug!("TCP接続試行中: {}:{}", self.config.host, self.config.port);
        let stream = self.create_connection().await?;
        
        // ストリームを保存
        {
            let mut stream_guard = self.stream.lock().await;
            *stream_guard = Some(stream);
        }
        debug!("TCPストリームをセッションに保存しました");

        // 接続状態を更新
        self.is_connected.store(true, Ordering::SeqCst);

        info!("成功: TCP接続が確立されました - {}:{}", self.config.host, self.config.port);
        Ok(())
    }

    async fn disconnect(&mut self) -> ConnectionResult<()> {
        debug!("Disconnecting from TCP: {}:{}", self.config.host, self.config.port);

        // ストリームを閉じる
        {
            let mut stream_guard = self.stream.lock().await;
            if let Some(mut stream) = stream_guard.take() {
                let _ = stream.flush().await;
                let _ = stream.shutdown().await;
            }
        }

        // 接続状態を更新
        self.is_connected.store(false, Ordering::SeqCst);

        info!("Disconnected from TCP: {}:{}", self.config.host, self.config.port);
        Ok(())
    }

    async fn send(&mut self, data: &[u8]) -> ConnectionResult<()> {
        let mut stream_guard = self.stream.lock().await;
        
        if let Some(stream) = stream_guard.as_mut() {
            match stream.write_all(data).await {
                Ok(_) => {
                    match stream.flush().await {
                        Ok(_) => {
                            debug!("Sent {} bytes to TCP connection", data.len());
                            Ok(())
                        }
                        Err(e) => {
                            error!("Failed to flush TCP stream: {}", e);
                            Err(ConnectionError::SendFailed(e.to_string()))
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to write to TCP stream: {}", e);
                    Err(ConnectionError::SendFailed(e.to_string()))
                }
            }
        } else {
            Err(ConnectionError::ConnectionClosed)
        }
    }

    async fn start_receive_loop(&mut self, tx: mpsc::UnboundedSender<TerminalMessage>) -> ConnectionResult<()> {
        let stream_arc = self.stream.clone();
        let is_connected_arc = self.is_connected.clone();
        let host = self.config.host.clone();
        let port = self.config.port;

        tokio::spawn(async move {
            let mut buffer = [0u8; 1024];
            
            loop {
                // 接続状態をチェック
                if !is_connected_arc.load(Ordering::SeqCst) {
                    debug!("TCP receive loop stopped: not connected");
                    break;
                }

                // データを読み取り
                let result = {
                    let mut stream_guard = stream_arc.lock().await;
                    if let Some(stream) = stream_guard.as_mut() {
                        // タイムアウト付きで読み取り
                        match timeout(Duration::from_millis(100), stream.read(&mut buffer)).await {
                            Ok(Ok(bytes_read)) => Some(Ok(bytes_read)),
                            Ok(Err(e)) => Some(Err(e)),
                            Err(_) => None, // timeout
                        }
                    } else {
                        debug!("TCP receive loop stopped: stream closed");
                        break;
                    }
                };

                match result {
                    Some(Ok(bytes_read)) if bytes_read > 0 => {
                        let data = &buffer[..bytes_read];
                        let content = String::from_utf8_lossy(data).to_string();
                        
                        debug!("Received {} bytes from TCP connection: {:?}", bytes_read, content);
                        
                        let message = TerminalMessage::new_received(content, "UTF-8".to_string());
                        
                        if tx.send(message).is_err() {
                            warn!("Failed to send received message to channel");
                            break;
                        }
                    }
                    Some(Ok(0)) => {
                        // Connection closed by peer
                        info!("TCP connection closed by peer");
                        let message = TerminalMessage::new_received(
                            "Connection closed by peer".to_string(),
                            "UTF-8".to_string()
                        );
                        let _ = tx.send(message);
                        
                        // 接続状態を更新
                        is_connected_arc.store(false, Ordering::SeqCst);
                        break;
                    }
                    Some(Ok(_)) => {
                        // 他のケース（通常は発生しない）
                    }
                    Some(Err(e)) => {
                        match e.kind() {
                            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => {
                                // タイムアウトやWouldBlockは正常、続行
                            }
                            std::io::ErrorKind::ConnectionReset |
                            std::io::ErrorKind::ConnectionAborted |
                            std::io::ErrorKind::UnexpectedEof => {
                                // 接続が切断された
                                info!("TCP connection lost: {}", e);
                                let message = TerminalMessage::new_received(
                                    format!("Connection lost: {}", e),
                                    "UTF-8".to_string()
                                );
                                let _ = tx.send(message);
                                
                                // 接続状態を更新
                                is_connected_arc.store(false, Ordering::SeqCst);
                                break;
                            }
                            _ => {
                                error!("TCP receive error: {}", e);
                                let error_message = TerminalMessage::new_received(
                                    format!("Error: {}", e),
                                    "UTF-8".to_string()
                                );
                                let _ = tx.send(error_message);
                                break;
                            }
                        }
                    }
                    None => {
                        // タイムアウト、続行
                    }
                }

                // 短時間スリープしてCPU使用率を下げる
                tokio::time::sleep(Duration::from_millis(1)).await;
            }

            info!("TCP receive loop ended for {}:{}", host, port);
        });

        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.is_connected.load(Ordering::SeqCst)
    }

    fn get_connection_info(&self) -> Option<String> {
        Some(format!(
            "TCP: {}:{} (timeout: {}ms, keep-alive: {})",
            self.config.host,
            self.config.port,
            self.config.timeout.as_millis(),
            self.config.keep_alive
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn create_test_tcp_config() -> TcpConfig {
        TcpConfig {
            host: "localhost".to_string(),
            port: 8080,
            timeout: Duration::from_secs(5),
            keep_alive: true,
        }
    }

    fn create_test_tcp_config_unreachable() -> TcpConfig {
        TcpConfig {
            host: "192.0.2.1".to_string(), // RFC 5737 - reserved for documentation
            port: 12345,
            timeout: Duration::from_millis(100),
            keep_alive: false,
        }
    }

    #[test]
    fn test_tcp_handler_new() {
        let config = create_test_tcp_config();
        let handler = TcpHandler::new(config.clone());
        
        assert_eq!(handler.config.host, config.host);
        assert_eq!(handler.config.port, config.port);
        assert_eq!(handler.config.timeout, config.timeout);
        assert_eq!(handler.config.keep_alive, config.keep_alive);
    }

    #[test]
    fn test_get_connection_info() {
        let config = create_test_tcp_config();
        let handler = TcpHandler::new(config);
        
        let info = handler.get_connection_info();
        assert!(info.is_some());
        
        let info_str = info.unwrap();
        assert!(info_str.contains("localhost:8080"));
        assert!(info_str.contains("5000ms"));
        assert!(info_str.contains("keep-alive: true"));
    }

    #[test]
    fn test_get_connection_info_no_keep_alive() {
        let config = create_test_tcp_config_unreachable();
        let handler = TcpHandler::new(config);
        
        let info = handler.get_connection_info();
        assert!(info.is_some());
        
        let info_str = info.unwrap();
        assert!(info_str.contains("192.0.2.1:12345"));
        assert!(info_str.contains("100ms"));
        assert!(info_str.contains("keep-alive: false"));
    }

    #[test]
    fn test_is_connected_default() {
        let config = create_test_tcp_config();
        let handler = TcpHandler::new(config);
        
        // 現在の実装では常にtrueを返すが、これは暫定的な実装
        assert!(handler.is_connected());
    }

    #[tokio::test]
    async fn test_connect_to_unreachable_host() {
        let config = create_test_tcp_config_unreachable();
        let mut handler = TcpHandler::new(config.clone());
        
        // ConnectionConfigを作成
        let connection_config = crate::models::ConnectionConfig {
            id: "test".to_string(),
            name: "test".to_string(),
            connection_type: crate::models::ConnectionType::Tcp,
            serial_config: None,
            tcp_config: Some(config),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        let result = handler.connect(&connection_config).await;
        
        // 到達不可能なホストへの接続は失敗する
        assert!(result.is_err());
        
        if let Err(e) = result {
            match e {
                ConnectionError::NetworkTimeout |
                ConnectionError::IoError(_) => {
                    // 期待されるエラー
                }
                _ => panic!("Unexpected error type: {:?}", e),
            }
        }
    }

    #[tokio::test]
    async fn test_send_without_connection() {
        let config = create_test_tcp_config();
        let mut handler = TcpHandler::new(config);
        
        let data = b"test data";
        let result = handler.send(data).await;
        
        // 接続していない状態での送信は失敗する
        assert!(result.is_err());
        
        if let Err(e) = result {
            match e {
                ConnectionError::ConnectionClosed => {
                    // 期待されるエラー
                }
                _ => panic!("Expected ConnectionClosed error, got: {:?}", e),
            }
        }
    }

    #[tokio::test]
    async fn test_disconnect_without_connection() {
        let config = create_test_tcp_config();
        let mut handler = TcpHandler::new(config);
        
        // 接続していない状態での切断は正常に完了する
        let result = handler.disconnect().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_tcp_config_values() {
        let config = TcpConfig {
            host: "example.com".to_string(),
            port: 443,
            timeout: Duration::from_secs(10),
            keep_alive: false,
        };
        
        assert_eq!(config.host, "example.com");
        assert_eq!(config.port, 443);
        assert_eq!(config.timeout, Duration::from_secs(10));
        assert!(!config.keep_alive);
    }

    #[tokio::test]
    async fn test_create_connection_timeout() {
        let config = create_test_tcp_config_unreachable();
        let handler = TcpHandler::new(config);
        
        let result = handler.create_connection().await;
        
        // 到達不可能なホストでは接続がタイムアウトまたは失敗する
        assert!(result.is_err());
        
        if let Err(e) = result {
            match e {
                ConnectionError::NetworkTimeout |
                ConnectionError::IoError(_) => {
                    // 期待されるエラー
                }
                _ => panic!("Unexpected error type: {:?}", e),
            }
        }
    }
}