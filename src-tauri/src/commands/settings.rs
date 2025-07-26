use crate::models::{AppConfig, ProfileManager, ConnectionConfig};
// use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

use super::ApiResponse;

// 設定状態
pub struct SettingsState {
    pub app_config: Arc<Mutex<AppConfig>>,
    pub profile_manager: Arc<Mutex<ProfileManager>>,
}

impl SettingsState {
    pub fn new() -> Self {
        Self {
            app_config: Arc::new(Mutex::new(AppConfig::default())),
            profile_manager: Arc::new(Mutex::new(ProfileManager::default())),
        }
    }
}

impl Default for SettingsState {
    fn default() -> Self {
        Self::new()
    }
}

// Tauri コマンド

#[tauri::command]
pub async fn get_app_config(
    settings_state: State<'_, SettingsState>,
) -> Result<ApiResponse<AppConfig>, String> {
    let config = settings_state.app_config.lock().await;
    Ok(ApiResponse::success(config.clone()))
}

#[tauri::command]
pub async fn update_app_config(
    config: AppConfig,
    settings_state: State<'_, SettingsState>,
) -> Result<ApiResponse<String>, String> {
    debug!("Updating app config");
    
    let mut current_config = settings_state.app_config.lock().await;
    *current_config = config;
    
    info!("App config updated successfully");
    Ok(ApiResponse::success("App config updated".to_string()))
}

#[tauri::command]
pub async fn get_profiles(
    settings_state: State<'_, SettingsState>,
) -> Result<ApiResponse<Vec<ConnectionConfig>>, String> {
    let profile_manager = settings_state.profile_manager.lock().await;
    Ok(ApiResponse::success(profile_manager.profiles.clone()))
}

#[tauri::command]
pub async fn add_profile(
    profile: ConnectionConfig,
    settings_state: State<'_, SettingsState>,
) -> Result<ApiResponse<String>, String> {
    debug!("Adding profile: {}", profile.name);
    
    let mut profile_manager = settings_state.profile_manager.lock().await;
    profile_manager.add_profile(profile);
    
    info!("Profile added successfully");
    Ok(ApiResponse::success("Profile added".to_string()))
}

#[tauri::command]
pub async fn update_profile(
    profile: ConnectionConfig,
    settings_state: State<'_, SettingsState>,
) -> Result<ApiResponse<String>, String> {
    debug!("Updating profile: {}", profile.name);
    
    let mut profile_manager = settings_state.profile_manager.lock().await;
    
    if let Some(existing_profile) = profile_manager.get_profile_mut(&profile.id) {
        *existing_profile = profile;
        info!("Profile updated successfully");
        Ok(ApiResponse::success("Profile updated".to_string()))
    } else {
        error!("Profile not found: {}", profile.id);
        Ok(ApiResponse::error("Profile not found".to_string()))
    }
}

#[tauri::command]
pub async fn delete_profile(
    profile_id: String,
    settings_state: State<'_, SettingsState>,
) -> Result<ApiResponse<String>, String> {
    debug!("Deleting profile: {}", profile_id);
    
    let mut profile_manager = settings_state.profile_manager.lock().await;
    
    if profile_manager.remove_profile(&profile_id) {
        info!("Profile deleted successfully");
        Ok(ApiResponse::success("Profile deleted".to_string()))
    } else {
        error!("Profile not found: {}", profile_id);
        Ok(ApiResponse::error("Profile not found".to_string()))
    }
}

#[tauri::command]
pub async fn get_active_profile(
    settings_state: State<'_, SettingsState>,
) -> Result<ApiResponse<Option<ConnectionConfig>>, String> {
    let profile_manager = settings_state.profile_manager.lock().await;
    let active_profile = profile_manager.get_active_profile().cloned();
    Ok(ApiResponse::success(active_profile))
}

#[tauri::command]
pub async fn set_active_profile(
    profile_id: String,
    settings_state: State<'_, SettingsState>,
) -> Result<ApiResponse<String>, String> {
    debug!("Setting active profile: {}", profile_id);
    
    let mut profile_manager = settings_state.profile_manager.lock().await;
    
    if profile_manager.get_profile(&profile_id).is_some() {
        profile_manager.set_active_profile(profile_id);
        info!("Active profile set successfully");
        Ok(ApiResponse::success("Active profile set".to_string()))
    } else {
        error!("Profile not found: {}", profile_id);
        Ok(ApiResponse::error("Profile not found".to_string()))
    }
}

