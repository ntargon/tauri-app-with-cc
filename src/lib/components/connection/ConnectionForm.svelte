<script lang="ts">
  import { appState, connectionForm, availablePorts, actions } from '$lib/stores';
  import { onMount } from 'svelte';

  onMount(() => {
    actions.loadSerialPorts();
  });

  async function handleConnect() {
    if ($appState.connection.isConnected) {
      await actions.disconnect();
    } else {
      await actions.connect();
    }
  }

  $: connectionButtonText = $appState.connection.isConnecting 
    ? '接続中...' 
    : $appState.connection.isConnected 
      ? '切断' 
      : '接続';
</script>

<div class="space-y-3">
  <!-- 接続名 -->
  <div>
    <label for="connection-name" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">接続名</label>
    <input
      id="connection-name"
      type="text"
      bind:value={$connectionForm.name}
      class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
      disabled={$appState.connection.isConnected}
    />
  </div>

  <!-- 接続タイプ -->
  <div>
    <label for="connection-type" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">接続タイプ</label>
    <select
      id="connection-type"
      bind:value={$connectionForm.type}
      class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
      disabled={$appState.connection.isConnected}
    >
      <option value="serial">シリアル通信</option>
      <option value="tcp">TCP通信</option>
    </select>
  </div>

  {#if $connectionForm.type === 'serial'}
    <!-- シリアル設定 -->
    <div>
      <label for="serial-port" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">ポート</label>
      <select
        id="serial-port"
        bind:value={$connectionForm.serialPort}
        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
        disabled={$appState.connection.isConnected}
      >
        <option value="">ポートを選択</option>
        {#each $availablePorts as port}
          <option value={port}>{port}</option>
        {/each}
      </select>
    </div>

    <div>
      <label for="baud-rate" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">ボーレート</label>
      <select
        id="baud-rate"
        bind:value={$connectionForm.baudRate}
        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
        disabled={$appState.connection.isConnected}
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
        bind:value={$connectionForm.host}
        placeholder="localhost"
        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
        disabled={$appState.connection.isConnected}
      />
    </div>

    <div>
      <label for="tcp-port" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">ポート</label>
      <input
        id="tcp-port"
        type="number"
        bind:value={$connectionForm.port}
        min="1"
        max="65535"
        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
        disabled={$appState.connection.isConnected}
      />
    </div>
  {/if}

  <!-- エラー表示 -->
  {#if $appState.connection.error}
    <div class="p-2 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md">
      <p class="text-sm text-red-700 dark:text-red-400">{$appState.connection.error}</p>
    </div>
  {/if}

  <!-- 接続ボタン -->
  <button
    on:click={handleConnect}
    disabled={$appState.connection.isConnecting}
    class="w-full px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white font-medium rounded-md text-sm transition-colors"
  >
    {connectionButtonText}
  </button>
</div>