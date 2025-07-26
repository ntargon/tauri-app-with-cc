<script lang="ts">
  import type { TerminalMessage } from '$lib/types';
  import { formatTimestamp } from '$lib/utils';
  import { appState } from '$lib/stores';

  export let message: TerminalMessage;
</script>

<div class="flex {message.direction === 'sent' ? 'justify-end' : 'justify-start'}">
  <div class="max-w-xs lg:max-w-md px-3 py-2 rounded-lg {
    message.direction === 'sent' 
      ? 'bg-blue-600 text-white' 
      : 'bg-gray-700 text-green-400'
  }">
    {#if $appState.settings.showTimestamps}
      <div class="text-xs opacity-75 mb-1">
        {formatTimestamp(new Date(message.timestamp))}
      </div>
    {/if}
    <div class="whitespace-pre-wrap break-words">{message.content}</div>
  </div>
</div>