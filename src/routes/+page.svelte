<script lang="ts">
	import { onMount } from 'svelte';
	import { app, notifications } from '$lib/stores';
	import ConnectionPanel from '$lib/components/ConnectionPanel/ConnectionPanel.svelte';
	import Terminal from '$lib/components/Terminal/Terminal.svelte';

	// アプリケーション状態
	let isInitialized = false;
	let initError: string | null = null;

	// レイアウト状態
	let sidebarCollapsed = false;
	let sidebarWidth = 350; // px

	// コンポーネントマウント時にアプリを初期化
	onMount(async () => {
		try {
			await app.initialize();
			isInitialized = true;
		} catch (error) {
			console.error('アプリケーション初期化エラー:', error);
			initError = error instanceof Error ? error.message : 'アプリケーションの初期化に失敗しました';
		}
	});

	// サイドバーの折りたたみ切り替え
	function toggleSidebar() {
		sidebarCollapsed = !sidebarCollapsed;
	}

	// サイドバー幅のリサイズハンドリング
	let isResizing = false;
	let startX = 0;
	let startWidth = 0;

	function handleMouseDown(event: MouseEvent) {
		isResizing = true;
		startX = event.clientX;
		startWidth = sidebarWidth;
		
		document.addEventListener('mousemove', handleMouseMove);
		document.addEventListener('mouseup', handleMouseUp);
		document.body.style.cursor = 'col-resize';
		document.body.style.userSelect = 'none';
	}

	function handleMouseMove(event: MouseEvent) {
		if (!isResizing) return;
		
		const deltaX = event.clientX - startX;
		const newWidth = Math.max(250, Math.min(600, startWidth + deltaX));
		sidebarWidth = newWidth;
	}

	function handleMouseUp() {
		isResizing = false;
		document.removeEventListener('mousemove', handleMouseMove);
		document.removeEventListener('mouseup', handleMouseUp);
		document.body.style.cursor = '';
		document.body.style.userSelect = '';
	}

	// 通知コンポーネント（簡易版）
	$: notificationList = $notifications;
</script>

<svelte:head>
	<title>組み込み開発ターミナル</title>
</svelte:head>

