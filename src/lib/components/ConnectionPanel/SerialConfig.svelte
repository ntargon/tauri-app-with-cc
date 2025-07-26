<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { connection, notifications } from '$lib/stores';
	import type { SerialConfig, ApiResponse } from '$lib/types';

	// シリアルポート情報の型定義
	interface SerialPortInfo {
		port_name: string;
		port_type?: string;
		vid?: number;
		pid?: number;
		serial_number?: string;
		manufacturer?: string;
		product?: string;
	}

	// プロパティ
	export let config: SerialConfig;
	export let onConfigChange: (config: SerialConfig) => void;
	export let disabled = false;

	// 状態管理
	let availablePorts: SerialPortInfo[] = [];
	let isLoadingPorts = false;
	let isRefreshing = false;

	// ボーレート選択肢
	const BAUD_RATES = [
		1200, 2400, 4800, 9600, 14400, 19200, 28800, 38400, 57600, 115200, 230400, 460800, 921600
	];

	// データビット選択肢
	const DATA_BITS = [5, 6, 7, 8];

	// パリティ選択肢
	const PARITY_OPTIONS = [
		{ value: 'None', label: 'なし' },
		{ value: 'Odd', label: '奇数' },
		{ value: 'Even', label: '偶数' }
	];

	// ストップビット選択肢
	const STOP_BITS = [
		{ value: 'One', label: '1' },
		{ value: 'Two', label: '2' }
	];

	// フロー制御選択肢
	const FLOW_CONTROL = [
		{ value: 'None', label: 'なし' },
		{ value: 'Software', label: 'ソフトウェア' },
		{ value: 'Hardware', label: 'ハードウェア' }
	];

	// コンポーネント初期化時にポート一覧を取得
	onMount(() => {
		loadAvailablePorts();
	});

	// 利用可能なポート一覧を取得
	async function loadAvailablePorts() {
		if (isLoadingPorts) return;
		
		isLoadingPorts = true;
		try {
			const response: ApiResponse<SerialPortInfo[]> = await invoke('get_serial_ports_info');
			
			if (response.success && response.data) {
				availablePorts = response.data;
			} else {
				console.error('シリアルポート取得エラー:', response.error);
				notifications.error('ポート取得エラー', response.error || 'シリアルポートの取得に失敗しました');
				availablePorts = [];
			}
		} catch (error) {
			console.error('シリアルポート取得エラー:', error);
			notifications.error('ポート取得エラー', 'シリアルポートの取得に失敗しました');
			availablePorts = [];
		} finally {
			isLoadingPorts = false;
		}
	}

	// ポート一覧を再取得
	async function refreshPorts() {
		if (isRefreshing) return;
		
		isRefreshing = true;
		await loadAvailablePorts();
		
		// 少し待ってからリフレッシュ状態を解除（UIフィードバックのため）
		setTimeout(() => {
			isRefreshing = false;
		}, 500);
	}

	// 設定変更ハンドラー
	function updateConfig(key: keyof SerialConfig, value: any) {
		const newConfig = { ...config, [key]: value };
		onConfigChange(newConfig);
	}

	// ポート情報の表示用フォーマット
	function formatPortInfo(portInfo: SerialPortInfo): string {
		let info = portInfo.port_name;
		
		if (portInfo.manufacturer || portInfo.product) {
			const parts = [];
			if (portInfo.manufacturer) parts.push(portInfo.manufacturer);
			if (portInfo.product) parts.push(portInfo.product);
			info += ` (${parts.join(' - ')})`;
		} else if (portInfo.port_type) {
			info += ` (${portInfo.port_type})`;
		}
		
		return info;
	}

	// 選択されたポートが有効かチェック
	$: selectedPortExists = availablePorts.some(port => port.port_name === config.port);
</script>

