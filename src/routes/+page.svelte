<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { ConnectionConfig, TerminalMessage, AppState, ApiResponse } from '$lib/types';
  import { generateId, formatTimestamp, validateSerialPort, validateTcpConnection } from '$lib/utils';

  // アプリケーション状態
  let appState: AppState = {
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

  // UI状態
  let sidebarCollapsed = false;
  let currentInput = '';
  let inputMode: 'text' | 'hex' = 'text';

  // 接続設定フォーム
  let connectionForm: ConnectionConfig = {
    id: generateId(),
    name: '新規接続',
    type: 'serial',
    serialPort: '',
    baudRate: 115200,
    host: 'localhost',
    port: 8080
  };

  // 利用可能なシリアルポート
  let availablePorts: string[] = [];

  onMount(async () => {
    await loadSerialPorts();
  });

  async function loadSerialPorts() {
    try {
      const response: ApiResponse<string[]> = await invoke('get_serial_ports');
      if (response.success && response.data) {
        availablePorts = response.data;
      }
    } catch (error) {
      console.error('シリアルポート取得エラー:', error);
    }
  }

  async function handleConnect() {
    if (appState.connection.isConnected) {
      await handleDisconnect();
      return;
    }

    // 入力検証
    if (connectionForm.type === 'serial') {
      if (!validateSerialPort(connectionForm.serialPort || '')) {
        appState.connection.error = 'シリアルポートを選択してください';
        return;
      }
    } else {
      if (!validateTcpConnection(connectionForm.host || '', connectionForm.port || 0)) {
        appState.connection.error = 'ホストとポートを正しく入力してください';
        return;
      }
    }

    appState.connection.isConnecting = true;
    appState.connection.error = null;

    try {
      const response: ApiResponse<string> = await invoke('connect_device', {
        config: connectionForm
      });

      if (response.success) {
        appState.connection.isConnected = true;
        appState.connection.config = { ...connectionForm };
        addMessage('info', `${connectionForm.name}に接続しました`);
      } else {
        appState.connection.error = response.error || '接続に失敗しました';
      }
    } catch (error) {
      appState.connection.error = '接続エラーが発生しました';
      console.error('接続エラー:', error);
    } finally {
      appState.connection.isConnecting = false;
    }
  }

  async function handleDisconnect() {
    try {
      const response: ApiResponse<string> = await invoke('disconnect_device');
      if (response.success) {
        addMessage('info', '接続を切断しました');
      }
    } catch (error) {
      console.error('切断エラー:', error);
    } finally {
      appState.connection.isConnected = false;
      appState.connection.config = null;
    }
  }

  async function sendMessage() {
    if (!appState.connection.isConnected || !currentInput.trim()) return;

    try {
      const message: TerminalMessage = {
        id: generateId(),
        timestamp: new Date().toISOString(),
        direction: 'sent',
        content: currentInput.trim(),
        type: inputMode
      };

      const response: ApiResponse<string> = await invoke('send_message', {
        message: currentInput.trim(),
        format: inputMode
      });

      if (response.success) {
        appState.messages = [...appState.messages, message];
        currentInput = '';
      } else {
        addMessage('error', response.error || 'メッセージ送信に失敗しました');
      }
    } catch (error) {
      addMessage('error', 'メッセージ送信エラー');
      console.error('送信エラー:', error);
    }
  }

  function addMessage(type: 'info' | 'error', content: string) {
    const message: TerminalMessage = {
      id: generateId(),
      timestamp: new Date().toISOString(),
      direction: 'received',
      content,
      type: 'text'
    };
    appState.messages = [...appState.messages, message];
  }

  function clearMessages() {
    appState.messages = [];
  }

  function handleInputKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      sendMessage();
    }
  }

  function toggleSidebar() {
    sidebarCollapsed = !sidebarCollapsed;
  }

  // リアクティブ更新
  $: connectionButtonText = appState.connection.isConnecting 
    ? '接続中...' 
    : appState.connection.isConnected 
      ? '切断' 
      : '接続';

  $: connectionStatusClass = appState.connection.isConnected 
    ? 'text-green-600' 
    : 'text-red-600';
</script>

<svelte:head>
  <title>組み込み開発ターミナル</title>
</svelte:head>

