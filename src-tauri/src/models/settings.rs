use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{ConnectionConfig, TerminalConfig};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub version: String,
    pub terminal: TerminalConfig,
    pub window: WindowConfig,
    pub logging: LoggingConfig,
    pub security: SecurityConfig,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub maximized: bool,
    pub always_on_top: bool,
    pub theme: AppTheme,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AppTheme {
    Light,
    Dark,
    System,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggingConfig {
    pub enabled: bool,
    pub auto_save: bool,
    pub max_file_size_mb: u64,
    pub retention_days: u32,
    pub log_level: LogLevel,
    pub mask_sensitive_data: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecurityConfig {
    pub encrypt_passwords: bool,
    pub require_confirmation_for_destructive_actions: bool,
    pub auto_lock_timeout_minutes: Option<u32>,
}

// プロファイル管理
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProfileManager {
    pub profiles: Vec<ConnectionConfig>,
    pub active_profile_id: Option<String>,
    pub last_used_profiles: Vec<String>, // 最近使用したプロファイルID
    pub groups: HashMap<String, ProfileGroup>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProfileGroup {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub profile_ids: Vec<String>,
    pub color: Option<String>,
}

// キーボードショートカット
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyboardShortcuts {
    pub send_command: String,
    pub clear_terminal: String,
    pub connect: String,
    pub disconnect: String,
    pub new_profile: String,
    pub save_log: String,
    pub toggle_timestamp: String,
    pub previous_command: String,
    pub next_command: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            terminal: TerminalConfig::default(),
            window: WindowConfig::default(),
            logging: LoggingConfig::default(),
            security: SecurityConfig::default(),
            last_updated: Utc::now(),
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 1200,
            height: 800,
            x: None,
            y: None,
            maximized: false,
            always_on_top: false,
            theme: AppTheme::System,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_save: true,
            max_file_size_mb: 100,
            retention_days: 30,
            log_level: LogLevel::Info,
            mask_sensitive_data: true,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            encrypt_passwords: true,
            require_confirmation_for_destructive_actions: true,
            auto_lock_timeout_minutes: None,
        }
    }
}

impl Default for ProfileManager {
    fn default() -> Self {
        Self {
            profiles: Vec::new(),
            active_profile_id: None,
            last_used_profiles: Vec::new(),
            groups: HashMap::new(),
        }
    }
}

impl Default for KeyboardShortcuts {
    fn default() -> Self {
        Self {
            send_command: "Enter".to_string(),
            clear_terminal: "Ctrl+L".to_string(),
            connect: "Ctrl+O".to_string(),
            disconnect: "Ctrl+D".to_string(),
            new_profile: "Ctrl+N".to_string(),
            save_log: "Ctrl+S".to_string(),
            toggle_timestamp: "Ctrl+T".to_string(),
            previous_command: "ArrowUp".to_string(),
            next_command: "ArrowDown".to_string(),
        }
    }
}

impl ProfileManager {
    pub fn add_profile(&mut self, profile: ConnectionConfig) {
        self.profiles.push(profile);
    }

    pub fn remove_profile(&mut self, profile_id: &str) -> bool {
        let original_len = self.profiles.len();
        self.profiles.retain(|p| p.id != profile_id);
        
        // アクティブプロファイルが削除された場合はクリア
        if self.active_profile_id.as_ref() == Some(&profile_id.to_string()) {
            self.active_profile_id = None;
        }
        
        // 最近使用したリストからも削除
        self.last_used_profiles.retain(|id| id != profile_id);
        
        self.profiles.len() < original_len
    }

    pub fn get_profile(&self, profile_id: &str) -> Option<&ConnectionConfig> {
        self.profiles.iter().find(|p| p.id == profile_id)
    }

    pub fn get_profile_mut(&mut self, profile_id: &str) -> Option<&mut ConnectionConfig> {
        self.profiles.iter_mut().find(|p| p.id == profile_id)
    }

    pub fn set_active_profile(&mut self, profile_id: String) {
        self.active_profile_id = Some(profile_id.clone());
        
        // 最近使用したリストの先頭に移動
        self.last_used_profiles.retain(|id| id != &profile_id);
        self.last_used_profiles.insert(0, profile_id);
        
        // 最大10個まで保持
        if self.last_used_profiles.len() > 10 {
            self.last_used_profiles.truncate(10);
        }
    }

    pub fn get_active_profile(&self) -> Option<&ConnectionConfig> {
        self.active_profile_id
            .as_ref()
            .and_then(|id| self.get_profile(id))
    }

    pub fn get_profiles_by_group(&self, group_id: &str) -> Vec<&ConnectionConfig> {
        if let Some(group) = self.groups.get(group_id) {
            group
                .profile_ids
                .iter()
                .filter_map(|id| self.get_profile(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn add_group(&mut self, group: ProfileGroup) {
        self.groups.insert(group.id.clone(), group);
    }

    pub fn remove_group(&mut self, group_id: &str) -> bool {
        self.groups.remove(group_id).is_some()
    }
}