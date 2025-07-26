use crate::models::{TerminalConfig, TerminalMessage, CommandHistory};
use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

use super::ApiResponse;

// ターミナル状態
pub struct TerminalState {
    pub config: Arc<Mutex<TerminalConfig>>,
    pub messages: Arc<Mutex<Vec<TerminalMessage>>>,
    pub command_history: Arc<Mutex<CommandHistory>>,
}

impl TerminalState {
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(TerminalConfig::default())),
            messages: Arc::new(Mutex::new(Vec::new())),
            command_history: Arc::new(Mutex::new(CommandHistory::default())),
        }
    }
}

impl Default for TerminalState {
    fn default() -> Self {
        Self::new()
    }
}

// メッセージフィルター
#[derive(Debug, Serialize, Deserialize)]
pub struct MessageFilter {
    pub direction: Option<String>, // "sent" | "received"
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub search_query: Option<String>,
    pub limit: Option<usize>,
}

// エクスポートオプション
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportOptions {
    pub format: String, // "txt" | "csv" | "json"
    pub include_timestamp: bool,
    pub include_direction: bool,
    pub filter: Option<MessageFilter>,
}

// Tauri コマンド

#[tauri::command]
pub async fn get_terminal_config(
    terminal_state: State<'_, TerminalState>,
) -> Result<ApiResponse<TerminalConfig>, String> {
    let config = terminal_state.config.lock().await;
    Ok(ApiResponse::success(config.clone()))
}

#[tauri::command]
pub async fn update_terminal_config(
    config: TerminalConfig,
    terminal_state: State<'_, TerminalState>,
) -> Result<ApiResponse<String>, String> {
    debug!("Updating terminal config");
    
    let mut current_config = terminal_state.config.lock().await;
    *current_config = config;
    
    info!("Terminal config updated successfully");
    Ok(ApiResponse::success("Terminal config updated".to_string()))
}

#[tauri::command]
pub async fn get_terminal_messages(
    filter: Option<MessageFilter>,
    terminal_state: State<'_, TerminalState>,
) -> Result<ApiResponse<Vec<TerminalMessage>>, String> {
    debug!("Getting terminal messages with filter: {:?}", filter);
    
    let messages = terminal_state.messages.lock().await;
    let mut filtered_messages = messages.clone();
    
    // フィルタリング適用
    if let Some(filter) = filter {
        // 方向フィルター
        if let Some(direction) = &filter.direction {
            filtered_messages.retain(|msg| {
                match direction.as_str() {
                    "sent" => matches!(msg.direction, crate::models::MessageDirection::Sent),
                    "received" => matches!(msg.direction, crate::models::MessageDirection::Received),
                    _ => true,
                }
            });
        }
        
        // 検索クエリフィルター
        if let Some(query) = &filter.search_query {
            let query_lower = query.to_lowercase();
            filtered_messages.retain(|msg| {
                msg.content.to_lowercase().contains(&query_lower)
            });
        }
        
        // 時間範囲フィルター（簡易実装）
        if filter.start_time.is_some() || filter.end_time.is_some() {
            // 実装が必要な場合は chrono を使用して時間比較
        }
        
        // リミット適用
        if let Some(limit) = filter.limit {
            if filtered_messages.len() > limit {
                let start_index = filtered_messages.len() - limit;
                filtered_messages = filtered_messages[start_index..].to_vec();
            }
        }
    }
    
    debug!("Returning {} filtered messages", filtered_messages.len());
    Ok(ApiResponse::success(filtered_messages))
}

#[tauri::command]
pub async fn add_terminal_message(
    message: TerminalMessage,
    terminal_state: State<'_, TerminalState>,
) -> Result<ApiResponse<String>, String> {
    debug!("Adding terminal message: {:?}", message.id);
    
    let mut messages = terminal_state.messages.lock().await;
    let config = terminal_state.config.lock().await;
    
    // メッセージを追加
    messages.push(message);
    
    // 最大履歴サイズを超えた場合は古いものを削除
    if messages.len() > config.max_history_size {
        let remove_count = messages.len() - config.max_history_size;
        messages.drain(0..remove_count);
    }
    
    Ok(ApiResponse::success("Message added".to_string()))
}

#[tauri::command]
pub async fn clear_terminal_messages(
    terminal_state: State<'_, TerminalState>,
) -> Result<ApiResponse<String>, String> {
    info!("Clearing terminal messages");
    
    let mut messages = terminal_state.messages.lock().await;
    messages.clear();
    
    Ok(ApiResponse::success("Messages cleared".to_string()))
}

