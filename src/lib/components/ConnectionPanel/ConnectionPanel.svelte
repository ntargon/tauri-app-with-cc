<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { connection, notifications } from '$lib/stores';
	import SerialConfig from './SerialConfig.svelte';
	import TcpConfig from './TcpConfig.svelte';
	import type { ConnectionConfig, ConnectionType, SerialConfig as SerialConfigType, TcpConfig as TcpConfigType, ApiResponse } from '$lib/types';

	// 現在の設定状態
	let currentConfig: ConnectionConfig = {
		id: '',
		name: '新規接続',
		connection_type: 'Serial',
		serial_config: {
			port: '',
			baud_rate: 115200,
			data_bits: 8,
			parity: 'None',
			stop_bits: 'One',
			flow_control: 'None',
			timeout_ms: 1000
		},
		tcp_config: {
			host: '192.168.1.1',
			port: 23,
			connect_timeout_sec: 10,
			read_timeout_sec: 30,
			keep_alive: true,
			keep_alive_interval_sec: 60,
			auto_reconnect: false,
			reconnect_interval_sec: 5,
			max_reconnect_attempts: 3
		},
		description: '',
		group_id: null,
		created_at: new Date().toISOString(),
		updated_at: new Date().toISOString()
	};

	// 接続状態
	let isConnecting = false;

	// 接続タイプ選択肢
	const CONNECTION_TYPES: { value: ConnectionType; label: string }[] = [
		{ value: 'Serial', label: 'シリアル通信' },
		{ value: 'Tcp', label: 'TCP通信' }
	];

	// 現在の接続状態を購読
	$: connectionState = $connection;
	$: isConnected = connectionState.status === 'connected';
	$: currentConnectionName = connectionState.config?.name || '';

	// 接続実行
	async function handleConnect() {
		if (isConnected) {
			// 切断処理
			await handleDisconnect();
			return;
		}

		// 設定の検証
		if (!validateConfig()) {
			return;
		}

		isConnecting = true;

		try {
			const response: ApiResponse<string> = await invoke('connect_device', {
				config: currentConfig
			});

			if (response.success) {
				notifications.success('接続成功', `${currentConfig.name}に接続しました`);
			} else {
				console.error('接続エラー:', response.error);
				notifications.error('接続エラー', response.error || '接続に失敗しました');
			}
		} catch (error) {
			console.error('接続エラー:', error);
			notifications.error('接続エラー', '接続処理でエラーが発生しました');
		} finally {
			isConnecting = false;
		}
	}

	// 切断実行
	async function handleDisconnect() {
		try {
			const response: ApiResponse<string> = await invoke('disconnect_device');

			if (response.success) {
				notifications.info('切断完了', '接続を切断しました');
			} else {
				console.error('切断エラー:', response.error);
				notifications.error('切断エラー', response.error || '切断に失敗しました');
			}
		} catch (error) {
			console.error('切断エラー:', error);
			notifications.error('切断エラー', '切断処理でエラーが発生しました');
		}
	}

	// 設定の検証
	function validateConfig(): boolean {
		if (!currentConfig.name.trim()) {
			notifications.error('設定エラー', '接続名を入力してください');
			return false;
		}

		if (currentConfig.connection_type === 'Serial') {
			if (!currentConfig.serial_config.port) {
				notifications.error('設定エラー', 'シリアルポートを選択してください');
				return false;
			}
		} else if (currentConfig.connection_type === 'Tcp') {
			if (!currentConfig.tcp_config.host.trim()) {
				notifications.error('設定エラー', 'ホスト/IPアドレスを入力してください');
				return false;
			}
			if (currentConfig.tcp_config.port <= 0 || currentConfig.tcp_config.port > 65535) {
				notifications.error('設定エラー', '有効なポート番号を入力してください');
				return false;
			}
		}

		return true;
	}

	// 接続タイプ変更ハンドラー
	function handleConnectionTypeChange(type: ConnectionType) {
		currentConfig = { ...currentConfig, connection_type: type };
	}

	// シリアル設定変更ハンドラー
	function handleSerialConfigChange(config: SerialConfigType) {
		currentConfig = { 
			...currentConfig, 
			serial_config: config,
			updated_at: new Date().toISOString()
		};
	}

	// TCP設定変更ハンドラー
	function handleTcpConfigChange(config: TcpConfigType) {
		currentConfig = { 
			...currentConfig, 
			tcp_config: config,
			updated_at: new Date().toISOString()
		};
	}

	// 設定リセット
	function resetConfig() {
		currentConfig = {
			...currentConfig,
			serial_config: {
				port: '',
				baud_rate: 115200,
				data_bits: 8,
				parity: 'None',
				stop_bits: 'One',
				flow_control: 'None',
				timeout_ms: 1000
			},
			tcp_config: {
				host: '',
				port: 23,
				connect_timeout_sec: 10,
				read_timeout_sec: 30,
				keep_alive: true,
				keep_alive_interval_sec: 60,
				auto_reconnect: false,
				reconnect_interval_sec: 5,
				max_reconnect_attempts: 3
			},
			updated_at: new Date().toISOString()
		};
	}
