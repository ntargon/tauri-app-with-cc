// 接続関連の型定義をエクスポート
export * from './connection';

// ターミナル関連の型定義をエクスポート
export * from './terminal';

// 設定関連の型定義をエクスポート
export * from './settings';

// 共通のエラー型
export interface AppError {
  code: string;
  message: string;
  details?: Record<string, unknown>;
}

// API応答の共通型
export interface ApiResponse<T = unknown> {
  success: boolean;
  data?: T;
  error?: AppError;
}

// ファイル関連
export interface FileInfo {
  name: string;
  path: string;
  size: number;
  modified: string;
  is_directory: boolean;
}

// ログエクスポート用
export interface LogExportOptions {
  format: 'txt' | 'csv' | 'json';
  include_timestamp: boolean;
  include_direction: boolean;
  date_range?: {
    start: string;
    end: string;
  };
  message_types?: MessageDirection[];
}

// 通知設定
export interface NotificationConfig {
  show_connection_status: boolean;
  show_errors: boolean;
  show_data_received: boolean;
  sound_enabled: boolean;
  duration_ms: number;
}

// アプリケーション情報
export interface AppInfo {
  name: string;
  version: string;
  description: string;
  author: string;
  license: string;
  repository?: string;
  build_date: string;
  dependencies: Record<string, string>;
}

// 統計情報
export interface AppStatistics {
  sessions_count: number;
  total_messages_sent: number;
  total_messages_received: number;
  total_bytes_sent: number;
  total_bytes_received: number;
  total_connection_time_ms: number;
  most_used_port?: string;
  most_used_baud_rate?: number;
  error_count: number;
  last_session_date?: string;
}

// タブ情報（将来の拡張用）
export interface TabInfo {
  id: string;
  title: string;
  connection_id?: string;
  is_active: boolean;
  is_dirty: boolean;
  created_at: string;
}

// プラグイン情報（将来の拡張用）
export interface PluginInfo {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  enabled: boolean;
  config?: Record<string, unknown>;
}

// キーボードイベント
export interface KeyboardEvent {
  key: string;
  ctrl: boolean;
  shift: boolean;
  alt: boolean;
  meta: boolean;
}

// ドラッグ&ドロップ
export interface DropEvent {
  files: FileInfo[];
  text?: string;
  position: { x: number; y: number };
}

// 検索関連
export interface SearchOptions {
  query: string;
  case_sensitive: boolean;
  regex: boolean;
  whole_word: boolean;
  direction: 'forward' | 'backward';
}

export interface SearchResult {
  message_id: string;
  start_index: number;
  end_index: number;
  match_text: string;
}

// クリップボード操作
export interface ClipboardData {
  text: string;
  html?: string;
  image?: Uint8Array;
}

// ウィンドウ状態
export interface WindowState {
  is_focused: boolean;
  is_minimized: boolean;
  is_maximized: boolean;
  is_fullscreen: boolean;
  bounds: {
    x: number;
    y: number;
    width: number;
    height: number;
  };
}

// システム情報
export interface SystemInfo {
  os: string;
  arch: string;
  platform: string;
  version: string;
  hostname: string;
  total_memory: number;
  available_memory: number;
  cpu_count: number;
  uptime: number;
}

// ユーティリティ型
export type Nullable<T> = T | null;
export type Optional<T> = T | undefined;
export type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};

// イベントハンドラー型
export type EventHandler<T = void> = (data: T) => void | Promise<void>;
export type ErrorHandler = (error: AppError) => void;

// 非同期操作の状態
export interface AsyncState<T> {
  data: T | null;
  loading: boolean;
  error: AppError | null;
}

// コンポーネントのプロパティ型
export interface BaseComponentProps {
  class?: string;
  disabled?: boolean;
  loading?: boolean;
}

export interface FormComponentProps extends BaseComponentProps {
  name?: string;
  required?: boolean;
  readonly?: boolean;
  placeholder?: string;
  error?: string;
}

// カスタムイベント型
export interface CustomEvents {
  connectionStatusChanged: { status: ConnectionStatus; connection?: ConnectionConfig };
  messageReceived: { message: TerminalMessage };
  messageSent: { message: TerminalMessage };
  settingsChanged: { settings: AppConfig };
  profileChanged: { profile: ConnectionConfig };
  errorOccurred: { error: AppError };
  fileDropped: { files: FileInfo[] };
  shortcutTriggered: { shortcut: string; action: string };
}

// Tauri コマンドの戻り値型
export type TauriCommandResult<T> = Promise<ApiResponse<T>>;

// 環境設定
export interface Environment {
  is_dev: boolean;
  is_tauri: boolean;
  platform: string;
  version: string;
}