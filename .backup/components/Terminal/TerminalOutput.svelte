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
		display: flex;
		flex-direction: column;
		height: 100%;
		background-color: white;
		border: 1px solid #e5e7eb;
		border-radius: 0.5rem;
		overflow: hidden;
	}

	.toolbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0.75rem;
		background-color: #f9fafb;
		border-bottom: 1px solid #e5e7eb;
	}

	.toolbar-left {
		display: flex;
		align-items: center;
		gap: 1rem;
	}

	.toolbar-right {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.message-count,
	.selection-count {
		font-size: 0.875rem;
		font-weight: 500;
		color: #6b7280;
	}

	.selection-count {
		color: #2563eb;
	}

	.toolbar-button {
		padding: 0.5rem;
		color: #6b7280;
		background-color: transparent;
		border: none;
		border-radius: 0.25rem;
		transition: all 0.2s;
		cursor: pointer;
	}

	.toolbar-button:hover:not(:disabled) {
		background-color: #e5e7eb;
	}

	.toolbar-button:focus {
		outline: none;
		box-shadow: 0 0 0 2px #3b82f6;
	}

	.toolbar-button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.output-container {
		flex: 1;
		overflow-y: auto;
		padding: 0.5rem;
		font-family: ui-monospace, SFMono-Regular, "SF Mono", Monaco, Consolas, monospace;
		font-size: 0.875rem;
		line-height: 1.625;
		background-color: #1f2937;
		scrollbar-width: thin;
		scrollbar-color: #9ca3af #e5e7eb;
	}

	.output-container::-webkit-scrollbar {
		width: 0.5rem;
	}

	.output-container::-webkit-scrollbar-track {
		background-color: #e5e7eb;
	}

	.output-container::-webkit-scrollbar-thumb {
		background-color: #9ca3af;
		border-radius: 0.25rem;
	}

	.output-container::-webkit-scrollbar-thumb:hover {
		background-color: #6b7280;
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		color: #6b7280;
		text-align: center;
	}

	.message-line {
		display: flex;
		align-items: flex-start;
		gap: 0.75rem;
		padding: 0.5rem;
		border-radius: 0.25rem;
		cursor: pointer;
		transition: all 0.2s;
		border-left: 2px solid transparent;
	}

	.message-line:hover {
		background-color: #f9fafb;
	}

	.message-line.selected {
		background-color: #eff6ff;
		border-left-color: #3b82f6;
	}

	.message-line.message-sent {
		border-left-color: #10b981;
	}

	.message-line.message-received {
		border-left-color: #3b82f6;
	}

	.message-metadata {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.25rem;
		flex-shrink: 0;
		width: 5rem;
	}

	.timestamp {
		font-size: 0.75rem;
		color: #6b7280;
		font-weight: 500;
	}

	.direction-indicator {
		font-size: 0.75rem;
		font-weight: bold;
	}

	.direction-indicator.sent {
		color: #059669;
	}

	.direction-indicator.received {
		color: #2563eb;
	}

	.message-content {
		flex: 1;
		white-space: pre-wrap;
		word-break: break-word;
		color: #111827;
	}

	.message-info {
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		gap: 0.25rem;
		flex-shrink: 0;
		font-size: 0.75rem;
		color: #9ca3af;
	}

	.encoding,
	.byte-count {
		font-weight: 500;
	}

	/* 制御文字の可視化スタイル */
	:global(.control-char) {
		background-color: #fef3c7;
		color: #92400e;
		padding: 0 0.25rem;
		border-radius: 0.25rem;
		font-size: 0.75rem;
		font-weight: bold;
	}

	/* ダークモード対応 */
	@media (prefers-color-scheme: dark) {
		.terminal-output {
			background-color: #374151;
			border-color: #4b5563;
		}
		
		.toolbar {
			background-color: #1f2937;
			border-color: #4b5563;
		}
		
		.message-count,
		.selection-count {
			color: #d1d5db;
		}
		
		.selection-count {
			color: #60a5fa;
		}
		
		.toolbar-button {
			color: #d1d5db;
		}

		.toolbar-button:hover:not(:disabled) {
			background-color: rgba(75, 85, 99, 0.2);
		}
		
		.output-container {
			background-color: #1f2937;
			scrollbar-color: rgba(209, 213, 219, 0.3) #4b5563;
		}

		.output-container::-webkit-scrollbar-track {
			background-color: #4b5563;
		}

		.output-container::-webkit-scrollbar-thumb {
			background-color: rgba(209, 213, 219, 0.3);
		}

		.output-container::-webkit-scrollbar-thumb:hover {
			background-color: rgba(209, 213, 219, 0.5);
		}
		
		.empty-state {
			color: rgba(209, 213, 219, 0.7);
		}
		
		.message-line:hover {
			background-color: rgba(75, 85, 99, 0.1);
		}
		
		.message-line.selected {
			background-color: rgba(37, 99, 235, 0.2);
			border-left-color: #60a5fa;
		}

		.message-line.message-sent {
			border-left-color: #10b981;
		}

		.message-line.message-received {
			border-left-color: #60a5fa;
		}
		
		.timestamp {
			color: rgba(209, 213, 219, 0.7);
		}
		
		.direction-indicator.sent {
			color: #10b981;
		}
		
		.direction-indicator.received {
			color: #60a5fa;
		}
		
		.message-content {
			color: #f9fafb;
		}
		
		.message-info {
			color: rgba(209, 213, 219, 0.5);
		}

		:global(.control-char) {
			background-color: rgba(180, 83, 9, 0.3);
			color: #fbbf24;
		}
	}

	/* その他のクラス定義 */
	.text-sm {
		font-size: 0.875rem;
	}

	.text-gray-500 {
		color: #6b7280;
	}
</style>