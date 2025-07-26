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
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}

	.config-section {
		background-color: white;
		border: 1px solid #e5e7eb;
		border-radius: 0.5rem;
		padding: 1.5rem;
		box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
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
		margin-bottom: 1rem;
	}

	.form-label {
		display: block;
		font-size: 0.875rem;
		font-weight: 500;
		color: #374151;
		margin-bottom: 0.5rem;
	}

	.form-select {
		width: 100%;
		padding: 0.5rem 0.75rem;
		border: 1px solid #d1d5db;
		background-color: white;
		color: #111827;
		border-radius: 0.375rem;
		box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
		transition: all 0.2s;
	}

	.form-select:focus {
		outline: none;
		border-color: #3b82f6;
		box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
	}

	.form-select:disabled {
		background-color: #f9fafb;
		color: #9ca3af;
		cursor: not-allowed;
	}

	.form-select.error {
		border-color: #ef4444;
	}

	.form-select.error:focus {
		border-color: #ef4444;
		box-shadow: 0 0 0 2px rgba(239, 68, 68, 0.1);
	}

	.form-input {
		width: 100%;
		padding: 0.5rem 0.75rem;
		border: 1px solid #d1d5db;
		background-color: white;
		color: #111827;
		border-radius: 0.375rem;
		box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
		transition: all 0.2s;
	}

	.form-input:focus {
		outline: none;
		border-color: #3b82f6;
		box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
	}

	.form-input:disabled {
		background-color: #f9fafb;
		color: #9ca3af;
		cursor: not-allowed;
	}

	.port-selection {
		display: flex;
		gap: 0.5rem;
	}

	.port-selection .form-select {
		flex: 1;
	}

	.refresh-button {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 2.5rem;
		height: 2.5rem;
		border: 1px solid #d1d5db;
		background-color: white;
		color: #6b7280;
		border-radius: 0.375rem;
		transition: all 0.2s;
		cursor: pointer;
	}

	.refresh-button:hover:not(:disabled) {
		background-color: #f9fafb;
	}

	.refresh-button:focus {
		outline: none;
		box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
	}

	.refresh-button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.refresh-icon {
		width: 1.25rem;
		height: 1.25rem;
		fill: currentColor;
		transition: transform 0.2s;
	}

	.refresh-button.loading .refresh-icon {
		animation: spin 1s linear infinite;
	}

	.form-error {
		margin-top: 0.25rem;
		font-size: 0.875rem;
		color: #dc2626;
	}

	/* ダークモード対応 */
	@media (prefers-color-scheme: dark) {
		.config-section {
			background-color: #374151;
			border-color: #4b5563;
		}
		
		.section-title {
			color: #f9fafb;
			border-color: #4b5563;
		}
		
		.form-label {
			color: #d1d5db;
		}
		
		.form-select,
		.form-input {
			background-color: #1f2937;
			border-color: #4b5563;
			color: #f9fafb;
		}

		.form-select:disabled,
		.form-input:disabled {
			background-color: rgba(31, 41, 55, 0.5);
			color: #9ca3af;
		}

		.form-select.error {
			border-color: #f87171;
		}

		.form-error {
			color: #f87171;
		}
		
		.refresh-button {
			background-color: #1f2937;
			border-color: #4b5563;
			color: #d1d5db;
		}

		.refresh-button:hover:not(:disabled) {
			background-color: rgba(75, 85, 99, 0.2);
		}
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