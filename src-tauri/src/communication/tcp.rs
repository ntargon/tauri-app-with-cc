use super::{ConnectionError, ConnectionHandler, ConnectionResult};
use crate::models::{ConnectionConfig, TcpConfig, TerminalMessage};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};

pub struct TcpHandler {
    config: TcpConfig,
    stream: Arc<Mutex<Option<TcpStream>>>,
    is_connected: Arc<Mutex<bool>>,
}

impl TcpHandler {
    pub fn new(config: TcpConfig) -> Self {
        Self {
            config,
            stream: Arc::new(Mutex::new(None)),
            is_connected: Arc::new(Mutex::new(false)),
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
                error!("Failed to connect to {}: {}", address, e);
                Err(ConnectionError::IoError(e))
            }
            Err(_) => {
                error!("TCP connection timeout to: {}", address);
                Err(ConnectionError::NetworkTimeout)
            }
        }
    }
}

#[async_trait]
impl ConnectionHandler for TcpHandler {
    async fn connect(&mut self, _config: &ConnectionConfig) -> ConnectionResult<()> {
        debug!("Attempting to connect to TCP: {}:{}", self.config.host, self.config.port);

        // 既存の接続があれば閉じる
        {
            let mut stream_guard = self.stream.lock().await;
            if let Some(mut stream) = stream_guard.take() {
                let _ = stream.shutdown().await;
            }
        }

        // 新しい接続を作成
        let stream = self.create_connection().await?;
        
        // ストリームを保存
        {
            let mut stream_guard = self.stream.lock().await;
            *stream_guard = Some(stream);
        }

        // 接続状態を更新
        {
            let mut connected = self.is_connected.lock().await;
            *connected = true;
        }

        info!("Successfully connected to TCP: {}:{}", self.config.host, self.config.port);
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
        {
            let mut connected = self.is_connected.lock().await;
            *connected = false;
        }

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
                {
                    let connected = is_connected_arc.lock().await;
                    if !*connected {
                        debug!("TCP receive loop stopped: not connected");
                        break;
                    }
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
                        {
                            let mut connected = is_connected_arc.lock().await;
                            *connected = false;
                        }
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
                                {
                                    let mut connected = is_connected_arc.lock().await;
                                    *connected = false;
                                }
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
        // Note: この関数は同期的なので、Arcの値を直接チェックできない
        // 実際の実装では、AtomicBoolを使用するか、別の方法を検討する必要がある
        true // 暫定的な実装
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