pub mod serial;
pub mod tcp;
#[cfg(test)]
mod tests;

use crate::models::{ConnectionConfig, TerminalMessage};
use async_trait::async_trait;
// use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc;
#[cfg(test)]
use mockall::automock;

pub use serial::SerialHandler;
pub use tcp::TcpHandler;

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("Port not found: {0}")]
    PortNotFound(String),
    
    #[error("Permission denied")]
    PermissionDenied,
    
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    
    #[error("Network timeout")]
    NetworkTimeout,
    
    #[error("Send failed: {0}")]
    SendFailed(String),
    
    #[error("Receive failed: {0}")]
    ReceiveFailed(String),
    
    #[error("Connection closed")]
    ConnectionClosed,
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serial port error: {0}")]
    SerialError(#[from] serialport::Error),
}

pub type ConnectionResult<T> = Result<T, ConnectionError>;

#[async_trait]
#[cfg_attr(test, automock)]
pub trait ConnectionHandler: Send + Sync {
    async fn connect(&mut self, config: &ConnectionConfig) -> ConnectionResult<()>;
    async fn disconnect(&mut self) -> ConnectionResult<()>;
    async fn send(&mut self, data: &[u8]) -> ConnectionResult<()>;
    async fn start_receive_loop(&mut self, tx: mpsc::UnboundedSender<TerminalMessage>) -> ConnectionResult<()>;
    fn is_connected(&self) -> bool;
    fn get_connection_info(&self) -> Option<String>;
}

pub struct ConnectionManager {
    current_handler: Option<Box<dyn ConnectionHandler>>,
    message_sender: Option<mpsc::UnboundedSender<TerminalMessage>>,
    receive_handle: Option<tokio::task::JoinHandle<()>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            current_handler: None,
            message_sender: None,
            receive_handle: None,
        }
    }

    pub async fn connect(&mut self, config: ConnectionConfig, message_tx: mpsc::UnboundedSender<TerminalMessage>) -> ConnectionResult<()> {
        // 既存の接続があれば切断
        if let Some(handler) = &mut self.current_handler {
            let _ = handler.disconnect().await;
        }

        // 受信タスクがあれば停止
        if let Some(handle) = self.receive_handle.take() {
            handle.abort();
        }

        // 新しいハンドラーを作成
        let mut handler: Box<dyn ConnectionHandler> = match config.connection_type {
            crate::models::ConnectionType::Serial => {
                if let Some(serial_config) = &config.serial_config {
                    Box::new(SerialHandler::new(serial_config.clone()))
                } else {
                    return Err(ConnectionError::InvalidConfiguration("Serial config is missing".to_string()));
                }
            }
            crate::models::ConnectionType::Tcp => {
                if let Some(tcp_config) = &config.tcp_config {
                    Box::new(TcpHandler::new(tcp_config.clone()))
                } else {
                    return Err(ConnectionError::InvalidConfiguration("TCP config is missing".to_string()));
                }
            }
        };

        // 接続実行
        handler.connect(&config).await?;

        // 受信ループ開始
        let message_tx_clone = message_tx.clone();
        handler.start_receive_loop(message_tx_clone).await?;

        self.current_handler = Some(handler);
        self.message_sender = Some(message_tx);

        Ok(())
    }

    pub async fn disconnect(&mut self) -> ConnectionResult<()> {
        if let Some(handler) = &mut self.current_handler {
            handler.disconnect().await?;
        }

        if let Some(handle) = self.receive_handle.take() {
            handle.abort();
        }

        self.current_handler = None;
        self.message_sender = None;

        Ok(())
    }

    pub async fn send_message(&mut self, message: String) -> ConnectionResult<()> {
        if let Some(handler) = &mut self.current_handler {
            let data = message.as_bytes();
            handler.send(data).await?;
            
            // 送信メッセージはフロントエンドで既に表示しているため、
            // バックエンドでは受信メッセージのみをチャンネルに送信する
            
            Ok(())
        } else {
            Err(ConnectionError::ConnectionClosed)
        }
    }

    pub fn is_connected(&self) -> bool {
        self.current_handler
            .as_ref()
            .map(|h| h.is_connected())
            .unwrap_or(false)
    }

    pub fn get_connection_info(&self) -> Option<String> {
        self.current_handler
            .as_ref()
            .and_then(|h| h.get_connection_info())
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}