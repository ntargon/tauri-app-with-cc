import type { ConnectionConfig, TerminalConfig } from './index';

export interface AppConfig {
  version: string;
  terminal: TerminalConfig;
  window: WindowConfig;
  logging: LoggingConfig;
  security: SecurityConfig;
  last_updated: string;
}

export interface WindowConfig {
  width: number;
  height: number;
  x?: number;
  y?: number;
  maximized: boolean;
  always_on_top: boolean;
  theme: AppTheme;
}

export enum AppTheme {
  Light = 'Light',
  Dark = 'Dark',
  System = 'System',
}

export interface LoggingConfig {
  enabled: boolean;
  auto_save: boolean;
  max_file_size_mb: number;
  retention_days: number;
  log_level: LogLevel;
  mask_sensitive_data: boolean;
}

export enum LogLevel {
  Error = 'Error',
  Warn = 'Warn',
  Info = 'Info',
  Debug = 'Debug',
  Trace = 'Trace',
}

export interface SecurityConfig {
  encrypt_passwords: boolean;
  require_confirmation_for_destructive_actions: boolean;
  auto_lock_timeout_minutes?: number;
}

export interface ProfileManager {
  profiles: ConnectionConfig[];
  active_profile_id?: string;
  last_used_profiles: string[];
  groups: Record<string, ProfileGroup>;
}

export interface ProfileGroup {
  id: string;
  name: string;
  description?: string;
  profile_ids: string[];
  color?: string;
}

export interface KeyboardShortcuts {
  send_command: string;
  clear_terminal: string;
  connect: string;
  disconnect: string;
  new_profile: string;
  save_log: string;
  toggle_timestamp: string;
  previous_command: string;
  next_command: string;
}

// UI用の状態
export interface SettingsState {
  appConfig: AppConfig;
  profileManager: ProfileManager;
  shortcuts: KeyboardShortcuts;
  isDirty: boolean;
  lastSaved: string | null;
  isLoading: boolean;
  error: string | null;
}

// 設定画面の各タブ
export enum SettingsTab {
  General = 'general',
  Terminal = 'terminal',
  Logging = 'logging',
  Security = 'security',
  Shortcuts = 'shortcuts',
  Profiles = 'profiles',
  About = 'about',
}

// 設定項目の表示オプション
export interface SettingOption<T = string> {
  value: T;
  label: string;
  description?: string;
  disabled?: boolean;
}

export const THEME_OPTIONS: SettingOption<AppTheme>[] = [
  { value: AppTheme.Light, label: 'ライト', description: '明るいテーマ' },
  { value: AppTheme.Dark, label: 'ダーク', description: '暗いテーマ' },
  { value: AppTheme.System, label: 'システム', description: 'OSの設定に従う' },
];

export const LOG_LEVEL_OPTIONS: SettingOption<LogLevel>[] = [
  { value: LogLevel.Error, label: 'エラー', description: 'エラーのみ記録' },
  { value: LogLevel.Warn, label: '警告', description: '警告以上を記録' },
  { value: LogLevel.Info, label: '情報', description: '情報以上を記録（推奨）' },
  { value: LogLevel.Debug, label: 'デバッグ', description: 'デバッグ情報も記録' },
  { value: LogLevel.Trace, label: 'トレース', description: '全ての情報を記録' },
];

// デフォルト値の生成関数
export function createDefaultAppConfig(): AppConfig {
  return {
    version: '0.1.0',
    terminal: {
      encoding: 'UTF-8',
      line_ending: 'CrLf' as any,
      echo_input: true,
      show_timestamp: true,
      font_family: 'Fira Code',
      font_size: 14,
      theme: {
        background_color: '#1a1b26',
        text_color: '#a9b1d6',
        input_color: '#24283b',
        timestamp_color: '#565f89',
        sent_color: '#7aa2f7',
        received_color: '#9ece6a',
        error_color: '#f7768e',
      },
      max_history_size: 1000,
      auto_scroll: true,
    },
    window: createDefaultWindowConfig(),
    logging: createDefaultLoggingConfig(),
    security: createDefaultSecurityConfig(),
    last_updated: new Date().toISOString(),
  };
}

export function createDefaultWindowConfig(): WindowConfig {
  return {
    width: 1200,
    height: 800,
    maximized: false,
    always_on_top: false,
    theme: AppTheme.System,
  };
}

