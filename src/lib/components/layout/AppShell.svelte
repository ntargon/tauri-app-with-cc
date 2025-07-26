<script lang="ts">
  import Sidebar from './Sidebar.svelte';
  import Terminal from '../terminal/Terminal.svelte';
</script>

<svelte:head>
  <title>組み込み開発ターミナル</title>
</svelte:head>

<div class="h-screen bg-gray-50 dark:bg-gray-900 flex overflow-hidden">
  <!-- サイドバー -->
  <Sidebar />

  <!-- メインコンテンツ -->
  <Terminal />
</div>