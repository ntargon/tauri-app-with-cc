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

	.form-group.keep-alive-settings,
	.form-group.reconnect-settings {
		@apply ml-6 border-l-2 border-blue-200 dark:border-blue-800 pl-4;
	}

	.form-label {
		@apply block text-sm font-medium text-gray-700 dark:text-terminal-light mb-2;
	}

	.form-input {
		@apply w-full px-3 py-2 border border-gray-300 dark:border-terminal-gray 
		       bg-white dark:bg-terminal-darker text-gray-900 dark:text-terminal-white
		       rounded-md shadow-sm transition-colors
		       focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500
		       disabled:bg-gray-50 dark:disabled:bg-terminal-darker/50 disabled:text-gray-500;
	}

	.form-input.error {
		@apply border-red-500 dark:border-red-400 focus:ring-red-500 focus:border-red-500;
	}

	.port-input-group {
		@apply flex gap-2;
	}

	.common-ports {
		@apply mt-3;
	}

	.common-ports-label {
		@apply block text-sm text-gray-600 dark:text-terminal-light mb-2;
	}

	.common-ports-grid {
		@apply grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-2;
	}

	.port-button {
		@apply px-3 py-1 text-xs border border-gray-300 dark:border-terminal-gray
		       bg-white dark:bg-terminal-darker hover:bg-gray-50 dark:hover:bg-terminal-gray/20
		       text-gray-700 dark:text-terminal-light
		       rounded transition-colors
		       focus:outline-none focus:ring-2 focus:ring-blue-500
		       disabled:opacity-50 disabled:cursor-not-allowed;
	}

	.port-button.active {
		@apply bg-blue-100 dark:bg-blue-900/30 border-blue-300 dark:border-blue-700
		       text-blue-700 dark:text-blue-300;
	}

	.checkbox-group {
		@apply flex items-center;
	}

	.form-checkbox {
		@apply w-4 h-4 text-blue-600 border-gray-300 dark:border-terminal-gray
		       bg-white dark:bg-terminal-darker
		       rounded focus:ring-blue-500
		       disabled:opacity-50 disabled:cursor-not-allowed;
	}

	.checkbox-label {
		@apply ml-3 text-sm font-medium text-gray-700 dark:text-terminal-light
		       cursor-pointer;
	}

	.form-help {
		@apply mt-1 text-xs text-gray-500 dark:text-terminal-light/70;
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
		
		.form-input {
			@apply bg-terminal-darker border-terminal-gray text-terminal-white;
		}
		
		.port-button {
			@apply bg-terminal-darker border-terminal-gray text-terminal-light
			       hover:bg-terminal-gray/20;
		}
		
		.form-checkbox {
			@apply border-terminal-gray bg-terminal-darker;
		}
		
		.checkbox-label {
			@apply text-terminal-light;
		}
		
		.form-help {
			@apply text-terminal-light/70;
		}
	}
</style>