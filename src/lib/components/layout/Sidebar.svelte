<script lang="ts">
  import { sidebarCollapsed, actions } from '$lib/stores';
  import ConnectionPanel from '../connection/ConnectionPanel.svelte';
</script>

<div class="flex-shrink-0 {$sidebarCollapsed ? 'w-16' : 'w-80'} transition-all duration-300 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700">
  <!-- サイドバーヘッダー -->
  <div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
    {#if !$sidebarCollapsed}
      <h1 class="text-lg font-semibold text-gray-900 dark:text-white truncate">組み込み開発ターミナル</h1>
    {/if}
    <button
      on:click={actions.toggleSidebar}
      class="p-2 rounded-md text-gray-500 hover:text-gray-700 hover:bg-gray-100 dark:text-gray-400 dark:hover:text-gray-300 dark:hover:bg-gray-700"
      aria-label={$sidebarCollapsed ? 'サイドバーを展開' : 'サイドバーを折りたたみ'}
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
      </svg>
    </button>
  </div>

  {#if !$sidebarCollapsed}
    <!-- 接続パネル -->
    <ConnectionPanel />
  {/if}
</div>