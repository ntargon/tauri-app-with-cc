<script lang="ts">
	import type { TcpConfig } from '$lib/types';

	// プロパティ
	export let config: TcpConfig;
	export let onConfigChange: (config: TcpConfig) => void;
	export let disabled = false;

	// よく使用されるポート番号
	const COMMON_PORTS = [
		{ value: 23, label: '23 (Telnet)' },
		{ value: 80, label: '80 (HTTP)' },
		{ value: 443, label: '443 (HTTPS)' },
		{ value: 22, label: '22 (SSH)' },
		{ value: 21, label: '21 (FTP)' },
		{ value: 25, label: '25 (SMTP)' },
		{ value: 110, label: '110 (POP3)' },
		{ value: 143, label: '143 (IMAP)' },
		{ value: 993, label: '993 (IMAPS)' },
		{ value: 995, label: '995 (POP3S)' },
		{ value: 8080, label: '8080 (HTTP Alt)' },
		{ value: 9600, label: '9600 (Custom)' }
	];

	// 設定変更ハンドラー
	function updateConfig(key: keyof TcpConfig, value: any) {
		const newConfig = { ...config, [key]: value };
		onConfigChange(newConfig);
	}

	// IPアドレスのバリデーション
	function validateIpAddress(ip: string): boolean {
		if (!ip) return false;
		
		// IPv4の基本的なバリデーション
		const ipv4Regex = /^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/;
		
		// ホスト名の基本的なバリデーション
		const hostnameRegex = /^[a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?)*$/;
		
		return ipv4Regex.test(ip) || hostnameRegex.test(ip);
	}

	// ポート番号のバリデーション
	function validatePort(port: number): boolean {
		return port > 0 && port <= 65535;
	}

	// よく使用されるポートから選択
	function selectCommonPort(port: number) {
		updateConfig('port', port);
	}

	// リアクティブな検証結果
	$: isIpValid = validateIpAddress(config.host);
	$: isPortValid = validatePort(config.port);
</script>