<div class="serial-config">
	<div class="config-section">
		<h3 class="section-title">シリアルポート設定</h3>
		
		<!-- ポート選択 -->
		<div class="form-group">
			<label for="port" class="form-label">ポート</label>
			<div class="port-selection">
				<select
					id="port"
					class="form-select"
					class:error={!selectedPortExists && config.port}
					bind:value={config.port}
					on:change={(e) => updateConfig('port', e.currentTarget.value)}
					{disabled}
				>
					<option value="">ポートを選択してください</option>
					{#each availablePorts as portInfo}
						<option value={portInfo.port_name}>
							{formatPortInfo(portInfo)}
						</option>
					{/each}
					{#if config.port && !selectedPortExists}
						<option value={config.port} disabled>
							{config.port} (利用不可)
						</option>
					{/if}
				</select>
				
				<button
					type="button"
					class="refresh-button"
					class:loading={isLoadingPorts || isRefreshing}
					on:click={refreshPorts}
					disabled={disabled || isLoadingPorts}
					title="ポート一覧を更新"
				>
					<svg class="refresh-icon" viewBox="0 0 24 24">
						<path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
					</svg>
				</button>
			</div>
			{#if !selectedPortExists && config.port}
				<div class="form-error">選択されたポートは利用できません</div>
			{/if}
		</div>

		<!-- ボーレート -->
		<div class="form-group">
			<label for="baud-rate" class="form-label">ボーレート</label>
			<select
				id="baud-rate"
				class="form-select"
				bind:value={config.baud_rate}
				on:change={(e) => updateConfig('baud_rate', parseInt(e.currentTarget.value))}
				{disabled}
			>
				{#each BAUD_RATES as rate}
					<option value={rate}>{rate}</option>
				{/each}
			</select>
		</div>

		<!-- データビット -->
		<div class="form-group">
			<label for="data-bits" class="form-label">データビット</label>
			<select
				id="data-bits"
				class="form-select"
				bind:value={config.data_bits}
				on:change={(e) => updateConfig('data_bits', parseInt(e.currentTarget.value))}
				{disabled}
			>
				{#each DATA_BITS as bits}
					<option value={bits}>{bits}</option>
				{/each}
			</select>
		</div>

		<!-- パリティ -->
		<div class="form-group">
			<label for="parity" class="form-label">パリティ</label>
			<select
				id="parity"
				class="form-select"
				bind:value={config.parity}
				on:change={(e) => updateConfig('parity', e.currentTarget.value)}
				{disabled}
			>
				{#each PARITY_OPTIONS as option}
					<option value={option.value}>{option.label}</option>
				{/each}
			</select>
		</div>

		<!-- ストップビット -->
		<div class="form-group">
			<label for="stop-bits" class="form-label">ストップビット</label>
			<select
				id="stop-bits"
				class="form-select"
				bind:value={config.stop_bits}
				on:change={(e) => updateConfig('stop_bits', e.currentTarget.value)}
				{disabled}
			>
				{#each STOP_BITS as option}
					<option value={option.value}>{option.label}</option>
				{/each}
			</select>
		</div>

		<!-- フロー制御 -->
		<div class="form-group">
			<label for="flow-control" class="form-label">フロー制御</label>
			<select
				id="flow-control"
				class="form-select"
				bind:value={config.flow_control}
				on:change={(e) => updateConfig('flow_control', e.currentTarget.value)}
				{disabled}
			>
				{#each FLOW_CONTROL as option}
					<option value={option.value}>{option.label}</option>
				{/each}
			</select>
		</div>

		<!-- タイムアウト -->
		<div class="form-group">
			<label for="timeout" class="form-label">タイムアウト (ミリ秒)</label>
			<input
				type="number"
				id="timeout"
				class="form-input"
				min="100"
				max="30000"
				step="100"
				bind:value={config.timeout_ms}
				on:input={(e) => updateConfig('timeout_ms', parseInt(e.currentTarget.value) || 1000)}
				{disabled}
			/>
		</div>
	</div>
</div>

<style>
	.serial-config {
		@apply space-y-6;
	}

	.config-section {
		@apply bg-white dark:bg-terminal-dark border border-gray-200 dark:border-terminal-gray 
		       rounded-lg p-6 shadow-sm;
	}

	.section-title {
		@apply text-lg font-semibold text-gray-900 dark:text-terminal-white mb-4 
		       border-b border-gray-200 dark:border-terminal-gray pb-2;
	}

	.form-group {
		@apply mb-4;
	}

	.form-label {
		@apply block text-sm font-medium text-gray-700 dark:text-terminal-light mb-2;
	}

	.form-select {
		@apply w-full px-3 py-2 border border-gray-300 dark:border-terminal-gray 
		       bg-white dark:bg-terminal-darker text-gray-900 dark:text-terminal-white
		       rounded-md shadow-sm transition-colors
		       focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500
		       disabled:bg-gray-50 dark:disabled:bg-terminal-darker/50 disabled:text-gray-500;
	}

	.form-select.error {
		@apply border-red-500 dark:border-red-400 focus:ring-red-500 focus:border-red-500;
	}

	.form-input {
		@apply w-full px-3 py-2 border border-gray-300 dark:border-terminal-gray 
		       bg-white dark:bg-terminal-darker text-gray-900 dark:text-terminal-white
		       rounded-md shadow-sm transition-colors
		       focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500
		       disabled:bg-gray-50 dark:disabled:bg-terminal-darker/50 disabled:text-gray-500;
	}

	.port-selection {
		@apply flex gap-2;
	}

	.port-selection .form-select {
		@apply flex-1;
	}

	.refresh-button {
		@apply flex items-center justify-center w-10 h-10 
		       border border-gray-300 dark:border-terminal-gray
		       bg-white dark:bg-terminal-darker hover:bg-gray-50 dark:hover:bg-terminal-gray/20
		       text-gray-600 dark:text-terminal-light
		       rounded-md transition-colors
		       focus:outline-none focus:ring-2 focus:ring-blue-500
		       disabled:opacity-50 disabled:cursor-not-allowed;
	}

	.refresh-icon {
		@apply w-5 h-5 fill-current transition-transform;
	}

	.refresh-button.loading .refresh-icon {
		@apply animate-spin;
	}

	.form-error {
		@apply mt-1 text-sm text-red-600 dark:text-red-400;
	}

	/* ダークモード対応 */
	@media (prefers-color-scheme: dark) {
		.config-section {
			@apply bg-terminal-dark border-terminal-gray;
		}
		
		.section-title {
			@apply text-terminal-white border-terminal-gray;
		}
		
		.form-label {
			@apply text-terminal-light;
		}
		
		.form-select,
		.form-input {
			@apply bg-terminal-darker border-terminal-gray text-terminal-white;
		}
		
		.refresh-button {
			@apply bg-terminal-darker border-terminal-gray text-terminal-light
			       hover:bg-terminal-gray/20;
		}
	}
</style>