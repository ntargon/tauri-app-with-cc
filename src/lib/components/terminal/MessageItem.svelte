<script lang="ts">
  import type { TerminalMessage } from '$lib/types';
  import { formatTimestamp } from '$lib/utils';
  import { appState } from '$lib/stores';

  export let message: TerminalMessage;
</script>

<div class="text-sm font-mono leading-tight py-0.5 px-2 hover:bg-gray-800/50 transition-colors">
  <span class="text-gray-500 text-xs">
    {#if $appState.settings.showTimestamps}
      [{formatTimestamp(new Date(message.timestamp))}]
    {/if}
    {message.direction === 'sent' ? '>' : '<'}
  </span>
  <span class="ml-2 {message.direction === 'sent' ? 'text-blue-400' : 'text-green-400'} whitespace-pre-wrap break-all">
    {message.content}
  </span>
</div>