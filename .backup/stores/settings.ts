import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type {
  AppConfig,
  SettingsState,
  AppTheme,
  WindowConfig,
  LoggingConfig,
  SecurityConfig,
  ApiResponse,
} from '$lib/types';

// 設定状態ストア
function createSettingsStore() {
  const { subscribe, set, update } = writable<SettingsState>({
    appConfig: {
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
      window: {
        width: 1200,
        height: 800,
        maximized: false,
        always_on_top: false,
        theme: 'System' as AppTheme,
      },
      logging: {
        enabled: true,
        auto_save: true,
        max_file_size_mb: 100,
        retention_days: 30,
        log_level: 'Info' as any,
        mask_sensitive_data: true,
      },
      security: {
        encrypt_passwords: true,
        require_confirmation_for_destructive_actions: true,
      },
      last_updated: new Date().toISOString(),
    },
    profileManager: {
      profiles: [],
      last_used_profiles: [],
      groups: {},
    },
    shortcuts: {
      send_command: 'Enter',
      clear_terminal: 'Ctrl+L',
      connect: 'Ctrl+O',
      disconnect: 'Ctrl+D',
      new_profile: 'Ctrl+N',
      save_log: 'Ctrl+S',
      toggle_timestamp: 'Ctrl+T',
      previous_command: 'ArrowUp',
      next_command: 'ArrowDown',
    },
    isDirty: false,
    lastSaved: null,
    isLoading: false,
    error: null,
  });

  return {
    subscribe,
    set,
    update,

    // 初期化
    async init() {
      await this.loadAppConfig();
    },

    // アプリ設定管理
    async loadAppConfig() {
      update((state) => ({ ...state, isLoading: true, error: null }));

      try {
        const response: ApiResponse<AppConfig> = await invoke('get_app_config');
        
        if (response.success && response.data) {
          update((state) => ({
            ...state,
            appConfig: response.data!,
            isLoading: false,
            isDirty: false,
          }));
        } else {
          throw new Error(response.error || 'Failed to load app config');
        }
      } catch (error) {
        console.error('Error loading app config:', error);
        update((state) => ({
          ...state,
          isLoading: false,
          error: error instanceof Error ? error.message : 'Unknown error',
        }));
      }
    },

    async saveAppConfig() {
      const currentState = get({ subscribe });
      
      update((state) => ({ ...state, isLoading: true, error: null }));

      try {
        const configToSave = {
          ...currentState.appConfig,
          last_updated: new Date().toISOString(),
        };

        const response: ApiResponse<string> = await invoke('update_app_config', { 
          config: configToSave 
        });
        
        if (response.success) {
          update((state) => ({
            ...state,
            appConfig: configToSave,
            isLoading: false,
            isDirty: false,
            lastSaved: new Date().toISOString(),
          }));
        } else {
          throw new Error(response.error || 'Failed to save app config');
        }
      } catch (error) {
        console.error('Error saving app config:', error);
        update((state) => ({
          ...state,
          isLoading: false,
          error: error instanceof Error ? error.message : 'Save failed',
        }));
        throw error;
      }
    },

    // 個別設定更新
    updateWindowConfig(windowConfig: Partial<WindowConfig>) {
      update((state) => ({
        ...state,
        appConfig: {
          ...state.appConfig,
          window: {
            ...state.appConfig.window,
            ...windowConfig,
          },
        },
        isDirty: true,
      }));
    },

    updateLoggingConfig(loggingConfig: Partial<LoggingConfig>) {
      update((state) => ({
        ...state,
        appConfig: {
          ...state.appConfig,
          logging: {
            ...state.appConfig.logging,
            ...loggingConfig,
          },
        },
        isDirty: true,
      }));
    },

    updateSecurityConfig(securityConfig: Partial<SecurityConfig>) {
      update((state) => ({
        ...state,
        appConfig: {
          ...state.appConfig,
          security: {
            ...state.appConfig.security,
            ...securityConfig,
          },
        },
        isDirty: true,
      }));
    },

    // テーマ管理
    setTheme(theme: AppTheme) {
      this.updateWindowConfig({ theme });
      
      // システムテーマに応じてダークモードクラスを適用
      if (typeof window !== 'undefined') {
        const root = document.documentElement;
        
        if (theme === 'Dark') {
          root.classList.add('dark');
        } else if (theme === 'Light') {
          root.classList.remove('dark');
        } else { // System
          const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
          if (prefersDark) {
            root.classList.add('dark');
          } else {
            root.classList.remove('dark');
          }
        }
      }
    },

    // システムテーマ監視
    initSystemThemeListener() {
      if (typeof window !== 'undefined') {
        const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
        
        const handleThemeChange = () => {
          const currentState = get({ subscribe });
          if (currentState.appConfig.window.theme === 'System') {
            const root = document.documentElement;
            if (mediaQuery.matches) {
              root.classList.add('dark');
            } else {
              root.classList.remove('dark');
            }
          }
        };

        mediaQuery.addEventListener('change', handleThemeChange);
        handleThemeChange(); // 初期適用

        return () => {
          mediaQuery.removeEventListener('change', handleThemeChange);
        };
      }
    },

    // ウィンドウサイズ管理
    updateWindowSize(width: number, height: number) {
      this.updateWindowConfig({ width, height });
    },

    updateWindowPosition(x: number, y: number) {
      this.updateWindowConfig({ x, y });
    },

    setWindowMaximized(maximized: boolean) {
      this.updateWindowConfig({ maximized });
    },

    setAlwaysOnTop(alwaysOnTop: boolean) {
      this.updateWindowConfig({ always_on_top: alwaysOnTop });
    },

    // 設定のリセット
    async resetToDefaults() {
      const defaultConfig: AppConfig = {
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
        window: {
          width: 1200,
          height: 800,
          maximized: false,
          always_on_top: false,
          theme: 'System' as AppTheme,
        },
        logging: {
          enabled: true,
          auto_save: true,
          max_file_size_mb: 100,
          retention_days: 30,
          log_level: 'Info' as any,
          mask_sensitive_data: true,
        },
        security: {
          encrypt_passwords: true,
          require_confirmation_for_destructive_actions: true,
        },
        last_updated: new Date().toISOString(),
      };

      update((state) => ({
        ...state,
        appConfig: defaultConfig,
        isDirty: true,
      }));
    },

    // 設定のインポート/エクスポート
    async exportSettings(): Promise<string> {
      const currentState = get({ subscribe });
      return JSON.stringify(currentState.appConfig, null, 2);
    },

    async importSettings(settingsJson: string) {
      try {
        const importedConfig: AppConfig = JSON.parse(settingsJson);
        
        // 基本的なバリデーション
        if (!importedConfig.version || !importedConfig.terminal || !importedConfig.window) {
          throw new Error('Invalid settings format');
        }

        update((state) => ({
          ...state,
          appConfig: {
            ...importedConfig,
            last_updated: new Date().toISOString(),
          },
          isDirty: true,
        }));
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'Import failed';
        update((state) => ({
          ...state,
          error: errorMessage,
        }));
        throw error;
      }
    },

    // バリデーション
    validateConfig(): string[] {
      const currentState = get({ subscribe });
      const errors: string[] = [];

      // ウィンドウサイズ
      if (currentState.appConfig.window.width < 400 || currentState.appConfig.window.width > 4000) {
        errors.push('ウィンドウ幅は400-4000の範囲で指定してください');
      }

      if (currentState.appConfig.window.height < 300 || currentState.appConfig.window.height > 3000) {
        errors.push('ウィンドウ高さは300-3000の範囲で指定してください');
      }

      // ログ設定
      if (currentState.appConfig.logging.max_file_size_mb < 1 || currentState.appConfig.logging.max_file_size_mb > 1000) {
        errors.push('ログファイルサイズは1-1000MBの範囲で指定してください');
      }

      if (currentState.appConfig.logging.retention_days < 1 || currentState.appConfig.logging.retention_days > 365) {
        errors.push('ログ保持期間は1-365日の範囲で指定してください');
      }

      // ターミナル設定
      if (currentState.appConfig.terminal.font_size < 8 || currentState.appConfig.terminal.font_size > 32) {
        errors.push('フォントサイズは8-32の範囲で指定してください');
      }

      if (currentState.appConfig.terminal.max_history_size < 10 || currentState.appConfig.terminal.max_history_size > 10000) {
        errors.push('履歴サイズは10-10000の範囲で指定してください');
      }

      return errors;
    },

    // エラークリア
    clearError() {
      update((state) => ({ ...state, error: null }));
    },

    // 変更検知リセット
    markAsSaved() {
      update((state) => ({
        ...state,
        isDirty: false,
        lastSaved: new Date().toISOString(),
      }));
    },
  };
}

export const settings = createSettingsStore();

// 派生ストア
export const isDirty = derived(
  settings,
  ($settings) => $settings.isDirty
);

export const currentTheme = derived(
  settings,
  ($settings) => $settings.appConfig.window.theme
);

export const isLoggingEnabled = derived(
  settings,
  ($settings) => $settings.appConfig.logging.enabled
);

export const windowConfig = derived(
  settings,
  ($settings) => $settings.appConfig.window
);

export const terminalConfig = derived(
  settings,
  ($settings) => $settings.appConfig.terminal
);

export const loggingConfig = derived(
  settings,
  ($settings) => $settings.appConfig.logging
);

export const securityConfig = derived(
  settings,
  ($settings) => $settings.appConfig.security
);