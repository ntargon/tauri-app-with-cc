import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type {
  ConnectionConfig,
  ConnectionStatus,
  SerialPortInfo,
  ApiResponse,
  ConnectionState,
} from '$lib/types';

// 接続状態ストア
function createConnectionStore() {
  const { subscribe, set, update } = writable<ConnectionState>({
    currentConnection: null,
    connectionStatus: 'Disconnected',
    availablePorts: [],
    profiles: [],
    isLoading: false,
    error: null,
  });

  return {
    subscribe,
    set,
    update,

    // 初期化
    async init() {
      await this.loadProfiles();
      await this.refreshPorts();
    },

    // シリアルポート一覧を取得
    async refreshPorts() {
      update((state) => ({ ...state, isLoading: true, error: null }));

      try {
        const response: ApiResponse<string[]> = await invoke('get_serial_ports');
        
        if (response.success && response.data) {
          update((state) => ({
            ...state,
            availablePorts: response.data || [],
            isLoading: false,
          }));
        } else {
          throw new Error(response.error || 'Failed to get serial ports');
        }
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'Unknown error';
        update((state) => ({
          ...state,
          availablePorts: [],
          isLoading: false,
          error: errorMessage,
        }));
      }
    },

    // シリアルポート詳細情報を取得
    async getPortsInfo(): Promise<SerialPortInfo[]> {
      try {
        const response: ApiResponse<SerialPortInfo[]> = await invoke('get_serial_ports_info');
        
        if (response.success && response.data) {
          return response.data;
        } else {
          throw new Error(response.error || 'Failed to get serial port info');
        }
      } catch (error) {
        console.error('Error getting port info:', error);
        return [];
      }
    },

    // デバイスに接続
    async connect(config: ConnectionConfig) {
      update((state) => ({ 
        ...state, 
        connectionStatus: 'Connecting',
        isLoading: true,
        error: null 
      }));

      try {
        const response: ApiResponse<string> = await invoke('connect_device', { config });
        
        if (response.success) {
          update((state) => ({
            ...state,
            currentConnection: config,
            connectionStatus: 'Connected',
            isLoading: false,
          }));
          
          // 最近使用したプロファイルを更新
          await this.updateRecentProfile(config.id);
        } else {
          throw new Error(response.error || 'Connection failed');
        }
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'Connection failed';
        update((state) => ({
          ...state,
          connectionStatus: 'Error',
          isLoading: false,
          error: errorMessage,
        }));
        throw error;
      }
    },

    // デバイスから切断
    async disconnect() {
      update((state) => ({ ...state, isLoading: true, error: null }));

      try {
        const response: ApiResponse<string> = await invoke('disconnect_device');
        
        if (response.success) {
          update((state) => ({
            ...state,
            currentConnection: null,
            connectionStatus: 'Disconnected',
            isLoading: false,
          }));
        } else {
          throw new Error(response.error || 'Disconnect failed');
        }
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'Disconnect failed';
        update((state) => ({
          ...state,
          isLoading: false,
          error: errorMessage,
        }));
        throw error;
      }
    },

    // メッセージ送信
    async sendMessage(message: string) {
      try {
        const response: ApiResponse<string> = await invoke('send_message', { message });
        
        if (!response.success) {
          throw new Error(response.error || 'Send failed');
        }
      } catch (error) {
        update((state) => ({
          ...state,
          error: error instanceof Error ? error.message : 'Send failed',
        }));
        throw error;
      }
    },

    // 接続状態を確認
    async checkConnectionStatus() {
      try {
        const response: ApiResponse<boolean> = await invoke('get_connection_status');
        
        if (response.success && response.data !== undefined) {
          const status: ConnectionStatus = response.data ? 'Connected' : 'Disconnected';
          update((state) => ({
            ...state,
            connectionStatus: status,
          }));
        }
      } catch (error) {
        console.error('Error checking connection status:', error);
      }
    },

    // プロファイル管理
    async loadProfiles() {
      try {
        const response: ApiResponse<ConnectionConfig[]> = await invoke('get_profiles');
        
        if (response.success && response.data) {
          update((state) => ({
            ...state,
            profiles: response.data || [],
          }));
        }
      } catch (error) {
        console.error('Error loading profiles:', error);
      }
    },

    async saveProfile(profile: ConnectionConfig) {
      try {
        const response: ApiResponse<string> = await invoke('add_profile', { profile });
        
        if (response.success) {
          await this.loadProfiles();
        } else {
          throw new Error(response.error || 'Failed to save profile');
        }
      } catch (error) {
        update((state) => ({
          ...state,
          error: error instanceof Error ? error.message : 'Save failed',
        }));
        throw error;
      }
    },

    async updateProfile(profile: ConnectionConfig) {
      try {
        const response: ApiResponse<string> = await invoke('update_profile', { profile });
        
        if (response.success) {
          await this.loadProfiles();
        } else {
          throw new Error(response.error || 'Failed to update profile');
        }
      } catch (error) {
        update((state) => ({
          ...state,
          error: error instanceof Error ? error.message : 'Update failed',
        }));
        throw error;
      }
    },

    async deleteProfile(profileId: string) {
      try {
        const response: ApiResponse<string> = await invoke('delete_profile', { profileId });
        
        if (response.success) {
          await this.loadProfiles();
        } else {
          throw new Error(response.error || 'Failed to delete profile');
        }
      } catch (error) {
        update((state) => ({
          ...state,
          error: error instanceof Error ? error.message : 'Delete failed',
        }));
        throw error;
      }
    },

    async setActiveProfile(profileId: string) {
      try {
        const response: ApiResponse<string> = await invoke('set_active_profile', { profileId });
        
        if (!response.success) {
          throw new Error(response.error || 'Failed to set active profile');
        }
      } catch (error) {
        update((state) => ({
          ...state,
          error: error instanceof Error ? error.message : 'Failed to set active profile',
        }));
        throw error;
      }
    },

    async updateRecentProfile(profileId: string) {
      try {
        await this.setActiveProfile(profileId);
      } catch (error) {
        console.error('Error updating recent profile:', error);
      }
    },

    async getRecentProfiles(limit = 5): Promise<ConnectionConfig[]> {
      try {
        const response: ApiResponse<ConnectionConfig[]> = await invoke('get_recent_profiles', { limit });
        
        if (response.success && response.data) {
          return response.data;
        }
        return [];
      } catch (error) {
        console.error('Error getting recent profiles:', error);
        return [];
      }
    },

    // エラークリア
    clearError() {
      update((state) => ({ ...state, error: null }));
    },

    // 接続情報取得
    async getConnectionInfo(): Promise<string | null> {
      try {
        const response: ApiResponse<string | null> = await invoke('get_connection_info');
        
        if (response.success) {
          return response.data || null;
        }
        return null;
      } catch (error) {
        console.error('Error getting connection info:', error);
        return null;
      }
    },
  };
}

export const connection = createConnectionStore();

// 派生ストア
export const isConnected = derived(
  connection,
  ($connection) => $connection.connectionStatus === 'Connected'
);

export const isConnecting = derived(
  connection,
  ($connection) => $connection.connectionStatus === 'Connecting'
);

export const hasError = derived(
  connection,
  ($connection) => $connection.error !== null
);

export const connectionDisplayName = derived(
  connection,
  ($connection) => {
    if (!$connection.currentConnection) return null;
    
    const config = $connection.currentConnection;
    if (config.connection_type === 'Serial' && config.serial_config) {
      return `${config.name} (${config.serial_config.port})`;
    } else if (config.connection_type === 'Tcp' && config.tcp_config) {
      return `${config.name} (${config.tcp_config.host}:${config.tcp_config.port})`;
    }
    return config.name;
  }
);