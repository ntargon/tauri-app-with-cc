<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { terminal } from '$lib/stores';
	import TerminalOutput from './TerminalOutput.svelte';
	import TerminalInput from './TerminalInput.svelte';
	import type { TerminalMessage } from '$lib/types';

	// イベントリスナーの参照
	let unlistenMessageReceived: UnlistenFn | null = null;

	// ターミナル設定
	let autoScroll = true;
	let maxLines = 1000;
	let showTimestamps = true;
	let searchTerm = '';

	// 検索とフィルタリング
	let searchVisible = false;

	// コンポーネントマウント時にイベントリスナーを設定
	onMount(async () => {
		try {
			// Tauriからのメッセージ受信イベントをリッスン
			unlistenMessageReceived = await listen<TerminalMessage>('terminal-message-received', (event) => {
				const message = event.payload;
				console.log('メッセージ受信:', message);
				
				// ストアにメッセージを追加
				terminal.addMessage(message);
			});

			console.log('ターミナルイベントリスナー設定完了');
		} catch (error) {
			console.error('イベントリスナー設定エラー:', error);
		}
	});

	// コンポーネント破棄時にイベントリスナーをクリーンアップ
	onDestroy(() => {
		if (unlistenMessageReceived) {
			unlistenMessageReceived();
		}
	});

	// 検索処理
	function handleSearch(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			terminal.setSearchTerm(searchTerm);
		} else if (event.key === 'Escape') {
			searchTerm = '';
			terminal.setSearchTerm('');
			searchVisible = false;
		}
	}

	// 検索をクリア
	function clearSearch() {
		searchTerm = '';
		terminal.setSearchTerm('');
	}

	// 検索の表示/非表示切り替え
	function toggleSearch() {
		searchVisible = !searchVisible;
		if (!searchVisible) {
			clearSearch();
		}
	}

	// ターミナルをクリア
	function clearTerminal() {
		terminal.clearMessages();
	}

	// エクスポート機能
	async function exportMessages() {
		try {
			const messages = $terminal.messages;
			if (messages.length === 0) {
				alert('エクスポートするメッセージがありません');
				return;
			}

			// CSV形式でエクスポート
			const csv = messages.map(msg => {
				const timestamp = new Date(msg.timestamp).toISOString();
				const direction = msg.direction === 'Sent' ? '送信' : '受信';
				const content = `"${msg.content.replace(/"/g, '""')}"`;
				const encoding = msg.encoding;
				return `${timestamp},${direction},${content},${encoding}`;
			}).join('\n');

			const header = 'Timestamp,Direction,Content,Encoding\n';
			const fullCsv = header + csv;

			// ダウンロード
			const blob = new Blob([fullCsv], { type: 'text/csv;charset=utf-8;' });
			const link = document.createElement('a');
			const url = URL.createObjectURL(blob);
			
			link.setAttribute('href', url);
			link.setAttribute('download', `terminal_log_${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.csv`);
			link.style.visibility = 'hidden';
			
			document.body.appendChild(link);
			link.click();
			document.body.removeChild(link);
			
			URL.revokeObjectURL(url);
		} catch (error) {
			console.error('エクスポートエラー:', error);
			alert('エクスポートに失敗しました');
		}
	}

	// 設定の保存
	function saveSettings() {
		// 設定をlocalStorageに保存
		const settings = {
			autoScroll,
			maxLines,
			showTimestamps
		};
		localStorage.setItem('terminal_settings', JSON.stringify(settings));
	}

	// 設定の読み込み
	function loadSettings() {
		try {
			const saved = localStorage.getItem('terminal_settings');
			if (saved) {
				const settings = JSON.parse(saved);
				autoScroll = settings.autoScroll ?? true;
				maxLines = settings.maxLines ?? 1000;
				showTimestamps = settings.showTimestamps ?? true;
			}
		} catch (error) {
			console.error('設定読み込みエラー:', error);
		}
	}

	// 初期化時に設定を読み込み
	onMount(() => {
		loadSettings();
	});

	// 設定変更時に保存
	$: if (typeof autoScroll !== 'undefined') saveSettings();
	$: if (typeof maxLines !== 'undefined') saveSettings();
	$: if (typeof showTimestamps !== 'undefined') saveSettings();

	// ストア状態の購読
	$: messageCount = $terminal.messages.length;
	$: filteredCount = $terminal.searchTerm 
		? $terminal.messages.filter(msg => 
			msg.content.toLowerCase().includes($terminal.searchTerm.toLowerCase())
		).length 
		: messageCount;
