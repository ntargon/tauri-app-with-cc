<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { connection, terminal, notifications } from '$lib/stores';
	import type { ApiResponse, LineEnding } from '$lib/types';

	// 状態管理
	let inputText = '';
	let isSending = false;
	let historyIndex = -1;

	// 設定
	let lineEnding: LineEnding = 'CRLF';
	let encoding = 'UTF-8';
	let hexMode = false;

	// 改行コード選択肢
	const LINE_ENDINGS: { value: LineEnding; label: string }[] = [
		{ value: 'None', label: 'なし' },
		{ value: 'CR', label: 'CR (\\r)' },
		{ value: 'LF', label: 'LF (\\n)' },
		{ value: 'CRLF', label: 'CRLF (\\r\\n)' }
	];

	// エンコーディング選択肢
	const ENCODINGS = [
		{ value: 'UTF-8', label: 'UTF-8' },
		{ value: 'ASCII', label: 'ASCII' },
		{ value: 'Shift_JIS', label: 'Shift_JIS' },
		{ value: 'EUC-JP', label: 'EUC-JP' },
		{ value: 'ISO-8859-1', label: 'ISO-8859-1' }
	];

	// よく使用されるコマンド
	const COMMON_COMMANDS = [
		{ label: 'ATコマンド', value: 'AT' },
		{ label: 'バージョン確認', value: 'AT+GMR' },
		{ label: 'ステータス確認', value: 'AT+CIPSTATUS' },
		{ label: 'リセット', value: 'AT+RST' },
		{ label: 'ヘルプ', value: 'help' },
		{ label: 'バージョン', value: 'version' },
		{ label: 'ステータス', value: 'status' },
		{ label: 'リスト', value: 'ls' }
	];

	// 接続状態を購読
	$: isConnected = $connection.status === 'connected';
	$: commandHistory = $terminal.commandHistory;

	// メッセージ送信
	async function sendMessage() {
		if (!inputText.trim() || !isConnected || isSending) return;

		const messageToSend = hexMode ? parseHexInput(inputText) : inputText;
		const finalMessage = appendLineEnding(messageToSend);

		isSending = true;

		try {
			const response: ApiResponse<string> = await invoke('send_message', {
				message: finalMessage
			});

			if (response.success) {
				// コマンド履歴に追加
				terminal.addToHistory(inputText);
				
				// 送信成功のフィードバック
				console.log('メッセージ送信成功');
				
				// 入力をクリア
				inputText = '';
				historyIndex = -1;
			} else {
				console.error('送信エラー:', response.error);
				notifications.error('送信エラー', response.error || 'メッセージの送信に失敗しました');
			}
		} catch (error) {
			console.error('送信エラー:', error);
			notifications.error('送信エラー', 'メッセージの送信中にエラーが発生しました');
		} finally {
			isSending = false;
		}
	}

	// 改行コードを追加
	function appendLineEnding(text: string): string {
		switch (lineEnding) {
			case 'CR':
				return text + '\r';
			case 'LF':
				return text + '\n';
			case 'CRLF':
				return text + '\r\n';
			default:
				return text;
		}
	}

	// HEX入力の解析
	function parseHexInput(hexString: string): string {
		// スペースや区切り文字を除去
		const cleanHex = hexString.replace(/[\s\-:]/g, '');
		
		// HEXとして有効かチェック
		if (!/^[0-9A-Fa-f]*$/.test(cleanHex)) {
			notifications.error('入力エラー', '有効な16進数を入力してください');
			return hexString;
		}

		// 16進数文字列をバイト配列に変換
		const bytes = [];
		for (let i = 0; i < cleanHex.length; i += 2) {
			const hex = cleanHex.substr(i, 2);
			if (hex.length === 2) {
				bytes.push(String.fromCharCode(parseInt(hex, 16)));
			}
		}

		return bytes.join('');
	}

	// キーボードイベント処理
	function handleKeyDown(event: KeyboardEvent) {
		switch (event.key) {
			case 'Enter':
				if (event.ctrlKey) {
					// Ctrl+Enter で改行を挿入
					const textarea = event.target as HTMLTextAreaElement;
					const start = textarea.selectionStart;
					const end = textarea.selectionEnd;
					inputText = inputText.substring(0, start) + '\n' + inputText.substring(end);
					
					// カーソル位置を調整
					setTimeout(() => {
						textarea.selectionStart = textarea.selectionEnd = start + 1;
					}, 0);
				} else {
					// 通常のEnterで送信
					event.preventDefault();
					sendMessage();
				}
				break;
				
			case 'ArrowUp':
				if (event.ctrlKey && commandHistory.length > 0) {
					event.preventDefault();
					historyIndex = Math.min(historyIndex + 1, commandHistory.length - 1);
					inputText = commandHistory[commandHistory.length - 1 - historyIndex] || '';
				}
				break;
				
			case 'ArrowDown':
				if (event.ctrlKey) {
					event.preventDefault();
					if (historyIndex > 0) {
						historyIndex--;
						inputText = commandHistory[commandHistory.length - 1 - historyIndex] || '';
					} else if (historyIndex === 0) {
						historyIndex = -1;
						inputText = '';
					}
				}
				break;
		}
	}

	// よく使用されるコマンドを挿入
	function insertCommand(command: string) {
		inputText = command;
	}

	// 入力をクリア
	function clearInput() {
		inputText = '';
		historyIndex = -1;
	}

	// HEX文字列の検証とフォーマット
	function formatHexInput(value: string): string {
		if (!hexMode) return value;
		
		// HEX文字のみを抽出
		const hexChars = value.replace(/[^0-9A-Fa-f]/g, '');
		
		// 2文字ずつスペースで区切り
		return hexChars.replace(/(.{2})/g, '$1 ').trim();
	}

	// 入力モード切り替え
	function toggleHexMode() {
		hexMode = !hexMode;
		if (hexMode && inputText) {
			// テキストをHEX表示に変換
			inputText = inputText.split('').map(char => 
				char.charCodeAt(0).toString(16).padStart(2, '0').toUpperCase()
			).join(' ');
		} else if (!hexMode && inputText) {
			// HEXをテキストに変換（可能な場合）
			try {
				const cleanHex = inputText.replace(/[\s\-:]/g, '');
				const text = cleanHex.replace(/(.{2})/g, (match) => 
					String.fromCharCode(parseInt(match, 16))
				);
				inputText = text;
			} catch (error) {
				// 変換失敗時は入力をクリア
				inputText = '';
			}
		}
	}
