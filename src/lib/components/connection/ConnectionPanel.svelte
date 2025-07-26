<script lang="ts">
  import ConnectionStatus from './ConnectionStatus.svelte';
  import ConnectionForm from './ConnectionForm.svelte';
</script>

<div class="p-4 space-y-4">
  <div class="space-y-3">
    <h2 class="text-sm font-medium text-gray-900 dark:text-white">接続設定</h2>
    
    <!-- 接続状態 -->
    <ConnectionStatus />

    <!-- 接続フォーム -->
    <ConnectionForm />
  </div>
</div>