<div class="app">
	<!-- 初期化中の表示 -->
	{#if !isInitialized && !initError}
		<div class="init-screen">
			<div class="init-content">
				<div class="spinner"></div>
				<h2 class="init-title">組み込み開発ターミナル</h2>
				<p class="init-message">アプリケーションを初期化しています...</p>
			</div>
		</div>
	{:else if initError}
		<!-- 初期化エラーの表示 -->
		<div class="error-screen">
			<div class="error-content">
				<svg class="error-icon" viewBox="0 0 24 24" fill="currentColor">
					<path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
				</svg>
				<h2 class="error-title">初期化エラー</h2>
				<p class="error-message">{initError}</p>
				<button class="retry-button" on:click={() => window.location.reload()}>
					再試行
				</button>
			</div>
		</div>
	{:else}
		<!-- メインアプリケーション -->
		<div class="main-layout">
			<!-- サイドバー -->
			<div 
				class="sidebar" 
				class:collapsed={sidebarCollapsed}
				style="width: {sidebarCollapsed ? '60px' : `${sidebarWidth}px`}"
			>
				<!-- サイドバーヘッダー -->
				<div class="sidebar-header">
					{#if !sidebarCollapsed}
						<h1 class="app-title">組み込み開発ターミナル</h1>
					{/if}
					<button 
						class="sidebar-toggle" 
						on:click={toggleSidebar}
						title={sidebarCollapsed ? 'サイドバーを展開' : 'サイドバーを折りたたみ'}
					>
						<svg class="toggle-icon" viewBox="0 0 24 24" fill="currentColor">
							{#if sidebarCollapsed}
								<path d="M3 18h13v-2H3v2zm0-5h10v-2H3v2zm0-7v2h13V6H3z"/>
							{:else}
								<path d="M3 18h18v-2H3v2zm0-5h18v-2H3v2zm0-7v2h18V6H3z"/>
							{/if}
						</svg>
					</button>
				</div>

				<!-- 接続パネル -->
				{#if !sidebarCollapsed}
					<div class="sidebar-content">
						<ConnectionPanel />
					</div>
				{/if}

				<!-- リサイズハンドル -->
				{#if !sidebarCollapsed}
					<div 
						class="resize-handle"
						on:mousedown={handleMouseDown}
						role="separator"
						tabindex="0"
					></div>
				{/if}
			</div>

			<!-- メインコンテンツ -->
			<div class="main-content">
				<Terminal />
			</div>
		</div>
	{/if}

	<!-- 通知表示 -->
	{#if notificationList.length > 0}
		<div class="notifications">
			{#each notificationList as notification (notification.id)}
				<div 
					class="notification notification-{notification.type}"
					role="alert"
				>
					<div class="notification-content">
						<h4 class="notification-title">{notification.title}</h4>
						<p class="notification-message">{notification.message}</p>
					</div>
					<button 
						class="notification-close"
						on:click={() => notifications.remove(notification.id)}
						title="通知を閉じる"
					>
						<svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
							<path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
						</svg>
					</button>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	:global(html, body) {
		@apply h-full m-0 p-0 font-sans;
		background-color: theme('colors.gray.100');
	}

	:global(*) {
		box-sizing: border-box;
	}

	.app {
		@apply h-screen overflow-hidden bg-gray-100 dark:bg-terminal-darker;
	}

	.init-screen,
	.error-screen {
		@apply flex items-center justify-center h-full;
	}

	.init-content,
	.error-content {
		@apply text-center space-y-4;
	}

	.spinner {
		@apply w-12 h-12 mx-auto border-4 border-gray-200 dark:border-terminal-gray
		       border-t-blue-600 dark:border-t-blue-400 rounded-full animate-spin;
	}

	.init-title,
	.error-title {
		@apply text-2xl font-bold text-gray-900 dark:text-terminal-white;
	}

	.init-message,
	.error-message {
		@apply text-gray-600 dark:text-terminal-light;
	}

	.error-icon {
		@apply w-16 h-16 mx-auto text-red-500 dark:text-red-400;
	}

	.retry-button {
		@apply px-6 py-3 bg-blue-600 hover:bg-blue-700 text-white
		       rounded-lg font-medium transition-colors
		       focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2;
	}

	.main-layout {
		@apply flex h-full;
	}

	.sidebar {
		@apply flex-shrink-0 bg-white dark:bg-terminal-dark
		       border-r border-gray-200 dark:border-terminal-gray
		       transition-all duration-300 ease-in-out
		       relative;
	}

	.sidebar.collapsed {
		@apply overflow-hidden;
	}

	.sidebar-header {
		@apply flex items-center justify-between p-4
		       border-b border-gray-200 dark:border-terminal-gray
		       bg-gray-50 dark:bg-terminal-darker;
	}

	.app-title {
		@apply text-lg font-bold text-gray-900 dark:text-terminal-white truncate;
	}

	.sidebar-toggle {
		@apply p-2 text-gray-600 dark:text-terminal-light
		       hover:bg-gray-200 dark:hover:bg-terminal-gray/20
		       rounded-md transition-colors
		       focus:outline-none focus:ring-2 focus:ring-blue-500;
	}

	.toggle-icon {
		@apply w-5 h-5;
	}

	.sidebar-content {
		@apply flex-1 overflow-y-auto p-4;
	}

	.resize-handle {
		@apply absolute right-0 top-0 w-1 h-full cursor-col-resize
		       hover:bg-blue-500 dark:hover:bg-blue-400
		       transition-colors z-10;
	}

	.resize-handle:hover {
		@apply bg-blue-200 dark:bg-blue-800/50;
	}

	.main-content {
		@apply flex-1 flex flex-col overflow-hidden;
	}

	.notifications {
		@apply fixed top-4 right-4 space-y-2 z-50;
	}

	.notification {
		@apply flex items-start p-4 rounded-lg shadow-lg border-l-4
		       max-w-md backdrop-blur-sm;
	}

	.notification-info {
		@apply bg-blue-50/90 dark:bg-blue-900/90 border-l-blue-500 dark:border-l-blue-400;
	}

	.notification-success {
		@apply bg-green-50/90 dark:bg-green-900/90 border-l-green-500 dark:border-l-green-400;
	}

	.notification-warning {
		@apply bg-yellow-50/90 dark:bg-yellow-900/90 border-l-yellow-500 dark:border-l-yellow-400;
	}

	.notification-error {
		@apply bg-red-50/90 dark:bg-red-900/90 border-l-red-500 dark:border-l-red-400;
	}

	.notification-content {
		@apply flex-1 space-y-1;
	}

	.notification-title {
		@apply text-sm font-semibold text-gray-900 dark:text-terminal-white;
	}

	.notification-message {
		@apply text-sm text-gray-700 dark:text-terminal-light;
	}

	.notification-close {
		@apply ml-4 p-1 text-gray-400 dark:text-terminal-light/50
		       hover:text-gray-600 dark:hover:text-terminal-light
		       transition-colors rounded
		       focus:outline-none focus:ring-2 focus:ring-gray-500;
	}

	/* ダークモード対応 */
	@media (prefers-color-scheme: dark) {
		.app {
			@apply bg-terminal-darker;
		}
		
		.init-title,
		.error-title {
			@apply text-terminal-white;
		}
		
		.init-message,
		.error-message {
			@apply text-terminal-light;
		}
		
		.spinner {
			@apply border-terminal-gray border-t-blue-400;
		}
		
		.error-icon {
			@apply text-red-400;
		}
		
		.sidebar {
			@apply bg-terminal-dark border-terminal-gray;
		}
		
		.sidebar-header {
			@apply border-terminal-gray bg-terminal-darker;
		}
		
		.app-title {
			@apply text-terminal-white;
		}
		
		.sidebar-toggle {
			@apply text-terminal-light hover:bg-terminal-gray/20;
		}
		
		.resize-handle:hover {
			@apply bg-blue-800/50;
		}
		
		.notification-title {
			@apply text-terminal-white;
		}
		
		.notification-message {
			@apply text-terminal-light;
		}
		
		.notification-close {
			@apply text-terminal-light/50 hover:text-terminal-light;
		}
	}

	/* スクロールバーのスタイリング */
	.sidebar-content {
		scrollbar-width: thin;
		scrollbar-color: theme('colors.gray.400') theme('colors.gray.200');
	}

	.sidebar-content::-webkit-scrollbar {
		@apply w-2;
	}

	.sidebar-content::-webkit-scrollbar-track {
		@apply bg-gray-200 dark:bg-terminal-gray;
	}

	.sidebar-content::-webkit-scrollbar-thumb {
		@apply bg-gray-400 dark:bg-terminal-light/30 rounded;
	}

	.sidebar-content::-webkit-scrollbar-thumb:hover {
		@apply bg-gray-500 dark:bg-terminal-light/50;
	}
</style>