</script>

<div class="terminal-input">
	<!-- 設定パネル -->
	<div class="settings-panel">
		<div class="settings-row">
			<div class="setting-group">
				<label class="setting-label">改行コード:</label>
				<select bind:value={lineEnding} class="setting-select">
					{#each LINE_ENDINGS as option}
						<option value={option.value}>{option.label}</option>
					{/each}
				</select>
			</div>
			
			<div class="setting-group">
				<label class="setting-label">エンコーディング:</label>
				<select bind:value={encoding} class="setting-select">
					{#each ENCODINGS as option}
						<option value={option.value}>{option.label}</option>
					{/each}
				</select>
			</div>
			
			<div class="setting-group">
				<label class="setting-checkbox">
					<input 
						type="checkbox" 
						bind:checked={hexMode} 
						on:change={toggleHexMode}
					/>
					<span class="checkbox-label">16進数モード</span>
				</label>
			</div>
		</div>
	</div>

	<!-- よく使用されるコマンド -->
	{#if !hexMode}
		<div class="commands-panel">
			<span class="commands-label">よく使用されるコマンド:</span>
			<div class="commands-grid">
				{#each COMMON_COMMANDS as cmd}
					<button
						class="command-button"
						on:click={() => insertCommand(cmd.value)}
						disabled={!isConnected}
						title={cmd.value}
					>
						{cmd.label}
					</button>
				{/each}
			</div>
		</div>
	{/if}

	<!-- 入力エリア -->
	<div class="input-area">
		<div class="input-container">
			<textarea
				class="input-field"
				class:hex-mode={hexMode}
				bind:value={inputText}
				on:keydown={handleKeyDown}
				on:input={(e) => {
					if (hexMode) {
						inputText = formatHexInput(e.currentTarget.value);
					}
				}}
				placeholder={hexMode ? '16進数を入力 (例: 41 54 0D 0A)' : 'メッセージを入力... (Enter: 送信, Ctrl+Enter: 改行, Ctrl+↑↓: 履歴)'}
				disabled={!isConnected}
				rows="3"
			></textarea>
			
			<div class="input-actions">
				<button
					class="action-button clear-button"
					on:click={clearInput}
					disabled={!inputText}
					title="入力をクリア"
				>
					<svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
						<path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
					</svg>
				</button>
				
				<button
					class="action-button send-button"
					on:click={sendMessage}
					disabled={!isConnected || !inputText.trim() || isSending}
					title="メッセージを送信 (Enter)"
				>
					{#if isSending}
						<svg class="animate-spin w-4 h-4" viewBox="0 0 24 24">
							<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
							<path class="opacity-75" fill="currentColor" d="m4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
						</svg>
					{:else}
						<svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
							<path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"/>
						</svg>
					{/if}
				</button>
			</div>
		</div>
		
		<!-- 状態表示 -->
		<div class="input-status">
			<div class="status-left">
				{#if !isConnected}
					<span class="status-disconnected">未接続</span>
				{:else}
					<span class="status-connected">接続中</span>
				{/if}
			</div>
			
			<div class="status-right">
				<span class="char-count">
					{inputText.length} 文字
					{#if hexMode}
						({Math.ceil(inputText.replace(/\s/g, '').length / 2)} bytes)
					{/if}
				</span>
			</div>
		</div>
	</div>
</div>

<style>
	.terminal-input {
		@apply bg-white dark:bg-terminal-dark 
		       border border-gray-200 dark:border-terminal-gray 
		       rounded-lg overflow-hidden;
	}

	.settings-panel {
		@apply p-3 bg-gray-50 dark:bg-terminal-darker
		       border-b border-gray-200 dark:border-terminal-gray;
	}

	.settings-row {
		@apply flex flex-wrap items-center gap-4;
	}

	.setting-group {
		@apply flex items-center space-x-2;
	}

	.setting-label {
		@apply text-sm font-medium text-gray-700 dark:text-terminal-light;
	}

	.setting-select {
		@apply px-2 py-1 text-sm border border-gray-300 dark:border-terminal-gray
		       bg-white dark:bg-terminal-darker text-gray-900 dark:text-terminal-white
		       rounded focus:outline-none focus:ring-2 focus:ring-blue-500;
	}

	.setting-checkbox {
		@apply flex items-center cursor-pointer;
	}

	.setting-checkbox input[type="checkbox"] {
		@apply w-4 h-4 text-blue-600 border-gray-300 dark:border-terminal-gray
		       bg-white dark:bg-terminal-darker rounded
		       focus:ring-blue-500;
	}

	.checkbox-label {
		@apply ml-2 text-sm font-medium text-gray-700 dark:text-terminal-light;
	}

	.commands-panel {
		@apply p-3 border-b border-gray-200 dark:border-terminal-gray;
	}

	.commands-label {
		@apply block text-sm font-medium text-gray-700 dark:text-terminal-light mb-2;
	}

	.commands-grid {
		@apply grid grid-cols-2 sm:grid-cols-4 lg:grid-cols-8 gap-2;
	}

	.command-button {
		@apply px-3 py-1 text-xs bg-gray-100 dark:bg-terminal-gray/30
		       hover:bg-gray-200 dark:hover:bg-terminal-gray/50
		       text-gray-700 dark:text-terminal-light
		       border border-gray-300 dark:border-terminal-gray rounded
		       transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500
		       disabled:opacity-50 disabled:cursor-not-allowed;
	}

	.input-area {
		@apply p-3;
	}

	.input-container {
		@apply relative;
	}

	.input-field {
		@apply w-full px-3 py-2 pr-20 border border-gray-300 dark:border-terminal-gray
		       bg-white dark:bg-terminal-darker text-gray-900 dark:text-terminal-white
		       rounded-md resize-none font-mono
		       focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500
		       disabled:bg-gray-50 dark:disabled:bg-terminal-darker/50 disabled:text-gray-500;
	}

	.input-field.hex-mode {
		@apply font-mono tracking-wider uppercase;
	}

	.input-actions {
		@apply absolute right-2 bottom-2 flex space-x-1;
	}

	.action-button {
		@apply p-2 rounded-md transition-colors
		       focus:outline-none focus:ring-2 focus:ring-blue-500
		       disabled:opacity-50 disabled:cursor-not-allowed;
	}

	.clear-button {
		@apply text-gray-500 dark:text-terminal-light/70
		       hover:bg-gray-100 dark:hover:bg-terminal-gray/20
		       hover:text-gray-700 dark:hover:text-terminal-light;
	}

	.send-button {
		@apply text-blue-600 dark:text-blue-400
		       hover:bg-blue-50 dark:hover:bg-blue-900/20
		       hover:text-blue-700 dark:hover:text-blue-300;
	}

	.input-status {
		@apply flex items-center justify-between mt-2 text-sm;
	}

	.status-disconnected {
		@apply text-red-600 dark:text-red-400 font-medium;
	}

	.status-connected {
		@apply text-green-600 dark:text-green-400 font-medium;
	}

	.char-count {
		@apply text-gray-500 dark:text-terminal-light/70;
	}

	/* ダークモード対応 */
	@media (prefers-color-scheme: dark) {
		.terminal-input {
			@apply bg-terminal-dark border-terminal-gray;
		}
		
		.settings-panel,
		.commands-panel {
			@apply bg-terminal-darker border-terminal-gray;
		}
		
		.setting-label,
		.checkbox-label,
		.commands-label {
			@apply text-terminal-light;
		}
		
		.setting-select {
			@apply bg-terminal-darker border-terminal-gray text-terminal-white;
		}
		
		.setting-checkbox input[type="checkbox"] {
			@apply bg-terminal-darker border-terminal-gray;
		}
		
		.command-button {
			@apply bg-terminal-gray/30 hover:bg-terminal-gray/50
			       text-terminal-light border-terminal-gray;
		}
		
		.input-field {
			@apply bg-terminal-darker border-terminal-gray text-terminal-white;
		}
		
		.clear-button {
			@apply text-terminal-light/70 hover:bg-terminal-gray/20 hover:text-terminal-light;
		}
		
		.send-button {
			@apply text-blue-400 hover:bg-blue-900/20 hover:text-blue-300;
		}
		
		.status-disconnected {
			@apply text-red-400;
		}
		
		.status-connected {
			@apply text-green-400;
		}
		
		.char-count {
			@apply text-terminal-light/70;
		}
	}
</style>