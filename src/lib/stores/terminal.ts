import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type {
  TerminalMessage,
  TerminalConfig,
  TerminalState,
  CommandHistory,
  ApiResponse,
  LineEnding,
} from '$lib/types';

// ターミナル状態ストア
function createTerminalStore() {
  const { subscribe, set, update } = writable<TerminalState>({
    messages: [],
    commandHistory: {
      commands: [],
      max_size: 100,
      current_index: null,
    },
    currentInput: '',
    config: {
      encoding: 'UTF-8',
      line_ending: 'CrLf' as LineEnding,
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
    isLogging: false,
    selectedMessages: [],
    searchQuery: '',
    filteredMessages: [],
  });

  let isInitialized = false;

  return {
    subscribe,
    set,
    update,

    // 初期化
    async init() {
      if (isInitialized) return;

      await this.loadConfig();
      await this.loadMessages();
      await this.loadCommandHistory();
      
      // メッセージ受信リスナーを設定
      await this.setupMessageListener();
      
      isInitialized = true;
    },

    // 設定管理
    async loadConfig() {
      try {
        const response: ApiResponse<TerminalConfig> = await invoke('get_terminal_config');
        
        if (response.success && response.data) {
          update((state) => ({
            ...state,
            config: response.data!,
          }));
        }
      } catch (error) {
        console.error('Error loading terminal config:', error);
      }
    },

    async saveConfig(config: TerminalConfig) {
      try {
        const response: ApiResponse<string> = await invoke('update_terminal_config', { config });
        
        if (response.success) {
          update((state) => ({
            ...state,
            config,
          }));
        } else {
          throw new Error(response.error || 'Failed to save config');
        }
      } catch (error) {
        console.error('Error saving terminal config:', error);
        throw error;
      }
    },

    // メッセージ管理
    async loadMessages() {
      try {
        const response: ApiResponse<TerminalMessage[]> = await invoke('get_terminal_messages');
        
        if (response.success && response.data) {
          update((state) => ({
            ...state,
            messages: response.data!,
            filteredMessages: response.data!,
          }));
        }
      } catch (error) {
        console.error('Error loading messages:', error);
      }
    },

    async addMessage(message: TerminalMessage) {
      try {
        const response: ApiResponse<string> = await invoke('add_terminal_message', { message });
        
        if (response.success) {
          update((state) => {
            const newMessages = [...state.messages, message];
            
            // 最大履歴サイズを超えた場合は古いものを削除
            if (newMessages.length > state.config.max_history_size) {
              const removeCount = newMessages.length - state.config.max_history_size;
              newMessages.splice(0, removeCount);
            }
            
            return {
              ...state,
              messages: newMessages,
              filteredMessages: this.applySearchFilter(newMessages, state.searchQuery),
            };
          });
        }
      } catch (error) {
        console.error('Error adding message:', error);
      }
    },

    async clearMessages() {
      try {
        const response: ApiResponse<string> = await invoke('clear_terminal_messages');
        
        if (response.success) {
          update((state) => ({
            ...state,
            messages: [],
            filteredMessages: [],
            selectedMessages: [],
          }));
        } else {
          throw new Error(response.error || 'Failed to clear messages');
        }
      } catch (error) {
        console.error('Error clearing messages:', error);
        throw error;
      }
    },

    // コマンド履歴管理
    async loadCommandHistory() {
      try {
        const response: ApiResponse<string[]> = await invoke('get_command_history');
        
        if (response.success && response.data) {
          update((state) => ({
            ...state,
            commandHistory: {
              ...state.commandHistory,
              commands: response.data!,
            },
          }));
        }
      } catch (error) {
        console.error('Error loading command history:', error);
      }
    },

    async addCommandToHistory(command: string) {
      if (!command.trim()) return;

      try {
        const response: ApiResponse<string> = await invoke('add_command_to_history', { command });
        
        if (response.success) {
          update((state) => {
            const newHistory = { ...state.commandHistory };
            
            // 同じコマンドが最後にある場合は追加しない
            if (newHistory.commands[newHistory.commands.length - 1] !== command) {
              newHistory.commands.push(command);
              
              // 最大サイズを超えた場合は古いものを削除
              if (newHistory.commands.length > newHistory.max_size) {
                newHistory.commands.shift();
              }
            }
            
            newHistory.current_index = null;
            
            return {
              ...state,
              commandHistory: newHistory,
            };
          });
        }
      } catch (error) {
        console.error('Error adding command to history:', error);
      }
    },

    // 履歴ナビゲーション
    getPreviousCommand(): string | null {
      let result: string | null = null;
      
      update((state) => {
        const history = state.commandHistory;
        if (history.commands.length === 0) return state;

        let newIndex: number;
        if (history.current_index === null) {
          newIndex = history.commands.length - 1;
        } else if (history.current_index > 0) {
          newIndex = history.current_index - 1;
        } else {
          return state; // 最初のコマンドの場合は変更しない
        }

        result = history.commands[newIndex];
        
        return {
          ...state,
          commandHistory: {
            ...history,
            current_index: newIndex,
          },
        };
      });
      
      return result;
    },

    getNextCommand(): string | null {
      let result: string | null = null;
      
      update((state) => {
        const history = state.commandHistory;
        if (history.current_index === null) return state;

        if (history.current_index < history.commands.length - 1) {
          const newIndex = history.current_index + 1;
          result = history.commands[newIndex];
          
          return {
            ...state,
            commandHistory: {
              ...history,
              current_index: newIndex,
            },
          };
        } else {
          // 最後のコマンドより先に進む場合は空文字を返す
          result = '';
          return {
            ...state,
            commandHistory: {
              ...history,
              current_index: null,
            },
          };
        }
      });
      
      return result;
    },

    // 検索機能
    setSearchQuery(query: string) {
      update((state) => ({
        ...state,
        searchQuery: query,
        filteredMessages: this.applySearchFilter(state.messages, query),
      }));
    },

    applySearchFilter(messages: TerminalMessage[], query: string): TerminalMessage[] {
      if (!query.trim()) return messages;
      
      const lowerQuery = query.toLowerCase();
      return messages.filter((message) =>
        message.content.toLowerCase().includes(lowerQuery)
      );
    },

    async searchCommandHistory(query: string): Promise<string[]> {
      try {
        const response: ApiResponse<string[]> = await invoke('search_command_history', { query });
        
        if (response.success && response.data) {
          return response.data;
        }
        return [];
      } catch (error) {
        console.error('Error searching command history:', error);
        return [];
      }
    },

    // 入力管理
    setCurrentInput(input: string) {
      update((state) => ({
        ...state,
        currentInput: input,
      }));
    },

    clearCurrentInput() {
      update((state) => ({
        ...state,
        currentInput: '',
      }));
    },

    // メッセージ選択
    toggleMessageSelection(messageId: string) {
      update((state) => {
        const selected = [...state.selectedMessages];
        const index = selected.indexOf(messageId);
        
        if (index > -1) {
          selected.splice(index, 1);
        } else {
          selected.push(messageId);
        }
        
        return {
          ...state,
          selectedMessages: selected,
        };
      });
    },

    clearSelection() {
      update((state) => ({
        ...state,
        selectedMessages: [],
      }));
    },

    selectAllMessages() {
      update((state) => ({
        ...state,
        selectedMessages: state.filteredMessages.map((msg) => msg.id),
      }));
    },

    // エクスポート
    async exportMessages(
      format: 'txt' | 'csv' | 'json',
      includeTimestamp = true,
      includeDirection = true
    ): Promise<string> {
      try {
        const options = {
          format,
          include_timestamp: includeTimestamp,
          include_direction: includeDirection,
          filter: null,
        };

        const response: ApiResponse<string> = await invoke('export_terminal_messages', { options });
        
        if (response.success && response.data) {
          return response.data;
        } else {
          throw new Error(response.error || 'Export failed');
        }
      } catch (error) {
        console.error('Error exporting messages:', error);
        throw error;
      }
    },

    // メッセージ受信リスナー
    async setupMessageListener() {
      try {
        await listen('terminal-message-received', (event) => {
          const message = event.payload as TerminalMessage;
          this.addMessage(message);
        });
      } catch (error) {
        console.error('Error setting up message listener:', error);
      }
    },

    // ログ状態
    setLoggingState(isLogging: boolean) {
      update((state) => ({
        ...state,
        isLogging,
      }));
    },
  };
}

export const terminal = createTerminalStore();

// 派生ストア
export const messageCount = derived(
  terminal,
  ($terminal) => $terminal.messages.length
);

export const filteredMessageCount = derived(
  terminal,
  ($terminal) => $terminal.filteredMessages.length
);

export const hasMessages = derived(
  terminal,
  ($terminal) => $terminal.messages.length > 0
);

export const hasSelection = derived(
  terminal,
  ($terminal) => $terminal.selectedMessages.length > 0
);

export const selectionCount = derived(
  terminal,
  ($terminal) => $terminal.selectedMessages.length
);

export const isSearchActive = derived(
  terminal,
  ($terminal) => $terminal.searchQuery.trim().length > 0
);

export const latestMessage = derived(
  terminal,
  ($terminal) => $terminal.messages[$terminal.messages.length - 1] || null
);