<script lang="ts">
  import { appState } from '$lib/stores';
  import MessageItem from './MessageItem.svelte';
</script>

<div class="h-full overflow-y-auto p-4 space-y-2">
  {#if $appState.messages.length === 0}
    <div class="text-center text-gray-500 mt-8">
      <p>メッセージはまだありません</p>
      <p class="text-xs mt-1">接続してデータを送受信してください</p>
    </div>
  {:else}
    {#each $appState.messages as message (message.id)}
      <MessageItem {message} />
    {/each}
  {/if}
</div>