#[tauri::command]
pub async fn get_command_history(
    terminal_state: State<'_, TerminalState>,
) -> Result<ApiResponse<Vec<String>>, String> {
    let history = terminal_state.command_history.lock().await;
    Ok(ApiResponse::success(history.commands.clone()))
}

#[tauri::command]
pub async fn add_command_to_history(
    command: String,
    terminal_state: State<'_, TerminalState>,
) -> Result<ApiResponse<String>, String> {
    debug!("Adding command to history: {}", command);
    
    let mut history = terminal_state.command_history.lock().await;
    history.add_command(command);
    
    Ok(ApiResponse::success("Command added to history".to_string()))
}

#[tauri::command]
pub async fn search_command_history(
    query: String,
    terminal_state: State<'_, TerminalState>,
) -> Result<ApiResponse<Vec<String>>, String> {
    debug!("Searching command history: {}", query);
    
    let history = terminal_state.command_history.lock().await;
    let results: Vec<String> = history.search(&query)
        .into_iter()
        .cloned()
        .collect();
    
    Ok(ApiResponse::success(results))
}

#[tauri::command]
pub async fn export_terminal_messages(
    options: ExportOptions,
    terminal_state: State<'_, TerminalState>,
) -> Result<ApiResponse<String>, String> {
    info!("Exporting terminal messages with format: {}", options.format);
    
    let messages = terminal_state.messages.lock().await;
    let mut export_messages = messages.clone();
    
    // フィルター適用
    if let Some(filter) = &options.filter {
        if let Some(direction) = &filter.direction {
            export_messages.retain(|msg| {
                match direction.as_str() {
                    "sent" => matches!(msg.direction, crate::models::MessageDirection::Sent),
                    "received" => matches!(msg.direction, crate::models::MessageDirection::Received),
                    _ => true,
                }
            });
        }
        
        if let Some(query) = &filter.search_query {
            let query_lower = query.to_lowercase();
            export_messages.retain(|msg| {
                msg.content.to_lowercase().contains(&query_lower)
            });
        }
    }
    
    // フォーマットに応じてエクスポート
    let exported_data = match options.format.as_str() {
        "txt" => export_as_text(&export_messages, &options),
        "csv" => export_as_csv(&export_messages, &options),
        "json" => export_as_json(&export_messages),
        _ => return Ok(ApiResponse::error("Unsupported export format".to_string())),
    };
    
    match exported_data {
        Ok(data) => Ok(ApiResponse::success(data)),
        Err(e) => Ok(ApiResponse::error(e)),
    }
}

// エクスポート関数

fn export_as_text(messages: &[TerminalMessage], options: &ExportOptions) -> Result<String, String> {
    let mut result = String::new();
    
    for message in messages {
        let mut line = String::new();
        
        if options.include_timestamp {
            line.push_str(&format!("[{}] ", message.timestamp));
        }
        
        if options.include_direction {
            let direction = match message.direction {
                crate::models::MessageDirection::Sent => "送信",
                crate::models::MessageDirection::Received => "受信",
            };
            line.push_str(&format!("{}: ", direction));
        }
        
        line.push_str(&message.content);
        line.push('\n');
        
        result.push_str(&line);
    }
    
    Ok(result)
}

fn export_as_csv(messages: &[TerminalMessage], options: &ExportOptions) -> Result<String, String> {
    let mut result = String::new();
    
    // ヘッダー
    let mut headers = Vec::new();
    if options.include_timestamp {
        headers.push("タイムスタンプ");
    }
    if options.include_direction {
        headers.push("方向");
    }
    headers.push("内容");
    headers.push("エンコーディング");
    
    result.push_str(&headers.join(","));
    result.push('\n');
    
    // データ
    for message in messages {
        let mut row: Vec<String> = Vec::new();
        
        if options.include_timestamp {
            row.push(message.timestamp.format("%Y-%m-%d %H:%M:%S%.3f UTC").to_string());
        }
        
        if options.include_direction {
            let direction = match message.direction {
                crate::models::MessageDirection::Sent => "送信",
                crate::models::MessageDirection::Received => "受信",
            };
            row.push(direction.to_string());
        }
        
        // CSVエスケープ
        let escaped_content = message.content.replace("\"", "\"\"");
        row.push(format!("\"{}\"", escaped_content));
        row.push(message.encoding.clone());
        
        result.push_str(&row.join(","));
        result.push('\n');
    }
    
    Ok(result)
}

