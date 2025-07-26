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
		background-color: white;
		border: 1px solid #e5e7eb;
		border-radius: 0.5rem;
		box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
		overflow: hidden;
	}

	.panel-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 1rem;
		border-bottom: 1px solid #e5e7eb;
		background-color: #f9fafb;
	}

	.panel-title {
		font-size: 1.125rem;
		font-weight: 600;
		color: #111827;
	}

	.connection-status {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.status-indicator {
		width: 0.75rem;
		height: 0.75rem;
		border-radius: 50%;
	}

	.connection-status.connected .status-indicator {
		background-color: #10b981;
		animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
	}

	.connection-status.disconnected .status-indicator {
		background-color: #9ca3af;
	}

	.status-text {
		font-size: 0.875rem;
		font-weight: 500;
	}

	.connection-status.connected .status-text {
		color: #047857;
	}

	.connection-status.disconnected .status-text {
		color: #6b7280;
	}

	.config-section {
		padding: 1.5rem;
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.section-title {
		font-size: 1.125rem;
		font-weight: 600;
		color: #111827;
		margin-bottom: 1rem;
		border-bottom: 1px solid #e5e7eb;
		padding-bottom: 0.5rem;
	}

	.form-group {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.form-label {
		display: block;
		font-size: 0.875rem;
		font-weight: 500;
		color: #374151;
	}

	.form-input,
	.form-textarea {
		width: 100%;
		padding: 0.5rem 0.75rem;
		border: 1px solid #d1d5db;
		background-color: white;
		color: #111827;
		border-radius: 0.375rem;
		box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
		transition: all 0.2s;
	}

	.form-input:focus,
	.form-textarea:focus {
		outline: none;
		border-color: #3b82f6;
		box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
	}

	.form-input:disabled,
	.form-textarea:disabled {
		background-color: #f9fafb;
		color: #9ca3af;
		cursor: not-allowed;
	}

	.form-textarea {
		resize: none;
	}

	.connection-type-selector {
		display: flex;
		border-radius: 0.375rem;
		overflow: hidden;
		border: 1px solid #d1d5db;
	}

	.type-button {
		flex: 1;
		padding: 0.5rem 1rem;
		font-size: 0.875rem;
		font-weight: 500;
		color: #374151;
		background-color: white;
		border-right: 1px solid #d1d5db;
		transition: all 0.2s;
		cursor: pointer;
	}

	.type-button:last-child {
		border-right: none;
	}

	.type-button:hover:not(:disabled) {
		background-color: #f9fafb;
	}

	.type-button:focus {
		outline: none;
		z-index: 10;
		box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
	}

	.type-button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.type-button.active {
		background-color: #dbeafe;
		color: #1d4ed8;
		border-color: #93c5fd;
	}

	.action-buttons {
		display: flex;
		gap: 0.75rem;
		padding: 1.5rem;
		padding-top: 0;
	}

	.button {
		padding: 0.5rem 1rem;
		border-radius: 0.375rem;
		font-weight: 500;
		transition: all 0.2s;
		cursor: pointer;
		border: none;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.button:focus {
		outline: none;
		box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
	}

	.button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.button-primary {
		background-color: #2563eb;
		color: white;
	}

	.button-primary:hover:not(:disabled) {
		background-color: #1d4ed8;
	}

	.button-secondary {
		background-color: #e5e7eb;
		color: #1f2937;
	}

	.button-secondary:hover:not(:disabled) {
		background-color: #d1d5db;
	}

	.button-danger {
		background-color: #dc2626;
		color: white;
	}

	.button-danger:hover:not(:disabled) {
		background-color: #b91c1c;
	}

	/* ダークモード対応 */
	@media (prefers-color-scheme: dark) {
		.connection-panel {
			background-color: #374151;
			border-color: #4b5563;
		}
		
		.panel-header {
			border-color: #4b5563;
			background-color: #1f2937;
		}
		
		.panel-title {
			color: #f9fafb;
		}
		
		.section-title {
			color: #f9fafb;
			border-color: #4b5563;
		}
		
		.form-label {
			color: #d1d5db;
		}
		
		.form-input,
		.form-textarea {
			background-color: #1f2937;
			border-color: #4b5563;
			color: #f9fafb;
		}
		
		.form-input:disabled,
		.form-textarea:disabled {
			background-color: rgba(31, 41, 55, 0.5);
			color: #9ca3af;
		}
		
		.connection-type-selector {
			border-color: #4b5563;
		}
		
		.type-button {
			background-color: #1f2937;
			color: #d1d5db;
			border-color: #4b5563;
		}
		
		.type-button:hover:not(:disabled) {
			background-color: rgba(75, 85, 99, 0.2);
		}
		
		.type-button.active {
			background-color: rgba(37, 99, 235, 0.3);
			color: #93c5fd;
			border-color: #3b82f6;
		}
		
		.button-secondary {
			background-color: #4b5563;
			color: #f9fafb;
		}
		
		.button-secondary:hover:not(:disabled) {
			background-color: rgba(75, 85, 99, 0.8);
		}

		.connection-status.disconnected .status-indicator {
			background-color: #4b5563;
		}

		.connection-status.connected .status-text {
			color: #10b981;
		}

		.connection-status.disconnected .status-text {
			color: #d1d5db;
		}
	}

	@keyframes pulse {
		0%, 100% {
			opacity: 1;
		}
		50% {
			opacity: 0.5;
		}
	}

	/* アニメーションとユーティリティクラス */
	.animate-spin {
		animation: spin 1s linear infinite;
	}

	.w-4 {
		width: 1rem;
	}

	.h-4 {
		height: 1rem;
	}

	.mr-2 {
		margin-right: 0.5rem;
	}

	.opacity-25 {
		opacity: 0.25;
	}

	.opacity-75 {
		opacity: 0.75;
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