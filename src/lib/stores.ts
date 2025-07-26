import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { AppState, ConnectionConfig, TerminalMessage, ApiResponse } from './types';
import { generateId, validateSerialPort, validateTcpConnection } from './utils';

// 初期状態
const initialState: AppState = {
  connection: {
    isConnected: false,
    isConnecting: false,
    config: null,
    error: null
  },
  messages: [],
  settings: {
    theme: 'auto',
    autoScroll: true,
    showTimestamps: true
  }
};

// メインアプリケーション状態
export const appState = writable<AppState>(initialState);

// UI状態
export const sidebarCollapsed = writable<boolean>(false);
export const currentInput = writable<string>('');
export const inputMode = writable<'text' | 'hex'>('text');
export const availablePorts = writable<string[]>([]);

// イベントリスナー管理
let listenersInitialized = false;
let lastMessageId = '';

// 接続設定フォーム状態
export const connectionForm = writable<ConnectionConfig>({
  id: generateId(),
  name: '新規接続',
  type: 'serial',
  serialPort: '',
  baudRate: 115200,
  host: 'localhost',
  port: 8080
});

// アクション関数群
export const actions = {
  async loadSerialPorts() {
    try {
      const response: ApiResponse<string[]> = await invoke('get_serial_ports');
      if (response.success && response.data) {
        availablePorts.set(response.data);
      }
    } catch (error) {
      console.error('シリアルポート取得エラー:', error);
    }
  },

  async connect() {
    let currentForm: ConnectionConfig;
    connectionForm.subscribe(form => currentForm = form)();

    // 入力検証
    if (currentForm.type === 'serial') {
      if (!validateSerialPort(currentForm.serialPort || '')) {
        appState.update(state => ({
          ...state,
          connection: { ...state.connection, error: 'シリアルポートを選択してください' }
        }));
        return;
      }
    } else {
      if (!validateTcpConnection(currentForm.host || '', currentForm.port || 0)) {
        appState.update(state => ({
          ...state,
          connection: { ...state.connection, error: 'ホストとポートを正しく入力してください' }
        }));
        return;
      }
    }

    appState.update(state => ({
      ...state,
      connection: { ...state.connection, isConnecting: true, error: null }
    }));

    try {
      const response: ApiResponse<string> = await invoke('connect_device', {
        config: currentForm
      });

      if (response.success) {
        appState.update(state => ({
          ...state,
          connection: {
            ...state.connection,
            isConnected: true,
            isConnecting: false,
            config: { ...currentForm }
          }
        }));
        actions.addMessage('info', `${currentForm.name}に接続しました`);
      } else {
        appState.update(state => ({
          ...state,
          connection: {
            ...state.connection,
            isConnecting: false,
            error: response.error || '接続に失敗しました'
          }
        }));
      }
    } catch (error) {
      appState.update(state => ({
        ...state,
        connection: {
          ...state.connection,
          isConnecting: false,
          error: '接続エラーが発生しました'
        }
      }));
      console.error('接続エラー:', error);
    }
  },

  async disconnect() {
    try {
      const response: ApiResponse<string> = await invoke('disconnect_device');
      if (response.success) {
        actions.addMessage('info', '接続を切断しました');
      }
    } catch (error) {
      console.error('切断エラー:', error);
    } finally {
      appState.update(state => ({
        ...state,
        connection: { ...state.connection, isConnected: false, config: null }
      }));
    }
  },

  async sendMessage() {
    let currentState: AppState;
    let input: string;
    let mode: 'text' | 'hex';

    appState.subscribe(state => currentState = state)();
    currentInput.subscribe(value => input = value)();
    inputMode.subscribe(value => mode = value)();

    if (!currentState.connection.isConnected || !input.trim()) return;

    try {
      const message: TerminalMessage = {
        id: generateId(),
        timestamp: new Date().toISOString(),
        direction: 'sent',
        content: input.trim(),
        type: mode
      };

      const response: ApiResponse<string> = await invoke('send_message', {
        message: input.trim(),
        format: mode
      });

      if (response.success) {
        appState.update(state => ({
          ...state,
          messages: [...state.messages, message]
        }));
        currentInput.set('');
      } else {
        actions.addMessage('error', response.error || 'メッセージ送信に失敗しました');
      }
    } catch (error) {
      actions.addMessage('error', 'メッセージ送信エラー');
      console.error('送信エラー:', error);
    }
  },

  addMessage(type: 'info' | 'error', content: string) {
    const message: TerminalMessage = {
      id: generateId(),
      timestamp: new Date().toISOString(),
      direction: 'received',
      content,
      type: 'text'
    };
    appState.update(state => ({
      ...state,
      messages: [...state.messages, message]
    }));
  },

  clearMessages() {
    appState.update(state => ({
      ...state,
      messages: []
    }));
  },

  toggleSidebar() {
    sidebarCollapsed.update(value => !value);
  },

  // Tauriイベントリスナーを初期化
  async initializeEventListeners() {
    if (listenersInitialized) {
      return;
    }
    
    try {
      // 受信メッセージのリスナー
      await listen('terminal-message-received', (event) => {
        const backendMessage = event.payload as any;
        
        // 重複メッセージをチェック
        const messageId = backendMessage.id || generateId();
        if (messageId === lastMessageId) {
          return;
        }
        lastMessageId = messageId;
        
        // バックエンドのメッセージを適切なフォーマットに変換
        const frontendMessage: TerminalMessage = {
          id: messageId,
          timestamp: backendMessage.timestamp || new Date().toISOString(),
          direction: 'received',
          content: backendMessage.content || '',
          type: backendMessage.type || 'text'
        };

        // メッセージをストアに追加
        appState.update(state => ({
          ...state,
          messages: [...state.messages, frontendMessage]
        }));
      });

      // 接続状態変更のリスナー  
      await listen('connection-status-changed', (event) => {
        const [status, info] = event.payload as [string, string];
        
        if (status === 'connected') {
          appState.update(state => ({
            ...state,
            connection: { ...state.connection, isConnected: true, error: null }
          }));
        } else if (status === 'disconnected') {
          appState.update(state => ({
            ...state,
            connection: { ...state.connection, isConnected: false, config: null }
          }));
        } else if (status === 'error') {
          appState.update(state => ({
            ...state,
            connection: { ...state.connection, error: info, isConnecting: false }
          }));
        }
      });

      listenersInitialized = true;
    } catch (error) {
      console.error('イベントリスナー初期化エラー:', error);
    }
  }
};