<div class="h-screen bg-gray-50 dark:bg-gray-900 flex overflow-hidden">
  <!-- サイドバー -->
  <div class="flex-shrink-0 {sidebarCollapsed ? 'w-16' : 'w-80'} transition-all duration-300 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700">
    <!-- サイドバーヘッダー -->
    <div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
      {#if !sidebarCollapsed}
        <h1 class="text-lg font-semibold text-gray-900 dark:text-white truncate">組み込み開発ターミナル</h1>
      {/if}
      <button
        on:click={toggleSidebar}
        class="p-2 rounded-md text-gray-500 hover:text-gray-700 hover:bg-gray-100 dark:text-gray-400 dark:hover:text-gray-300 dark:hover:bg-gray-700"
        aria-label={sidebarCollapsed ? 'サイドバーを展開' : 'サイドバーを折りたたみ'}
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
        </svg>
      </button>
    </div>

    {#if !sidebarCollapsed}
      <!-- 接続パネル -->
      <div class="p-4 space-y-4">
        <div class="space-y-3">
          <h2 class="text-sm font-medium text-gray-900 dark:text-white">接続設定</h2>
          
          <!-- 接続状態 -->
          <div class="text-sm">
            <span class="text-gray-600 dark:text-gray-400">状態: </span>
            <span class="{connectionStatusClass} font-medium">
              {appState.connection.isConnected ? '接続中' : '未接続'}
            </span>
          </div>

          <!-- 接続名 -->
          <div>
            <label for="connection-name" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">接続名</label>
            <input
              id="connection-name"
              type="text"
              bind:value={connectionForm.name}
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              disabled={appState.connection.isConnected}
            />
          </div>

          <!-- 接続タイプ -->
          <div>
            <label for="connection-type" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">接続タイプ</label>
            <select
              id="connection-type"
              bind:value={connectionForm.type}
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              disabled={appState.connection.isConnected}
            >
              <option value="serial">シリアル通信</option>
              <option value="tcp">TCP通信</option>
            </select>
          </div>

          {#if connectionForm.type === 'serial'}
            <!-- シリアル設定 -->
            <div>
              <label for="serial-port" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">ポート</label>
              <select
                id="serial-port"
                bind:value={connectionForm.serialPort}
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                disabled={appState.connection.isConnected}
              >
                <option value="">ポートを選択</option>
                {#each availablePorts as port}
                  <option value={port}>{port}</option>
                {/each}
              </select>
            </div>

            <div>
              <label for="baud-rate" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">ボーレート</label>
              <select
                id="baud-rate"
                bind:value={connectionForm.baudRate}
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                disabled={appState.connection.isConnected}
              >
                <option value={9600}>9600</option>
                <option value={19200}>19200</option>
                <option value={38400}>38400</option>
                <option value={57600}>57600</option>
                <option value={115200}>115200</option>
              </select>
            </div>
          {:else}
            <!-- TCP設定 -->
            <div>
              <label for="tcp-host" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">ホスト</label>
              <input
                id="tcp-host"
                type="text"
                bind:value={connectionForm.host}
                placeholder="localhost"
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                disabled={appState.connection.isConnected}
              />
            </div>

            <div>
              <label for="tcp-port" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">ポート</label>
              <input
                id="tcp-port"
                type="number"
                bind:value={connectionForm.port}
                min="1"
                max="65535"
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                disabled={appState.connection.isConnected}
              />
            </div>
          {/if}

          <!-- エラー表示 -->
          {#if appState.connection.error}
            <div class="p-2 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md">
              <p class="text-sm text-red-700 dark:text-red-400">{appState.connection.error}</p>
            </div>
          {/if}

          <!-- 接続ボタン -->
          <button
            on:click={handleConnect}
            disabled={appState.connection.isConnecting}
            class="w-full px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white font-medium rounded-md text-sm transition-colors"
          >
            {connectionButtonText}
          </button>
        </div>
      </div>
    {/if}
  </div>

  <!-- メインコンテンツ -->
  <div class="flex-1 flex flex-col min-w-0">
    <!-- ヘッダー -->
    <div class="flex-shrink-0 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-4 py-3">
      <div class="flex items-center justify-between">
        <h2 class="text-lg font-medium text-gray-900 dark:text-white">ターミナル</h2>
        <div class="flex items-center space-x-2">
          <span class="text-sm text-gray-600 dark:text-gray-400">
            {appState.messages.length} メッセージ
          </span>
          <button
            on:click={clearMessages}
            class="px-3 py-1 text-sm text-red-600 hover:text-red-700 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-900/20 rounded-md transition-colors"
            disabled={appState.messages.length === 0}
          >
            クリア
          </button>
        </div>
      </div>
    </div>

    <!-- メッセージエリア -->
    <div class="flex-1 overflow-hidden bg-gray-900 text-green-400 font-mono text-sm">
      <div class="h-full overflow-y-auto p-4 space-y-2">
        {#if appState.messages.length === 0}
          <div class="text-center text-gray-500 mt-8">
            <p>メッセージはまだありません</p>
            <p class="text-xs mt-1">接続してデータを送受信してください</p>
          </div>
        {:else}
          {#each appState.messages as message (message.id)}
            <div class="flex {message.direction === 'sent' ? 'justify-end' : 'justify-start'}">
              <div class="max-w-xs lg:max-w-md px-3 py-2 rounded-lg {
                message.direction === 'sent' 
                  ? 'bg-blue-600 text-white' 
                  : 'bg-gray-700 text-green-400'
              }">
                {#if appState.settings.showTimestamps}
                  <div class="text-xs opacity-75 mb-1">
                    {formatTimestamp(new Date(message.timestamp))}
                  </div>
                {/if}
                <div class="whitespace-pre-wrap break-words">{message.content}</div>
              </div>
            </div>
          {/each}
        {/if}
      </div>
    </div>

    <!-- 入力エリア -->
    <div class="flex-shrink-0 bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 p-4">
      <div class="flex items-end space-x-3">
        <div class="flex-1">
          <div class="flex items-center space-x-2 mb-2">
            <select
              bind:value={inputMode}
              class="px-2 py-1 text-xs border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            >
              <option value="text">テキスト</option>
              <option value="hex">16進数</option>
            </select>
          </div>
          <textarea
            bind:value={currentInput}
            on:keydown={handleInputKeydown}
            placeholder={inputMode === 'text' ? 'メッセージを入力...' : '16進数を入力 (例: 41 42 43)'}
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md resize-none bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            rows="2"
            disabled={!appState.connection.isConnected}
          ></textarea>
        </div>
        <button
          on:click={sendMessage}
          disabled={!appState.connection.isConnected || !currentInput.trim()}
          class="px-4 py-2 bg-green-600 hover:bg-green-700 disabled:bg-gray-400 text-white font-medium rounded-md transition-colors"
        >
          送信
        </button>
      </div>
    </div>
  </div>
</div>