export function createDefaultLoggingConfig(): LoggingConfig {
  return {
    enabled: true,
    auto_save: true,
    max_file_size_mb: 100,
    retention_days: 30,
    log_level: LogLevel.Info,
    mask_sensitive_data: true,
  };
}

export function createDefaultSecurityConfig(): SecurityConfig {
  return {
    encrypt_passwords: true,
    require_confirmation_for_destructive_actions: true,
  };
}

export function createDefaultProfileManager(): ProfileManager {
  return {
    profiles: [],
    last_used_profiles: [],
    groups: {},
  };
}

export function createDefaultKeyboardShortcuts(): KeyboardShortcuts {
  return {
    send_command: 'Enter',
    clear_terminal: 'Ctrl+L',
    connect: 'Ctrl+O',
    disconnect: 'Ctrl+D',
    new_profile: 'Ctrl+N',
    save_log: 'Ctrl+S',
    toggle_timestamp: 'Ctrl+T',
    previous_command: 'ArrowUp',
    next_command: 'ArrowDown',
  };
}

// バリデーション関数
export function validateAppConfig(config: AppConfig): string[] {
  const errors: string[] = [];
  
  // ウィンドウサイズの検証
  if (config.window.width < 400 || config.window.width > 4000) {
    errors.push('ウィンドウ幅は400-4000の範囲で指定してください');
  }
  
  if (config.window.height < 300 || config.window.height > 3000) {
    errors.push('ウィンドウ高さは300-3000の範囲で指定してください');
  }
  
  // ログ設定の検証
  if (config.logging.max_file_size_mb < 1 || config.logging.max_file_size_mb > 1000) {
    errors.push('ログファイルサイズは1-1000MBの範囲で指定してください');
  }
  
  if (config.logging.retention_days < 1 || config.logging.retention_days > 365) {
    errors.push('ログ保持期間は1-365日の範囲で指定してください');
  }
  
  // セキュリティ設定の検証
  if (
    config.security.auto_lock_timeout_minutes !== undefined &&
    (config.security.auto_lock_timeout_minutes < 1 || config.security.auto_lock_timeout_minutes > 1440)
  ) {
    errors.push('自動ロック時間は1-1440分の範囲で指定してください');
  }
  
  return errors;
}

export function validateProfileGroup(group: ProfileGroup): string[] {
  const errors: string[] = [];
  
  if (!group.name.trim()) {
    errors.push('グループ名を入力してください');
  }
  
  if (group.name.length > 50) {
    errors.push('グループ名は50文字以内で入力してください');
  }
  
  if (group.description && group.description.length > 200) {
    errors.push('説明は200文字以内で入力してください');
  }
  
  // カラーコードの検証
  if (group.color && !/^#[0-9A-Fa-f]{6}$/.test(group.color)) {
    errors.push('有効なカラーコード（#RRGGBB）を入力してください');
  }
  
  return errors;
}

// ユーティリティ関数
export function getThemeDisplayName(theme: AppTheme): string {
  switch (theme) {
    case AppTheme.Light:
      return 'ライト';
    case AppTheme.Dark:
      return 'ダーク';
    case AppTheme.System:
      return 'システム';
    default:
      return 'Unknown';
  }
}

export function getLogLevelDisplayName(level: LogLevel): string {
  switch (level) {
    case LogLevel.Error:
      return 'エラー';
    case LogLevel.Warn:
      return '警告';
    case LogLevel.Info:
      return '情報';
    case LogLevel.Debug:
      return 'デバッグ';
    case LogLevel.Trace:
      return 'トレース';
    default:
      return 'Unknown';
  }
}

export function parseShortcut(shortcut: string): {
  ctrl: boolean;
  shift: boolean;
  alt: boolean;
  key: string;
} {
  const parts = shortcut.split('+');
  return {
    ctrl: parts.includes('Ctrl'),
    shift: parts.includes('Shift'),
    alt: parts.includes('Alt'),
    key: parts[parts.length - 1],
  };
}

export function formatShortcut(shortcut: {
  ctrl: boolean;
  shift: boolean;
  alt: boolean;
  key: string;
}): string {
  const parts: string[] = [];
  if (shortcut.ctrl) parts.push('Ctrl');
  if (shortcut.shift) parts.push('Shift');
  if (shortcut.alt) parts.push('Alt');
  parts.push(shortcut.key);
  return parts.join('+');
}