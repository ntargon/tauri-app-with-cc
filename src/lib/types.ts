// タイプ定義の統合ファイル

export interface ConnectionConfig {
  id: string;
  name: string;
  type: 'serial' | 'tcp';
  serialPort?: string;
  baudRate?: number;
  host?: string;
  port?: number;
}

export interface TerminalMessage {
  id: string;
  timestamp: string;
  direction: 'sent' | 'received';
  content: string;
  type: 'text' | 'hex';
}

export interface ConnectionState {
  isConnected: boolean;
  isConnecting: boolean;
  config: ConnectionConfig | null;
  error: string | null;
}

export interface AppState {
  connection: ConnectionState;
  messages: TerminalMessage[];
  settings: {
    theme: 'light' | 'dark' | 'auto';
    autoScroll: boolean;
    showTimestamps: boolean;
  };
}

// Tauri API レスポンス型
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}