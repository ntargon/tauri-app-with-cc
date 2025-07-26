mod commands;
mod communication;
mod models;
mod services;
mod utils;

use commands::{
    AppState, TerminalState, SettingsState,
    // Connection commands
    get_serial_ports, get_serial_ports_info, connect_device, disconnect_device,
    send_message, get_connection_status, get_connection_info,
    // Terminal commands
    get_terminal_config, update_terminal_config, get_terminal_messages,
    add_terminal_message, clear_terminal_messages, get_command_history,
    add_command_to_history, search_command_history, export_terminal_messages,
    // Settings commands
    get_app_config, update_app_config, get_profiles, add_profile,
    update_profile, delete_profile, get_active_profile, set_active_profile,
    get_recent_profiles, duplicate_profile, export_profiles, import_profiles,
    validate_profile,
};

use tracing_subscriber;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // ログ初期化
    tracing_subscriber::fmt::init();

    // アプリケーション状態を初期化
    let app_state = AppState::new();
    let terminal_state = TerminalState::new();
    let settings_state = SettingsState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .manage(terminal_state)
        .manage(settings_state)
        .invoke_handler(tauri::generate_handler![
            // Connection commands
            get_serial_ports,
            get_serial_ports_info,
            connect_device,
            disconnect_device,
            send_message,
            get_connection_status,
            get_connection_info,
            // Terminal commands
            get_terminal_config,
            update_terminal_config,
            get_terminal_messages,
            add_terminal_message,
            clear_terminal_messages,
            get_command_history,
            add_command_to_history,
            search_command_history,
            export_terminal_messages,
            // Settings commands
            get_app_config,
            update_app_config,
            get_profiles,
            add_profile,
            update_profile,
            delete_profile,
            get_active_profile,
            set_active_profile,
            get_recent_profiles,
            duplicate_profile,
            export_profiles,
            import_profiles,
            validate_profile,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