<div class="tcp-config">
	<div class="config-section">
		<h3 class="section-title">TCP接続設定</h3>
		
		<!-- ホスト/IPアドレス -->
		<div class="form-group">
			<label for="host" class="form-label">ホスト/IPアドレス</label>
			<input
				type="text"
				id="host"
				class="form-input"
				class:error={!isIpValid && config.host}
				placeholder="192.168.1.1 または hostname"
				bind:value={config.host}
				on:input={(e) => updateConfig('host', e.currentTarget.value)}
				{disabled}
			/>
			{#if !isIpValid && config.host}
				<div class="form-error">有効なIPアドレスまたはホスト名を入力してください</div>
			{/if}
		</div>

		<!-- ポート番号 -->
		<div class="form-group">
			<label for="port" class="form-label">ポート番号</label>
			<div class="port-input-group">
				<input
					type="number"
					id="port"
					class="form-input"
					class:error={!isPortValid}
					min="1"
					max="65535"
					bind:value={config.port}
					on:input={(e) => updateConfig('port', parseInt(e.currentTarget.value) || 1)}
					{disabled}
				/>
			</div>
			{#if !isPortValid}
				<div class="form-error">ポート番号は1-65535の範囲で入力してください</div>
			{/if}
			
			<!-- よく使用されるポート -->
			<div class="common-ports">
				<span class="common-ports-label">よく使用されるポート:</span>
				<div class="common-ports-grid">
					{#each COMMON_PORTS as portOption}
						<button
							type="button"
							class="port-button"
							class:active={config.port === portOption.value}
							on:click={() => selectCommonPort(portOption.value)}
							{disabled}
						>
							{portOption.label}
						</button>
					{/each}
				</div>
			</div>
		</div>

		<!-- 接続タイムアウト -->
		<div class="form-group">
			<label for="connect-timeout" class="form-label">接続タイムアウト (秒)</label>
			<input
				type="number"
				id="connect-timeout"
				class="form-input"
				min="1"
				max="300"
				step="1"
				bind:value={config.connect_timeout_sec}
				on:input={(e) => updateConfig('connect_timeout_sec', parseInt(e.currentTarget.value) || 10)}
				{disabled}
			/>
		</div>

		<!-- 読み取りタイムアウト -->
		<div class="form-group">
			<label for="read-timeout" class="form-label">読み取りタイムアウト (秒)</label>
			<input
				type="number"
				id="read-timeout"
				class="form-input"
				min="1"
				max="300"
				step="1"
				bind:value={config.read_timeout_sec}
				on:input={(e) => updateConfig('read_timeout_sec', parseInt(e.currentTarget.value) || 30)}
				{disabled}
			/>
		</div>

		<!-- Keep-alive設定 -->
		<div class="form-group">
			<div class="checkbox-group">
				<input
					type="checkbox"
					id="keep-alive"
					class="form-checkbox"
					bind:checked={config.keep_alive}
					on:change={(e) => updateConfig('keep_alive', e.currentTarget.checked)}
					{disabled}
				/>
				<label for="keep-alive" class="checkbox-label">Keep-Alive有効</label>
			</div>
			<div class="form-help">接続を維持するためのキープアライブ機能を有効にします</div>
		</div>

		{#if config.keep_alive}
			<!-- Keep-alive間隔 -->
			<div class="form-group keep-alive-settings">
				<label for="keep-alive-interval" class="form-label">Keep-Alive間隔 (秒)</label>
				<input
					type="number"
					id="keep-alive-interval"
					class="form-input"
					min="10"
					max="3600"
					step="10"
					bind:value={config.keep_alive_interval_sec}
					on:input={(e) => updateConfig('keep_alive_interval_sec', parseInt(e.currentTarget.value) || 60)}
					{disabled}
				/>
			</div>
		{/if}

		<!-- 自動再接続設定 -->
		<div class="form-group">
			<div class="checkbox-group">
				<input
					type="checkbox"
					id="auto-reconnect"
					class="form-checkbox"
					bind:checked={config.auto_reconnect}
					on:change={(e) => updateConfig('auto_reconnect', e.currentTarget.checked)}
					{disabled}
				/>
				<label for="auto-reconnect" class="checkbox-label">自動再接続</label>
			</div>
			<div class="form-help">接続が切断された場合に自動的に再接続を試行します</div>
		</div>

		{#if config.auto_reconnect}
			<!-- 再接続間隔 -->
			<div class="form-group reconnect-settings">
				<label for="reconnect-interval" class="form-label">再接続間隔 (秒)</label>
				<input
					type="number"
					id="reconnect-interval"
					class="form-input"
					min="1"
					max="300"
					step="1"
					bind:value={config.reconnect_interval_sec}
					on:input={(e) => updateConfig('reconnect_interval_sec', parseInt(e.currentTarget.value) || 5)}
					{disabled}
				/>
			</div>

			<!-- 最大再接続試行回数 -->
			<div class="form-group">
				<label for="max-reconnect" class="form-label">最大再接続試行回数</label>
				<input
					type="number"
					id="max-reconnect"
					class="form-input"
					min="0"
					max="100"
					step="1"
					bind:value={config.max_reconnect_attempts}
					on:input={(e) => updateConfig('max_reconnect_attempts', parseInt(e.currentTarget.value) || 3)}
					{disabled}
				/>
				<div class="form-help">0を指定すると無制限に再接続を試行します</div>
			</div>
		{/if}
	</div>
</div>

<style>
	.tcp-config {
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

	.form-group.keep-alive-settings,
	.form-group.reconnect-settings {
		margin-left: 1.5rem;
		border-left: 2px solid #bfdbfe;
		padding-left: 1rem;
	}

	.form-label {
		display: block;
		font-size: 0.875rem;
		font-weight: 500;
		color: #374151;
		margin-bottom: 0.5rem;
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

	.form-input.error {
		border-color: #ef4444;
	}

	.form-input.error:focus {
		border-color: #ef4444;
		box-shadow: 0 0 0 2px rgba(239, 68, 68, 0.1);
	}

	.port-input-group {
		display: flex;
		gap: 0.5rem;
	}

	.common-ports {
		margin-top: 0.75rem;
	}

	.common-ports-label {
		display: block;
		font-size: 0.875rem;
		color: #6b7280;
		margin-bottom: 0.5rem;
	}

	.common-ports-grid {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 0.5rem;
	}

	@media (min-width: 768px) {
		.common-ports-grid {
			grid-template-columns: repeat(3, 1fr);
		}
	}

	@media (min-width: 1024px) {
		.common-ports-grid {
			grid-template-columns: repeat(4, 1fr);
		}
	}

	.port-button {
		padding: 0.25rem 0.75rem;
		font-size: 0.75rem;
		border: 1px solid #d1d5db;
		background-color: white;
		color: #374151;
		border-radius: 0.25rem;
		transition: all 0.2s;
		cursor: pointer;
	}

	.port-button:hover:not(:disabled) {
		background-color: #f9fafb;
	}

	.port-button:focus {
		outline: none;
		box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
	}

	.port-button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.port-button.active {
		background-color: #dbeafe;
		border-color: #93c5fd;
		color: #1d4ed8;
	}

	.checkbox-group {
		display: flex;
		align-items: center;
	}

	.form-checkbox {
		width: 1rem;
		height: 1rem;
		color: #2563eb;
		border: 1px solid #d1d5db;
		background-color: white;
		border-radius: 0.25rem;
	}

	.form-checkbox:focus {
		box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
	}

	.form-checkbox:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.checkbox-label {
		margin-left: 0.75rem;
		font-size: 0.875rem;
		font-weight: 500;
		color: #374151;
		cursor: pointer;
	}

	.form-help {
		margin-top: 0.25rem;
		font-size: 0.75rem;
		color: #6b7280;
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

		.form-group.keep-alive-settings,
		.form-group.reconnect-settings {
			border-left-color: #1e40af;
		}
		
		.form-label {
			color: #d1d5db;
		}
		
		.form-input {
			background-color: #1f2937;
			border-color: #4b5563;
			color: #f9fafb;
		}

		.form-input:disabled {
			background-color: rgba(31, 41, 55, 0.5);
			color: #9ca3af;
		}

		.form-input.error {
			border-color: #f87171;
		}

		.common-ports-label {
			color: #d1d5db;
		}
		
		.port-button {
			background-color: #1f2937;
			border-color: #4b5563;
			color: #d1d5db;
		}

		.port-button:hover:not(:disabled) {
			background-color: rgba(75, 85, 99, 0.2);
		}

		.port-button.active {
			background-color: rgba(37, 99, 235, 0.3);
			border-color: #3b82f6;
			color: #93c5fd;
		}
		
		.form-checkbox {
			border-color: #4b5563;
			background-color: #1f2937;
		}
		
		.checkbox-label {
			color: #d1d5db;
		}
		
		.form-help {
			color: rgba(209, 213, 219, 0.7);
		}

		.form-error {
			color: #f87171;
		}
	}
</style>