</script>

<div class="connection-panel">
	<!-- パネルヘッダー -->
	<div class="panel-header">
		<h2 class="panel-title">接続設定</h2>
		{#if isConnected}
			<div class="connection-status connected">
				<div class="status-indicator"></div>
				<span class="status-text">{currentConnectionName}に接続中</span>
			</div>
		{:else}
			<div class="connection-status disconnected">
				<div class="status-indicator"></div>
				<span class="status-text">未接続</span>
			</div>
		{/if}
	</div>

	<!-- 基本設定 -->
	<div class="config-section">
		<h3 class="section-title">基本設定</h3>
		
		<!-- 接続名 -->
		<div class="form-group">
			<label for="connection-name" class="form-label">接続名</label>
			<input
				type="text"
				id="connection-name"
				class="form-input"
				placeholder="接続の名前を入力"
				bind:value={currentConfig.name}
				disabled={isConnected}
			/>
		</div>

		<!-- 説明 -->
		<div class="form-group">
			<label for="description" class="form-label">説明 (オプション)</label>
			<textarea
				id="description"
				class="form-textarea"
				rows="2"
				placeholder="接続の説明を入力"
				bind:value={currentConfig.description}
				disabled={isConnected}
			></textarea>
		</div>

		<!-- 接続タイプ選択 -->
		<div class="form-group">
			<label class="form-label">接続タイプ</label>
			<div class="connection-type-selector">
				{#each CONNECTION_TYPES as type}
					<button
						type="button"
						class="type-button"
						class:active={currentConfig.connection_type === type.value}
						on:click={() => handleConnectionTypeChange(type.value)}
						disabled={isConnected}
					>
						{type.label}
					</button>
				{/each}
			</div>
		</div>
	</div>

	<!-- 詳細設定 -->
	{#if currentConfig.connection_type === 'Serial'}
		<SerialConfig
			config={currentConfig.serial_config}
			onConfigChange={handleSerialConfigChange}
			disabled={isConnected}
		/>
	{:else if currentConfig.connection_type === 'Tcp'}
		<TcpConfig
			config={currentConfig.tcp_config}
			onConfigChange={handleTcpConfigChange}
			disabled={isConnected}
		/>
	{/if}

	<!-- アクションボタン -->
	<div class="action-buttons">
		{#if isConnected}
			<button
				type="button"
				class="button button-danger"
				on:click={handleDisconnect}
				disabled={isConnecting}
			>
				切断
			</button>
		{:else}
			<button
				type="button"
				class="button button-primary"
				on:click={handleConnect}
				disabled={isConnecting}
			>
				{#if isConnecting}
					<svg class="animate-spin w-4 h-4 mr-2" viewBox="0 0 24 24">
						<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
						<path class="opacity-75" fill="currentColor" d="m4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
					</svg>
					接続中...
				{:else}
					接続
				{/if}
			</button>

			<button
				type="button"
				class="button button-secondary"
				on:click={resetConfig}
				disabled={isConnecting}
			>
				リセット
			</button>
		{/if}
	</div>
</div>

<style>
	.connection-panel {
		@apply bg-white dark:bg-terminal-dark border border-gray-200 dark:border-terminal-gray
		       rounded-lg shadow-sm overflow-hidden;
	}

	.panel-header {
		@apply flex items-center justify-between p-4 border-b border-gray-200 dark:border-terminal-gray
		       bg-gray-50 dark:bg-terminal-darker;
	}

	.panel-title {
		@apply text-lg font-semibold text-gray-900 dark:text-terminal-white;
	}

	.connection-status {
		@apply flex items-center space-x-2;
	}

	.status-indicator {
		@apply w-3 h-3 rounded-full;
	}

	.connection-status.connected .status-indicator {
		@apply bg-green-500 animate-pulse;
	}

	.connection-status.disconnected .status-indicator {
		@apply bg-gray-400 dark:bg-terminal-gray;
	}

	.status-text {
		@apply text-sm font-medium;
	}

	.connection-status.connected .status-text {
		@apply text-green-700 dark:text-green-400;
	}

	.connection-status.disconnected .status-text {
		@apply text-gray-600 dark:text-terminal-light;
	}

	.config-section {
		@apply p-6 space-y-4;
	}

	.section-title {
		@apply text-lg font-semibold text-gray-900 dark:text-terminal-white mb-4 
		       border-b border-gray-200 dark:border-terminal-gray pb-2;
	}

	.form-group {
		@apply space-y-2;
	}

	.form-label {
		@apply block text-sm font-medium text-gray-700 dark:text-terminal-light;
	}

	.form-input,
	.form-textarea {
		@apply w-full px-3 py-2 border border-gray-300 dark:border-terminal-gray 
		       bg-white dark:bg-terminal-darker text-gray-900 dark:text-terminal-white
		       rounded-md shadow-sm transition-colors
		       focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500
		       disabled:bg-gray-50 dark:disabled:bg-terminal-darker/50 disabled:text-gray-500;
	}

	.form-textarea {
		@apply resize-none;
	}

	.connection-type-selector {
		@apply flex rounded-md overflow-hidden border border-gray-300 dark:border-terminal-gray;
	}

	.type-button {
		@apply flex-1 px-4 py-2 text-sm font-medium text-gray-700 dark:text-terminal-light
		       bg-white dark:bg-terminal-darker hover:bg-gray-50 dark:hover:bg-terminal-gray/20
		       border-r border-gray-300 dark:border-terminal-gray last:border-r-0
		       transition-colors focus:outline-none focus:z-10 focus:ring-2 focus:ring-blue-500
		       disabled:opacity-50 disabled:cursor-not-allowed;
	}

	.type-button.active {
		@apply bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300
		       border-blue-300 dark:border-blue-700;
	}

	.action-buttons {
		@apply flex gap-3 p-6 pt-0;
	}

	.button {
		@apply px-4 py-2 rounded-md font-medium transition-colors
		       focus:outline-none focus:ring-2 focus:ring-offset-2
		       disabled:opacity-50 disabled:cursor-not-allowed
		       flex items-center justify-center;
	}

	.button-primary {
		@apply bg-blue-600 hover:bg-blue-700 text-white
		       focus:ring-blue-500;
	}

	.button-secondary {
		@apply bg-gray-200 hover:bg-gray-300 text-gray-800
		       dark:bg-terminal-gray dark:hover:bg-terminal-gray/80 dark:text-terminal-white
		       focus:ring-gray-500;
	}

	.button-danger {
		@apply bg-red-600 hover:bg-red-700 text-white
		       focus:ring-red-500;
	}

	/* ダークモード対応 */
	@media (prefers-color-scheme: dark) {
		.connection-panel {
			@apply bg-terminal-dark border-terminal-gray;
		}
		
		.panel-header {
			@apply border-terminal-gray bg-terminal-darker;
		}
		
		.panel-title {
			@apply text-terminal-white;
		}
		
		.section-title {
			@apply text-terminal-white border-terminal-gray;
		}
		
		.form-label {
			@apply text-terminal-light;
		}
		
		.form-input,
		.form-textarea {
			@apply bg-terminal-darker border-terminal-gray text-terminal-white;
		}
		
		.connection-type-selector {
			@apply border-terminal-gray;
		}
		
		.type-button {
			@apply bg-terminal-darker text-terminal-light border-terminal-gray
			       hover:bg-terminal-gray/20;
		}
		
		.type-button.active {
			@apply bg-blue-900/30 text-blue-300 border-blue-700;
		}
		
		.button-secondary {
			@apply bg-terminal-gray hover:bg-terminal-gray/80 text-terminal-white;
		}
	}
</style>