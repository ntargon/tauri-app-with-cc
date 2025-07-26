use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TerminalMessage {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub direction: MessageDirection,
    pub content: String,
    pub encoding: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum MessageDirection {
    Sent,
    Received,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TerminalConfig {
    pub encoding: String,
    pub line_ending: LineEnding,
    pub echo_input: bool,
    pub show_timestamp: bool,
    pub font_family: String,
    pub font_size: u32,
    pub theme: TerminalTheme,
    pub max_history_size: usize,
    pub auto_scroll: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum LineEnding {
    Cr,    // \r
    Lf,    // \n
    CrLf,  // \r\n
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TerminalTheme {
    pub background_color: String,
    pub text_color: String,
    pub input_color: String,
    pub timestamp_color: String,
    pub sent_color: String,
    pub received_color: String,
    pub error_color: String,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            encoding: "UTF-8".to_string(),
            line_ending: LineEnding::CrLf,
            echo_input: true,
            show_timestamp: true,
            font_family: "Fira Code".to_string(),
            font_size: 14,
            theme: TerminalTheme::default(),
            max_history_size: 1000,
            auto_scroll: true,
        }
    }
}

impl Default for TerminalTheme {
    fn default() -> Self {
        Self {
            background_color: "#1a1b26".to_string(),
            text_color: "#a9b1d6".to_string(),
            input_color: "#24283b".to_string(),
            timestamp_color: "#565f89".to_string(),
            sent_color: "#7aa2f7".to_string(),
            received_color: "#9ece6a".to_string(),
            error_color: "#f7768e".to_string(),
        }
    }
}

impl TerminalMessage {
    pub fn new_sent(content: String, encoding: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            direction: MessageDirection::Sent,
            content,
            encoding,
        }
    }

    pub fn new_received(content: String, encoding: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            direction: MessageDirection::Received,
            content,
            encoding,
        }
    }
}

impl LineEnding {
    #[allow(dead_code)]
    pub fn to_bytes(&self) -> &'static [u8] {
        match self {
            LineEnding::Cr => b"\r",
            LineEnding::Lf => b"\n",
            LineEnding::CrLf => b"\r\n",
        }
    }

    #[allow(dead_code)]
    pub fn to_string(&self) -> &'static str {
        match self {
            LineEnding::Cr => "\r",
            LineEnding::Lf => "\n",
            LineEnding::CrLf => "\r\n",
        }
    }
}

// コマンド履歴管理
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandHistory {
    pub commands: Vec<String>,
    pub max_size: usize,
    pub current_index: Option<usize>,
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            max_size: 100,
            current_index: None,
        }
    }
}

impl CommandHistory {
    pub fn add_command(&mut self, command: String) {
        if !command.trim().is_empty() {
            // 同じコマンドが最後にある場合は追加しない
            if self.commands.last() != Some(&command) {
                self.commands.push(command);
                
                // 最大サイズを超えた場合は古いものを削除
                if self.commands.len() > self.max_size {
                    self.commands.remove(0);
                }
            }
        }
        self.current_index = None;
    }

    #[allow(dead_code)]
    pub fn get_previous(&mut self) -> Option<&String> {
        if self.commands.is_empty() {
            return None;
        }

        match self.current_index {
            None => {
                self.current_index = Some(self.commands.len() - 1);
                self.commands.last()
            }
            Some(index) => {
                if index > 0 {
                    self.current_index = Some(index - 1);
                    self.commands.get(index - 1)
                } else {
                    self.commands.get(index)
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn get_next(&mut self) -> Option<&String> {
        match self.current_index {
            None => None,
            Some(index) => {
                if index < self.commands.len() - 1 {
                    self.current_index = Some(index + 1);
                    self.commands.get(index + 1)
                } else {
                    self.current_index = None;
                    None
                }
            }
        }
    }

    pub fn search(&self, query: &str) -> Vec<&String> {
        self.commands
            .iter()
            .filter(|cmd| cmd.to_lowercase().contains(&query.to_lowercase()))
            .collect()
    }
}