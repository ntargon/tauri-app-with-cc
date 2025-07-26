<script lang="ts">
	import { onMount, afterUpdate, tick } from 'svelte';
	import { terminal } from '$lib/stores';
	import type { TerminalMessage, MessageDirection } from '$lib/types';

	// プロパティ
	export let autoScroll = true;
	export let maxLines = 1000;

	// DOM要素への参照
	let outputContainer: HTMLDivElement;
	let shouldScrollToBottom = false;

	// ストアからメッセージを購読
	$: messages = $terminal.messages.slice(-maxLines);
	$: filteredMessages = $terminal.searchTerm 
		? messages.filter(msg => msg.content.toLowerCase().includes($terminal.searchTerm.toLowerCase()))
		: messages;

	// 選択状態
	let selectedMessages = new Set<string>();

	// 新しいメッセージが追加された時の自動スクロール処理
	afterUpdate(() => {
		if (shouldScrollToBottom && autoScroll && outputContainer) {
			outputContainer.scrollTop = outputContainer.scrollHeight;
			shouldScrollToBottom = false;
		}
	});

	// メッセージが追加されたかを監視
	$: if (messages.length > 0) {
		shouldScrollToBottom = true;
	}

	// メッセージの時刻フォーマット
	function formatTime(timestamp: string): string {
		const date = new Date(timestamp);
		return date.toLocaleTimeString('ja-JP', { 
			hour12: false,
			hour: '2-digit',
			minute: '2-digit',
			second: '2-digit',
			fractionalSecondDigits: 3
		});
	}

	// メッセージの方向に応じたクラス
	function getMessageClass(direction: MessageDirection): string {
		return direction === 'Sent' ? 'message-sent' : 'message-received';
	}

	// メッセージの選択/選択解除
	function toggleMessageSelection(messageId: string, event: Event) {
		if (event.ctrlKey || event.metaKey) {
			if (selectedMessages.has(messageId)) {
				selectedMessages.delete(messageId);
			} else {
				selectedMessages.add(messageId);
			}
			selectedMessages = selectedMessages;
		} else {
			// 通常クリックの場合は単一選択
			selectedMessages = new Set([messageId]);
		}
	}

	// 全選択/全選択解除
	function toggleSelectAll() {
		if (selectedMessages.size === filteredMessages.length) {
			selectedMessages = new Set();
		} else {
			selectedMessages = new Set(filteredMessages.map(msg => msg.id));
		}
	}

	// 選択されたメッセージをクリップボードにコピー
	async function copySelectedMessages() {
		const selectedMsgs = filteredMessages.filter(msg => selectedMessages.has(msg.id));
		if (selectedMsgs.length === 0) return;

		const text = selectedMsgs.map(msg => {
			const time = formatTime(msg.timestamp);
			const direction = msg.direction === 'Sent' ? '送信' : '受信';
			return `[${time}] ${direction}: ${msg.content}`;
		}).join('\n');

		try {
			await navigator.clipboard.writeText(text);
			// 成功のフィードバック（簡易版）
			console.log('クリップボードにコピーしました');
		} catch (err) {
			console.error('コピーに失敗しました:', err);
		}
	}

	// 手動スクロールを下に
	function scrollToBottom() {
		if (outputContainer) {
			outputContainer.scrollTop = outputContainer.scrollHeight;
		}
	}

	// 出力をクリア
	function clearOutput() {
		selectedMessages = new Set();
		terminal.clearMessages();
	}

	// エスケープHTML（XSS対策）
	function escapeHtml(text: string): string {
		const div = document.createElement('div');
		div.textContent = text;
		return div.innerHTML;
	}

	// 制御文字の可視化
	function visualizeControlChars(text: string): string {
		return text
			.replace(/\r\n/g, '<span class="control-char">↵</span>\n')
			.replace(/\r/g, '<span class="control-char">␍</span>')
			.replace(/\n/g, '<span class="control-char">↵</span>\n')
			.replace(/\t/g, '<span class="control-char">→</span>')
			.replace(/\0/g, '<span class="control-char">␀</span>');
	}

	// メッセージ内容の処理
	function processMessageContent(content: string): string {
		const escaped = escapeHtml(content);
		return visualizeControlChars(escaped);
	}
