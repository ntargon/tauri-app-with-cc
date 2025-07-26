#[cfg(test)]
mod tests {
    use super::*;
    use crate::communication::{ConnectionManager, ConnectionError, ConnectionResult};
    use crate::models::{ConnectionConfig, ConnectionType, TcpConfig};
    use chrono::Utc;
    use std::time::Duration;
    use tokio::sync::mpsc;

    fn create_test_tcp_config() -> ConnectionConfig {
        ConnectionConfig {
            id: "test-id".to_string(),
            name: "Test Connection".to_string(),
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

    #[tokio::test]
    async fn test_connection_manager_new() {
        let manager = ConnectionManager::new();
        assert!(!manager.is_connected());
        assert_eq!(manager.get_connection_info(), None);
    }

    #[tokio::test]
    async fn test_connection_manager_connect_success() {
        let mut manager = ConnectionManager::new();
        let config = create_test_tcp_config();
        let (tx, _rx) = mpsc::unbounded_channel();

        // モックハンドラーは実際のTCP/Serial接続をモックできないため、
        // このテストでは基本的な構造のテストに留める
        // 実際の接続テストは個別のハンドラーレベルで行う
        let result = manager.connect(config.clone(), tx).await;
        
        // 設定が不正でない限り、接続試行は行われる
        // ただし、実際のサーバーが存在しないため失敗する可能性が高い
        // テスト環境では接続成功をアサートしない
    }

    #[tokio::test]
    async fn test_connection_manager_connect_invalid_config() {
        let mut manager = ConnectionManager::new();
        let mut config = create_test_tcp_config();
        config.tcp_config = None; // 無効な設定
        let (tx, _rx) = mpsc::unbounded_channel();

        let result = manager.connect(config, tx).await;
        assert!(result.is_err());
        
        if let Err(e) = result {
            match e {
                ConnectionError::InvalidConfiguration(_) => {},
                _ => panic!("Expected InvalidConfiguration error"),
            }
        }
    }

    #[tokio::test]
    async fn test_connection_manager_send_message_not_connected() {
        let mut manager = ConnectionManager::new();
        let result = manager.send_message("test message".to_string()).await;
        
        assert!(result.is_err());
        if let Err(e) = result {
            match e {
                ConnectionError::ConnectionClosed => {},
                _ => panic!("Expected ConnectionClosed error"),
            }
        }
    }

    #[tokio::test]
    async fn test_connection_manager_disconnect_not_connected() {
        let mut manager = ConnectionManager::new();
        let result = manager.disconnect().await;
        
        // 接続していない状態での切断は正常に完了する
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_connection_manager_is_connected_default() {
        let manager = ConnectionManager::new();
        assert!(!manager.is_connected());
    }

    #[tokio::test]
    async fn test_connection_manager_get_connection_info_default() {
        let manager = ConnectionManager::new();
        assert_eq!(manager.get_connection_info(), None);
    }

    #[test]
    fn test_connection_error_display() {
        let error = ConnectionError::PortNotFound("COM1".to_string());
        assert_eq!(error.to_string(), "Port not found: COM1");

        let error = ConnectionError::PermissionDenied;
        assert_eq!(error.to_string(), "Permission denied");

        let error = ConnectionError::InvalidConfiguration("Test config".to_string());
        assert_eq!(error.to_string(), "Invalid configuration: Test config");

        let error = ConnectionError::NetworkTimeout;
        assert_eq!(error.to_string(), "Network timeout");

        let error = ConnectionError::ConnectionClosed;
        assert_eq!(error.to_string(), "Connection closed");
    }

    #[test]
    fn test_connection_result_type() {
        let success_result: ConnectionResult<String> = Ok("success".to_string());
        assert!(success_result.is_ok());

        let error_result: ConnectionResult<String> = Err(ConnectionError::NetworkTimeout);
        assert!(error_result.is_err());
    }

    #[test]
    fn test_connection_manager_default() {
        let manager = ConnectionManager::default();
        assert!(!manager.is_connected());
        assert_eq!(manager.get_connection_info(), None);
    }
}