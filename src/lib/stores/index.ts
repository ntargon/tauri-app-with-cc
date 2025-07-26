// メインストアのエクスポート
export { connection, isConnected, isConnecting, hasError, connectionDisplayName } from './connection';
export { terminal, messageCount, filteredMessageCount, hasMessages, hasSelection, selectionCount, isSearchActive, latestMessage } from './terminal';
export { settings, isDirty, currentTheme, isLoggingEnabled, windowConfig, terminalConfig, loggingConfig, securityConfig } from './settings';

// 通知ストア
import { writable } from 'svelte/store';

export interface Notification {
  id: string;
  type: 'info' | 'success' | 'warning' | 'error';
  title: string;
  message: string;
  duration?: number;
  actions?: Array<{
    label: string;
    action: () => void;
  }>;
}

function createNotificationStore() {
  const { subscribe, update } = writable<Notification[]>([]);

  return {
    subscribe,
    
    add(notification: Omit<Notification, 'id'>) {
      const id = crypto.randomUUID();
      const fullNotification: Notification = {
        id,
        duration: 5000,
        ...notification,
      };

      update(notifications => [...notifications, fullNotification]);

      // 自動削除
      if (fullNotification.duration && fullNotification.duration > 0) {
        setTimeout(() => {
          this.remove(id);
        }, fullNotification.duration);
      }

      return id;
    },

    remove(id: string) {
      update(notifications => notifications.filter(n => n.id !== id));
    },

    clear() {
      update(() => []);
    },

    // 便利メソッド
    info(title: string, message: string, options?: Partial<Notification>) {
      return this.add({ type: 'info', title, message, ...options });
    },

    success(title: string, message: string, options?: Partial<Notification>) {
      return this.add({ type: 'success', title, message, ...options });
    },

    warning(title: string, message: string, options?: Partial<Notification>) {
      return this.add({ type: 'warning', title, message, ...options });
    },

    error(title: string, message: string, options?: Partial<Notification>) {
      return this.add({ type: 'error', title, message, duration: 0, ...options });
    },
  };
}

export const notifications = createNotificationStore();

// アプリケーション初期化ストア
function createAppStore() {
  const { subscribe, set, update } = writable({
    isInitialized: false,
    isLoading: true,
    error: null as string | null,
  });

  return {
    subscribe,

    async initialize() {
      update(state => ({ ...state, isLoading: true, error: null }));

      try {
        // 各ストアを初期化
        const { connection } = await import('./connection');
        const { terminal } = await import('./terminal');
        const { settings } = await import('./settings');

        await Promise.all([
          connection.init(),
          terminal.init(),
          settings.init(),
        ]);

        // システムテーマリスナーを設定
        settings.initSystemThemeListener();

        update(state => ({
          ...state,
          isInitialized: true,
          isLoading: false,
        }));

        notifications.success('初期化完了', 'アプリケーションが正常に初期化されました');

      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : '初期化に失敗しました';
        
        update(state => ({
          ...state,
          isLoading: false,
          error: errorMessage,
        }));

        notifications.error('初期化エラー', errorMessage);
        throw error;
      }
    },

    clearError() {
      update(state => ({ ...state, error: null }));
    },
  };
}

export const app = createAppStore();

// 全体的なローディング状態
export const isGlobalLoading = derived(
  [app, connection, settings],
  ([$app, $connection, $settings]) => 
    $app.isLoading || $connection.isLoading || $settings.isLoading
);

// グローバルエラー状態
export const globalError = derived(
  [app, connection, settings],
  ([$app, $connection, $settings]) => 
    $app.error || $connection.error || $settings.error
);