</script>

<div class="terminal">
	<!-- ターミナルヘッダー -->
	<div class="terminal-header">
		<div class="header-left">
			<h2 class="terminal-title">ターミナル</h2>
			<div class="message-counter">
				{#if $terminal.searchTerm}
					{filteredCount} / {messageCount} メッセージ
				{:else}
					{messageCount} メッセージ
				{/if}
			</div>
		</div>
		
		<div class="header-right">
			<!-- 検索ボタン -->
			<button
				class="header-button"
				class:active={searchVisible}
				on:click={toggleSearch}
				title="検索 (Ctrl+F)"
			>
				<svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
					<path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
				</svg>
			</button>

			<!-- 設定ボタン -->
			<div class="settings-dropdown">
				<button class="header-button" title="設定">
					<svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
						<path d="M12 8c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm0 2c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm0 6c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2z"/>
					</svg>
				</button>
				
				<div class="dropdown-content">
					<label class="dropdown-item">
						<input type="checkbox" bind:checked={autoScroll} />
						自動スクロール
					</label>
					<label class="dropdown-item">
						<input type="checkbox" bind:checked={showTimestamps} />
						タイムスタンプ表示
					</label>
					<div class="dropdown-divider"></div>
					<label class="dropdown-item">
						最大行数:
						<input 
							type="number" 
							bind:value={maxLines} 
							min="100" 
							max="10000" 
							step="100"
							class="number-input"
						/>
					</label>
				</div>
			</div>

			<!-- エクスポートボタン -->
			<button
				class="header-button"
				on:click={exportMessages}
				disabled={messageCount === 0}
				title="ログをエクスポート"
			>
				<svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
					<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8s0 0 0 0l-6-6zM6 4h7v5h5v10H6V4zm11 7h-4V6l4 4-4-4z"/>
				</svg>
			</button>

			<!-- クリアボタン -->
			<button
				class="header-button"
				on:click={clearTerminal}
				disabled={messageCount === 0}
				title="ターミナルをクリア"
			>
				<svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
					<path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
				</svg>
			</button>
		</div>
	</div>

	<!-- 検索バー -->
	{#if searchVisible}
		<div class="search-bar">
			<div class="search-container">
				<svg class="search-icon" viewBox="0 0 24 24" fill="currentColor">
					<path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
				</svg>
				<input
					type="text"
					class="search-input"
					placeholder="メッセージを検索... (Enter: 検索, Esc: 終了)"
					bind:value={searchTerm}
					on:keydown={handleSearch}
					on:input={() => terminal.setSearchTerm(searchTerm)}
				/>
				{#if searchTerm}
					<button class="clear-search" on:click={clearSearch} title="検索をクリア">
						<svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
							<path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
						</svg>
					</button>
				{/if}
			</div>
		</div>
	{/if}

	<!-- ターミナルコンテンツ -->
	<div class="terminal-content">
		<!-- 出力エリア -->
		<div class="output-area">
			<TerminalOutput 
				{autoScroll} 
				{maxLines}
			/>
		</div>

		<!-- 入力エリア -->
		<div class="input-area">
			<TerminalInput />
		</div>
	</div>
</div>

<style>
	.terminal {
		display: flex;
		flex-direction: column;
		height: 100%;
		background-color: #f9fafb;
	}

	.terminal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 1rem;
		background-color: white;
		border-bottom: 1px solid #e5e7eb;
		box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
	}

	.header-left {
		display: flex;
		align-items: center;
		gap: 1rem;
	}

	.terminal-title {
		font-size: 1.125rem;
		font-weight: 600;
		color: #111827;
	}

	.message-counter {
		font-size: 0.875rem;
		color: #6b7280;
		background-color: #f3f4f6;
		padding: 0.25rem 0.75rem;
		border-radius: 9999px;
	}

	.header-right {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.header-button {
		padding: 0.5rem;
		color: #6b7280;
		background-color: transparent;
		border: none;
		border-radius: 0.375rem;
		transition: all 0.2s;
		cursor: pointer;
	}

	.header-button:hover:not(:disabled) {
		background-color: #f3f4f6;
	}

	.header-button:focus {
		outline: none;
		box-shadow: 0 0 0 2px #3b82f6;
	}

	.header-button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.header-button.active {
		background-color: #dbeafe;
		color: #2563eb;
	}

	.settings-dropdown {
		position: relative;
	}

	.settings-dropdown:hover .dropdown-content {
		display: block;
	}

	.dropdown-content {
		display: none;
		position: absolute;
		right: 0;
		top: 100%;
		margin-top: 0.25rem;
		background-color: white;
		border: 1px solid #e5e7eb;
		border-radius: 0.375rem;
		box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
		z-index: 10;
		min-width: 12rem;
	}

	.dropdown-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0.5rem 1rem;
		font-size: 0.875rem;
		color: #374151;
		cursor: pointer;
	}

	.dropdown-item:hover {
		background-color: #f9fafb;
	}

	.dropdown-item input[type="checkbox"] {
		width: 1rem;
		height: 1rem;
		color: #2563eb;
		border: 1px solid #d1d5db;
		background-color: white;
		border-radius: 0.25rem;
	}

	.dropdown-item input[type="checkbox"]:focus {
		box-shadow: 0 0 0 2px #3b82f6;
	}

	.number-input {
		width: 4rem;
		padding: 0.125rem 0.5rem;
		font-size: 0.75rem;
		border: 1px solid #d1d5db;
		background-color: white;
		color: #111827;
		border-radius: 0.25rem;
	}

	.number-input:focus {
		outline: none;
		box-shadow: 0 0 0 1px #3b82f6;
	}

	.dropdown-divider {
		height: 1px;
		background-color: #e5e7eb;
		margin: 0 0.5rem;
	}

	.search-bar {
		padding: 0.75rem;
		background-color: white;
		border-bottom: 1px solid #e5e7eb;
	}

	.search-container {
		position: relative;
		display: flex;
		align-items: center;
	}

	.search-icon {
		position: absolute;
		left: 0.75rem;
		width: 1rem;
		height: 1rem;
		color: #9ca3af;
	}

	.search-input {
		width: 100%;
		padding: 0.5rem 2.5rem 0.5rem 2.5rem;
		border: 1px solid #d1d5db;
		background-color: white;
		color: #111827;
		border-radius: 0.375rem;
	}

	.search-input:focus {
		outline: none;
		border-color: #3b82f6;
		box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
	}

	.clear-search {
		position: absolute;
		right: 0.75rem;
		padding: 0.25rem;
		color: #9ca3af;
		background-color: transparent;
		border: none;
		border-radius: 0.25rem;
		transition: color 0.2s;
		cursor: pointer;
	}

	.clear-search:hover {
		color: #6b7280;
	}

	.terminal-content {
		display: flex;
		flex-direction: column;
		flex: 1;
		gap: 1rem;
		padding: 1rem;
		overflow: hidden;
	}

	.output-area {
		flex: 1;
		overflow: hidden;
	}

	.input-area {
		flex-shrink: 0;
	}

	/* ダークモード対応 */
	@media (prefers-color-scheme: dark) {
		.terminal {
			background-color: #1f2937;
		}
		
		.terminal-header {
			background-color: #374151;
			border-color: #4b5563;
		}
		
		.terminal-title {
			color: #f9fafb;
		}
		
		.message-counter {
			color: #d1d5db;
			background-color: rgba(75, 85, 99, 0.2);
		}
		
		.header-button {
			color: #d1d5db;
		}

		.header-button:hover:not(:disabled) {
			background-color: rgba(75, 85, 99, 0.2);
		}
		
		.header-button.active {
			background-color: rgba(37, 99, 235, 0.3);
			color: #60a5fa;
		}
		
		.dropdown-content {
			background-color: #374151;
			border-color: #4b5563;
		}
		
		.dropdown-item {
			color: #d1d5db;
		}

		.dropdown-item:hover {
			background-color: rgba(75, 85, 99, 0.2);
		}
		
		.dropdown-item input[type="checkbox"] {
			border-color: #4b5563;
			background-color: #1f2937;
		}
		
		.number-input {
			border-color: #4b5563;
			background-color: #1f2937;
			color: #f9fafb;
		}
		
		.dropdown-divider {
			background-color: #4b5563;
		}
		
		.search-bar {
			background-color: #374151;
			border-color: #4b5563;
		}
		
		.search-icon {
			color: rgba(209, 213, 219, 0.5);
		}
		
		.search-input {
			border-color: #4b5563;
			background-color: #1f2937;
			color: #f9fafb;
		}
		
		.clear-search {
			color: rgba(209, 213, 219, 0.5);
		}

		.clear-search:hover {
			color: #d1d5db;
		}
	}

	/* ユーティリティクラス */
	.w-4 {
		width: 1rem;
	}

	.h-4 {
		height: 1rem;
	}
</style>