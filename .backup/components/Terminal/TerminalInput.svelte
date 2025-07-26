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
		background-color: white;
		border: 1px solid #e5e7eb;
		border-radius: 0.5rem;
		overflow: hidden;
	}

	.settings-panel {
		padding: 0.75rem;
		background-color: #f9fafb;
		border-bottom: 1px solid #e5e7eb;
	}

	.settings-row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 1rem;
	}

	.setting-group {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.setting-label {
		font-size: 0.875rem;
		font-weight: 500;
		color: #374151;
	}

	.setting-select {
		padding: 0.25rem 0.5rem;
		font-size: 0.875rem;
		border: 1px solid #d1d5db;
		background-color: white;
		color: #111827;
		border-radius: 0.25rem;
	}

	.setting-select:focus {
		outline: none;
		box-shadow: 0 0 0 2px #3b82f6;
	}

	.setting-checkbox {
		display: flex;
		align-items: center;
		cursor: pointer;
	}

	.setting-checkbox input[type="checkbox"] {
		width: 1rem;
		height: 1rem;
		color: #2563eb;
		border: 1px solid #d1d5db;
		background-color: white;
		border-radius: 0.25rem;
	}

	.setting-checkbox input[type="checkbox"]:focus {
		box-shadow: 0 0 0 2px #3b82f6;
	}

	.checkbox-label {
		margin-left: 0.5rem;
		font-size: 0.875rem;
		font-weight: 500;
		color: #374151;
	}

	.commands-panel {
		padding: 0.75rem;
		border-bottom: 1px solid #e5e7eb;
	}

	.commands-label {
		display: block;
		font-size: 0.875rem;
		font-weight: 500;
		color: #374151;
		margin-bottom: 0.5rem;
	}

	.commands-grid {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 0.5rem;
	}

	@media (min-width: 640px) {
		.commands-grid {
			grid-template-columns: repeat(4, 1fr);
		}
	}

	@media (min-width: 1024px) {
		.commands-grid {
			grid-template-columns: repeat(8, 1fr);
		}
	}

	.command-button {
		padding: 0.25rem 0.75rem;
		font-size: 0.75rem;
		background-color: #f3f4f6;
		color: #374151;
		border: 1px solid #d1d5db;
		border-radius: 0.25rem;
		transition: all 0.2s;
		cursor: pointer;
	}

	.command-button:hover:not(:disabled) {
		background-color: #e5e7eb;
	}

	.command-button:focus {
		outline: none;
		box-shadow: 0 0 0 2px #3b82f6;
	}

	.command-button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.input-area {
		padding: 0.75rem;
	}

	.input-container {
		position: relative;
	}

	.input-field {
		width: 100%;
		padding: 0.5rem 5rem 0.5rem 0.75rem;
		border: 1px solid #d1d5db;
		background-color: white;
		color: #111827;
		border-radius: 0.375rem;
		resize: none;
		font-family: ui-monospace, SFMono-Regular, "SF Mono", Monaco, Consolas, monospace;
	}

	.input-field:focus {
		outline: none;
		border-color: #3b82f6;
		box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
	}

	.input-field:disabled {
		background-color: #f9fafb;
		color: #9ca3af;
	}

	.input-field.hex-mode {
		font-family: ui-monospace, SFMono-Regular, "SF Mono", Monaco, Consolas, monospace;
		letter-spacing: 0.05em;
		text-transform: uppercase;
	}

	.input-actions {
		position: absolute;
		right: 0.5rem;
		bottom: 0.5rem;
		display: flex;
		gap: 0.25rem;
	}

	.action-button {
		padding: 0.5rem;
		border: none;
		border-radius: 0.375rem;
		transition: all 0.2s;
		cursor: pointer;
		background-color: transparent;
	}

	.action-button:focus {
		outline: none;
		box-shadow: 0 0 0 2px #3b82f6;
	}

	.action-button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.clear-button {
		color: #6b7280;
	}

	.clear-button:hover:not(:disabled) {
		background-color: #f3f4f6;
		color: #374151;
	}

	.send-button {
		color: #2563eb;
	}

	.send-button:hover:not(:disabled) {
		background-color: #eff6ff;
		color: #1d4ed8;
	}

	.input-status {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-top: 0.5rem;
		font-size: 0.875rem;
	}

	.status-disconnected {
		color: #dc2626;
		font-weight: 500;
	}

	.status-connected {
		color: #059669;
		font-weight: 500;
	}

	.char-count {
		color: #6b7280;
	}

	/* ダークモード対応 */
	@media (prefers-color-scheme: dark) {
		.terminal-input {
			background-color: #374151;
			border-color: #4b5563;
		}
		
		.settings-panel,
		.commands-panel {
			background-color: #1f2937;
			border-color: #4b5563;
		}
		
		.setting-label,
		.checkbox-label,
		.commands-label {
			color: #d1d5db;
		}
		
		.setting-select {
			background-color: #1f2937;
			border-color: #4b5563;
			color: #f9fafb;
		}
		
		.setting-checkbox input[type="checkbox"] {
			background-color: #1f2937;
			border-color: #4b5563;
		}
		
		.command-button {
			background-color: rgba(75, 85, 99, 0.3);
			color: #d1d5db;
			border-color: #4b5563;
		}

		.command-button:hover:not(:disabled) {
			background-color: rgba(75, 85, 99, 0.5);
		}
		
		.input-field {
			background-color: #1f2937;
			border-color: #4b5563;
			color: #f9fafb;
		}

		.input-field:disabled {
			background-color: rgba(31, 41, 55, 0.5);
			color: #9ca3af;
		}
		
		.clear-button {
			color: rgba(209, 213, 219, 0.7);
		}

		.clear-button:hover:not(:disabled) {
			background-color: rgba(75, 85, 99, 0.2);
			color: #d1d5db;
		}
		
		.send-button {
			color: #60a5fa;
		}

		.send-button:hover:not(:disabled) {
			background-color: rgba(37, 99, 235, 0.2);
			color: #93c5fd;
		}
		
		.status-disconnected {
			color: #f87171;
		}
		
		.status-connected {
			color: #10b981;
		}
		
		.char-count {
			color: rgba(209, 213, 219, 0.7);
		}
	}

	/* アニメーション */
	.animate-spin {
		animation: spin 1s linear infinite;
	}

	.opacity-25 {
		opacity: 0.25;
	}

	.opacity-75 {
		opacity: 0.75;
	}

	.w-4 {
		width: 1rem;
	}

	.h-4 {
		height: 1rem;
	}

	@keyframes spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}
</style>