</script>

<div class="terminal-output">
	<!-- ツールバー -->
	<div class="toolbar">
		<div class="toolbar-left">
			<span class="message-count">
				{filteredMessages.length} メッセージ
				{#if $terminal.searchTerm}
					(フィルタ済み)
				{/if}
			</span>
			{#if selectedMessages.size > 0}
				<span class="selection-count">
					{selectedMessages.size} 件選択中
				</span>
			{/if}
		</div>
		
		<div class="toolbar-right">
			<button
				class="toolbar-button"
				on:click={toggleSelectAll}
				disabled={filteredMessages.length === 0}
				title="全選択/全選択解除"
			>
				<svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
					<path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
				</svg>
			</button>
			
			<button
				class="toolbar-button"
				on:click={copySelectedMessages}
				disabled={selectedMessages.size === 0}
				title="選択したメッセージをコピー"
			>
				<svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
					<path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
				</svg>
			</button>
			
			<button
				class="toolbar-button"
				on:click={scrollToBottom}
				title="最下部にスクロール"
			>
				<svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
					<path d="M7.41 8.59L12 13.17l4.59-4.58L18 10l-6 6-6-6 1.41-1.41z"/>
				</svg>
			</button>
			
			<button
				class="toolbar-button"
				on:click={clearOutput}
				disabled={messages.length === 0}
				title="出力をクリア"
			>
				<svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
					<path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
				</svg>
			</button>
		</div>
	</div>

	<!-- メッセージ出力エリア -->
	<div class="output-container" bind:this={outputContainer}>
		{#if filteredMessages.length === 0}
			<div class="empty-state">
				{#if $terminal.searchTerm}
					<p>検索条件に一致するメッセージが見つかりません</p>
					<p class="text-sm text-gray-500 dark:text-terminal-light/70">
						検索キーワード: "{$terminal.searchTerm}"
					</p>
				{:else}
					<p>メッセージはまだありません</p>
					<p class="text-sm text-gray-500 dark:text-terminal-light/70">
						接続してデータを送受信すると、ここにメッセージが表示されます
					</p>
				{/if}
			</div>
		{:else}
			{#each filteredMessages as message (message.id)}
				<div 
					class="message-line {getMessageClass(message.direction)}"
					class:selected={selectedMessages.has(message.id)}
					on:click={(e) => toggleMessageSelection(message.id, e)}
					on:keydown={(e) => {
						if (e.key === 'Enter' || e.key === ' ') {
							toggleMessageSelection(message.id, e);
						}
					}}
					role="button"
					tabindex="0"
				>
					<div class="message-metadata">
						<span class="timestamp">{formatTime(message.timestamp)}</span>
						<span class="direction-indicator {message.direction.toLowerCase()}">
							{message.direction === 'Sent' ? '▲' : '▼'}
						</span>
					</div>
					
					<div class="message-content">
						{@html processMessageContent(message.content)}
					</div>
					
					<div class="message-info">
						<span class="encoding">{message.encoding}</span>
						<span class="byte-count">{message.content.length} bytes</span>
					</div>
				</div>
			{/each}
		{/if}
	</div>
</div>

<style>
	.terminal-output {
		@apply flex flex-col h-full bg-white dark:bg-terminal-dark 
		       border border-gray-200 dark:border-terminal-gray rounded-lg overflow-hidden;
	}

	.toolbar {
		@apply flex items-center justify-between p-3 
		       bg-gray-50 dark:bg-terminal-darker
		       border-b border-gray-200 dark:border-terminal-gray;
	}

	.toolbar-left {
		@apply flex items-center space-x-4;
	}

	.toolbar-right {
		@apply flex items-center space-x-2;
	}

	.message-count,
	.selection-count {
		@apply text-sm font-medium text-gray-600 dark:text-terminal-light;
	}

	.selection-count {
		@apply text-blue-600 dark:text-blue-400;
	}

	.toolbar-button {
		@apply p-2 text-gray-600 dark:text-terminal-light 
		       hover:bg-gray-200 dark:hover:bg-terminal-gray/20
		       rounded transition-colors
		       focus:outline-none focus:ring-2 focus:ring-blue-500
		       disabled:opacity-50 disabled:cursor-not-allowed;
	}

	.output-container {
		@apply flex-1 overflow-y-auto p-2 
		       font-mono text-sm leading-relaxed
		       bg-terminal-darker dark:bg-terminal-darker;
		scrollbar-width: thin;
		scrollbar-color: theme('colors.gray.400') theme('colors.gray.200');
	}

	.output-container::-webkit-scrollbar {
		@apply w-2;
	}

	.output-container::-webkit-scrollbar-track {
		@apply bg-gray-200 dark:bg-terminal-gray;
	}

	.output-container::-webkit-scrollbar-thumb {
		@apply bg-gray-400 dark:bg-terminal-light/30 rounded;
	}

	.output-container::-webkit-scrollbar-thumb:hover {
		@apply bg-gray-500 dark:bg-terminal-light/50;
	}

	.empty-state {
		@apply flex flex-col items-center justify-center h-full
		       text-gray-500 dark:text-terminal-light/70 text-center;
	}

	.message-line {
		@apply flex items-start space-x-3 p-2 rounded 
		       hover:bg-gray-50 dark:hover:bg-terminal-gray/10
		       cursor-pointer transition-colors
		       border-l-2 border-transparent;
	}

	.message-line.selected {
		@apply bg-blue-50 dark:bg-blue-900/20 border-l-blue-500 dark:border-l-blue-400;
	}

	.message-line.message-sent {
		@apply border-l-green-500 dark:border-l-green-400;
	}

	.message-line.message-received {
		@apply border-l-blue-500 dark:border-l-blue-400;
	}

	.message-metadata {
		@apply flex flex-col items-center space-y-1 flex-shrink-0 w-20;
	}

	.timestamp {
		@apply text-xs text-gray-500 dark:text-terminal-light/70 font-medium;
	}

	.direction-indicator {
		@apply text-xs font-bold;
	}

	.direction-indicator.sent {
		@apply text-green-600 dark:text-green-400;
	}

	.direction-indicator.received {
		@apply text-blue-600 dark:text-blue-400;
	}

	.message-content {
		@apply flex-1 whitespace-pre-wrap break-words
		       text-gray-900 dark:text-terminal-white;
	}

	.message-info {
		@apply flex flex-col items-end space-y-1 flex-shrink-0 text-xs
		       text-gray-400 dark:text-terminal-light/50;
	}

	.encoding,
	.byte-count {
		@apply font-medium;
	}

	/* 制御文字の可視化スタイル */
	:global(.control-char) {
		@apply bg-yellow-100 dark:bg-yellow-900/30 text-yellow-800 dark:text-yellow-300
		       px-1 rounded text-xs font-bold;
	}

	/* ダークモード対応 */
	@media (prefers-color-scheme: dark) {
		.terminal-output {
			@apply bg-terminal-dark border-terminal-gray;
		}
		
		.toolbar {
			@apply bg-terminal-darker border-terminal-gray;
		}
		
		.message-count,
		.selection-count {
			@apply text-terminal-light;
		}
		
		.selection-count {
			@apply text-blue-400;
		}
		
		.toolbar-button {
			@apply text-terminal-light hover:bg-terminal-gray/20;
		}
		
		.output-container {
			@apply bg-terminal-darker;
		}
		
		.empty-state {
			@apply text-terminal-light/70;
		}
		
		.message-line {
			@apply hover:bg-terminal-gray/10;
		}
		
		.message-line.selected {
			@apply bg-blue-900/20 border-l-blue-400;
		}
		
		.timestamp {
			@apply text-terminal-light/70;
		}
		
		.direction-indicator.sent {
			@apply text-green-400;
		}
		
		.direction-indicator.received {
			@apply text-blue-400;
		}
		
		.message-content {
			@apply text-terminal-white;
		}
		
		.message-info {
			@apply text-terminal-light/50;
		}
	}
</style>