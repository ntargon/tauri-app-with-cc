<script lang="ts">
  import { appState } from '$lib/stores';
  
  $: connectionStatusClass = $appState.connection.isConnected 
    ? 'text-green-600' 
    : 'text-red-600';
</script>

<div class="text-sm">
  <span class="text-gray-600 dark:text-gray-400">状態: </span>
  <span class="{connectionStatusClass} font-medium">
    {$appState.connection.isConnected ? '接続中' : '未接続'}
  </span>
</div>