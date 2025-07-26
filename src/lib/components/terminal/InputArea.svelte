<script lang="ts">
  import { appState, currentInput, inputMode, actions } from '$lib/stores';

  function handleInputKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      actions.sendMessage();
    }
  }
</script>

<div class="flex-shrink-0 bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 p-4">
  <div class="flex items-end space-x-3">
    <div class="flex-1">
      <div class="flex items-center space-x-2 mb-2">
        <select
          bind:value={$inputMode}
          class="px-2 py-1 text-xs border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
        >
          <option value="text">テキスト</option>
          <option value="hex">16進数</option>
        </select>
      </div>
      <textarea
        bind:value={$currentInput}
        on:keydown={handleInputKeydown}
        placeholder={$inputMode === 'text' ? 'メッセージを入力...' : '16進数を入力 (例: 41 42 43)'}
        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md resize-none bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
        rows="2"
        disabled={!$appState.connection.isConnected}
      ></textarea>
    </div>
    <button
      on:click={actions.sendMessage}
      disabled={!$appState.connection.isConnected || !$currentInput.trim()}
      class="px-4 py-2 bg-green-600 hover:bg-green-700 disabled:bg-gray-400 text-white font-medium rounded-md transition-colors"
    >
      送信
    </button>
  </div>
</div>