<script lang="ts">
  import { onMount } from 'svelte';
  import { appState, actions } from '$lib/stores';
  import MessageList from './MessageList.svelte';
  import InputArea from './InputArea.svelte';

  onMount(() => {
    // イベントリスナーを初期化
    actions.initializeEventListeners();
  });
</script>

<div class="flex-1 flex flex-col min-w-0">
  <!-- ヘッダー -->
  <div class="flex-shrink-0 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-4 py-3">
    <div class="flex items-center justify-between">
      <h2 class="text-lg font-medium text-gray-900 dark:text-white">ターミナル</h2>
      <div class="flex items-center space-x-2">
        <span class="text-sm text-gray-600 dark:text-gray-400">
          {$appState.messages.length} メッセージ
        </span>
        <button
          on:click={actions.clearMessages}
          class="px-3 py-1 text-sm text-red-600 hover:text-red-700 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-900/20 rounded-md transition-colors"
          disabled={$appState.messages.length === 0}
        >
          クリア
        </button>
      </div>
    </div>
  </div>

  <!-- メッセージエリア -->
  <div class="flex-1 overflow-hidden bg-gray-900 text-green-400 font-mono text-sm">
    <MessageList />
  </div>

  <!-- 入力エリア -->
  <InputArea />
</div>