#[tauri::command]
pub async fn get_recent_profiles(
    limit: Option<usize>,
    settings_state: State<'_, SettingsState>,
) -> Result<ApiResponse<Vec<ConnectionConfig>>, String> {
    let profile_manager = settings_state.profile_manager.lock().await;
    let limit = limit.unwrap_or(5);
    
    let recent_profiles: Vec<ConnectionConfig> = profile_manager
        .last_used_profiles
        .iter()
        .take(limit)
        .filter_map(|id| profile_manager.get_profile(id))
        .cloned()
        .collect();
    
    Ok(ApiResponse::success(recent_profiles))
}

#[tauri::command]
pub async fn duplicate_profile(
    profile_id: String,
    new_name: String,
    settings_state: State<'_, SettingsState>,
) -> Result<ApiResponse<ConnectionConfig>, String> {
    debug!("Duplicating profile: {} -> {}", profile_id, new_name);
    
    let mut profile_manager = settings_state.profile_manager.lock().await;
    
    if let Some(original_profile) = profile_manager.get_profile(&profile_id) {
        let mut new_profile = original_profile.clone();
        new_profile.id = uuid::Uuid::new_v4().to_string();
        new_profile.name = new_name;
        new_profile.created_at = chrono::Utc::now();
        new_profile.updated_at = chrono::Utc::now();
        
        profile_manager.add_profile(new_profile.clone());
        
        info!("Profile duplicated successfully");
        Ok(ApiResponse::success(new_profile))
    } else {
        error!("Profile not found: {}", profile_id);
        Ok(ApiResponse::error("Profile not found".to_string()))
    }
}

#[tauri::command]
pub async fn export_profiles(
    settings_state: State<'_, SettingsState>,
) -> Result<ApiResponse<String>, String> {
    info!("Exporting profiles");
    
    let profile_manager = settings_state.profile_manager.lock().await;
    
    match serde_json::to_string_pretty(&profile_manager.profiles) {
        Ok(json) => Ok(ApiResponse::success(json)),
        Err(e) => {
            error!("Failed to export profiles: {}", e);
            Ok(ApiResponse::error(format!("Export failed: {}", e)))
        }
    }
}

#[tauri::command]
pub async fn import_profiles(
    profiles_json: String,
    replace_existing: bool,
    settings_state: State<'_, SettingsState>,
) -> Result<ApiResponse<String>, String> {
    info!("Importing profiles (replace_existing: {})", replace_existing);
    
    let imported_profiles: Vec<ConnectionConfig> = match serde_json::from_str(&profiles_json) {
        Ok(profiles) => profiles,
        Err(e) => {
            error!("Failed to parse profiles JSON: {}", e);
            return Ok(ApiResponse::error(format!("Invalid JSON: {}", e)));
        }
    };
    
    let mut profile_manager = settings_state.profile_manager.lock().await;
    
    if replace_existing {
        profile_manager.profiles.clear();
        profile_manager.active_profile_id = None;
        profile_manager.last_used_profiles.clear();
    }
    
    let mut imported_count = 0;
    for mut profile in imported_profiles {
        // 新しいIDを生成
        profile.id = uuid::Uuid::new_v4().to_string();
        profile.created_at = chrono::Utc::now();
        profile.updated_at = chrono::Utc::now();
        
        profile_manager.add_profile(profile);
        imported_count += 1;
    }
    
    info!("Imported {} profiles", imported_count);
    Ok(ApiResponse::success(format!("Imported {} profiles", imported_count)))
}

// プロファイルバリデーション
#[tauri::command]
pub async fn validate_profile(
    profile: ConnectionConfig,
) -> Result<ApiResponse<Vec<String>>, String> {
    debug!("Validating profile: {}", profile.name);
    
    let mut errors = Vec::new();
    
    // 名前チェック
    if profile.name.trim().is_empty() {
        errors.push("プロファイル名を入力してください".to_string());
    }
    
    // 接続設定チェック
    match profile.connection_type {
        crate::models::ConnectionType::Serial => {
            if let Some(serial_config) = &profile.serial_config {
                if serial_config.port.trim().is_empty() {
                    errors.push("シリアルポートを選択してください".to_string());
                }
                if serial_config.baud_rate == 0 {
                    errors.push("有効なボーレートを入力してください".to_string());
                }
            } else {
                errors.push("シリアル設定が見つかりません".to_string());
            }
        }
        crate::models::ConnectionType::Tcp => {
            if let Some(tcp_config) = &profile.tcp_config {
                if tcp_config.host.trim().is_empty() {
                    errors.push("ホストアドレスを入力してください".to_string());
                }
                if tcp_config.port == 0 {
                    errors.push("有効なポート番号（1-65535）を入力してください".to_string());
                }
            } else {
                errors.push("TCP設定が見つかりません".to_string());
            }
        }
    }
    
    Ok(ApiResponse::success(errors))
}