fn export_as_json(messages: &[TerminalMessage]) -> Result<String, String> {
    match serde_json::to_string_pretty(messages) {
        Ok(json) => Ok(json),
        Err(e) => Err(format!("JSON serialization error: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{TerminalConfig, TerminalMessage, MessageDirection, TerminalTheme, LineEnding};
    use chrono::Utc;

    fn create_test_terminal_state() -> TerminalState {
        TerminalState::new()
    }

    fn create_test_message(content: &str, direction: MessageDirection) -> TerminalMessage {
        TerminalMessage {
            id: uuid::Uuid::new_v4().to_string(),
            content: content.to_string(),
            direction,
            timestamp: Utc::now(),
            encoding: "UTF-8".to_string(),
        }
    }

    fn create_test_terminal_config() -> TerminalConfig {
        TerminalConfig {
            encoding: "UTF-8".to_string(),
            line_ending: LineEnding::CrLf,
            echo_input: true,
            show_timestamp: true,
            font_family: "monospace".to_string(),
            font_size: 12,
            theme: TerminalTheme {
                background_color: "#000000".to_string(),
                text_color: "#ffffff".to_string(),
                input_color: "#00ff00".to_string(),
                timestamp_color: "#888888".to_string(),
                sent_color: "#0088ff".to_string(),
                received_color: "#ffaa00".to_string(),
                error_color: "#ff0000".to_string(),
            },
            max_history_size: 1000,
            auto_scroll: true,
        }
    }

    #[test]
    fn test_terminal_state_new() {
        let _state = create_test_terminal_state();
        // 状態が正しく初期化されることを確認
    }

    #[test]
    fn test_terminal_state_default() {
        let _state = TerminalState::default();
        // デフォルト実装が機能することを確認
    }

    #[test]
    fn test_message_filter_creation() {
        let filter = MessageFilter {
            direction: Some("sent".to_string()),
            start_time: Some("2024-01-01T00:00:00Z".to_string()),
            end_time: Some("2024-12-31T23:59:59Z".to_string()),
            search_query: Some("test".to_string()),
            limit: Some(100),
        };
        
        assert_eq!(filter.direction, Some("sent".to_string()));
        assert_eq!(filter.limit, Some(100));
    }

    #[test]
    fn test_export_options_creation() {
        let options = ExportOptions {
            format: "json".to_string(),
            include_timestamp: true,
            include_direction: true,
            filter: None,
        };
        
        assert_eq!(options.format, "json");
        assert!(options.include_timestamp);
        assert!(options.include_direction);
        assert!(options.filter.is_none());
    }

    #[tokio::test]
    async fn test_terminal_config_update() {
        let state = create_test_terminal_state();
        let new_config = create_test_terminal_config();
        
        // 設定を更新
        {
            let mut config = state.config.lock().await;
            *config = new_config.clone();
        }
        
        // 設定が更新されたことを確認
        let config = state.config.lock().await;
        assert_eq!(config.max_history_size, 1000);
        assert_eq!(config.auto_scroll, true);
        assert_eq!(config.font_size, 12);
        assert_eq!(config.encoding, "UTF-8");
    }

    #[tokio::test]
    async fn test_terminal_message_management() {
        let state = create_test_terminal_state();
        
        // メッセージを直接追加してテスト
        let message1 = create_test_message("Hello", MessageDirection::Sent);
        let message2 = create_test_message("World", MessageDirection::Received);
        
        {
            let mut messages = state.messages.lock().await;
            messages.push(message1.clone());
            messages.push(message2.clone());
        }
        
        // メッセージを取得
        let messages = state.messages.lock().await;
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].content, "Hello");
        assert_eq!(messages[1].content, "World");
        assert_eq!(messages[0].direction, MessageDirection::Sent);
        assert_eq!(messages[1].direction, MessageDirection::Received);
    }

    #[test]
    fn test_export_as_text() {
        let messages = vec![
            create_test_message("Hello", MessageDirection::Sent),
            create_test_message("World", MessageDirection::Received),
        ];
        
        let options = ExportOptions {
            format: "txt".to_string(),
            include_timestamp: false,
            include_direction: true,
            filter: None,
        };
        
        let result = export_as_text(&messages, &options);
        assert!(result.is_ok());
        
        let text = result.unwrap();
        assert!(text.contains("送信: Hello"));
        assert!(text.contains("受信: World"));
    }

    #[test]
    fn test_export_as_json() {
        let messages = vec![
            create_test_message("Hello", MessageDirection::Sent),
        ];
        
        let result = export_as_json(&messages);
        assert!(result.is_ok());
        
        let json = result.unwrap();
        assert!(json.contains("\"content\": \"Hello\""));
        assert!(json.contains("\"direction\": \"Sent\""));
    }
}