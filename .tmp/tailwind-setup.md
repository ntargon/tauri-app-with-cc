# TailwindCSS セットアップガイド

## 1. 必要なパッケージのインストール

```bash
yarn add -D tailwindcss postcss autoprefixer @tailwindcss/forms @tailwindcss/typography
```

## 2. 設定ファイル

### 2.1 tailwind.config.js
```javascript
/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        terminal: {
          bg: '#1a1b26',
          text: '#a9b1d6',
          input: '#24283b',
          border: '#414868',
          success: '#9ece6a',
          error: '#f7768e',
          warning: '#e0af68',
          info: '#7aa2f7'
        },
        connection: {
          connected: '#9ece6a',
          connecting: '#e0af68',
          disconnected: '#565f89',
          error: '#f7768e'
        }
      },
      fontFamily: {
        mono: ['Fira Code', 'Consolas', 'Monaco', 'monospace']
      }
    }
  },
  plugins: [
    require('@tailwindcss/forms'),
    require('@tailwindcss/typography')
  ]
}
```

### 2.2 postcss.config.js
```javascript
export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {}
  }
}
```

### 2.3 src/app.css
```css
@import 'tailwindcss/base';
@import 'tailwindcss/components';
@import 'tailwindcss/utilities';

@layer base {
  html {
    font-family: system-ui, sans-serif;
  }
  
  code {
    font-family: 'Fira Code', Consolas, Monaco, monospace;
  }
}

@layer components {
  /* ターミナル関連コンポーネント */
  .terminal-container {
    @apply bg-terminal-bg border border-terminal-border rounded-lg overflow-hidden h-full flex flex-col;
  }
  
  .terminal-output {
    @apply flex-1 p-4 overflow-y-auto text-terminal-text font-mono text-sm leading-relaxed;
  }
  
  .terminal-input {
    @apply bg-terminal-input border-t border-terminal-border p-3 flex items-center gap-2;
  }
  
  .terminal-input input {
    @apply bg-transparent text-terminal-text font-mono text-sm flex-1 border-0 focus:ring-0 focus:outline-none placeholder-terminal-text/50;
  }
  
  /* 接続パネル関連コンポーネント */
  .connection-panel {
    @apply bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-4;
  }
  
  .connection-status {
    @apply inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium;
  }
  
  .connection-status.connected {
    @apply bg-connection-connected/10 text-connection-connected;
  }
  
  .connection-status.connecting {
    @apply bg-connection-connecting/10 text-connection-connecting animate-pulse;
  }
  
  .connection-status.disconnected {
    @apply bg-connection-disconnected/10 text-connection-disconnected;
  }
  
  .connection-status.error {
    @apply bg-connection-error/10 text-connection-error;
  }
  
  /* フォーム関連コンポーネント */
  .form-input {
    @apply block w-full rounded-md border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm;
  }
  
  .form-select {
    @apply block w-full rounded-md border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm;
  }
  
  .form-label {
    @apply block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2;
  }
  
  /* ボタン関連コンポーネント */
  .btn {
    @apply inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-offset-2 transition-colors;
  }
  
  .btn-primary {
    @apply btn text-white bg-blue-600 hover:bg-blue-700 focus:ring-blue-500;
  }
  
  .btn-secondary {
    @apply btn text-gray-700 dark:text-gray-200 bg-white dark:bg-gray-700 border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-600 focus:ring-blue-500;
  }
  
  .btn-danger {
    @apply btn text-white bg-red-600 hover:bg-red-700 focus:ring-red-500;
  }
  
  .btn-success {
    @apply btn text-white bg-green-600 hover:bg-green-700 focus:ring-green-500;
  }
  
  /* カード関連コンポーネント */
  .card {
    @apply bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-sm;
  }
  
  .card-header {
    @apply px-4 py-3 border-b border-gray-200 dark:border-gray-700;
  }
  
  .card-body {
    @apply p-4;
  }
  
  /* ユーティリティクラス */
  .scrollbar-thin {
    scrollbar-width: thin;
    scrollbar-color: rgb(156 163 175) transparent;
  }
  
  .scrollbar-thin::-webkit-scrollbar {
    width: 6px;
  }
  
  .scrollbar-thin::-webkit-scrollbar-track {
    background: transparent;
  }
  
  .scrollbar-thin::-webkit-scrollbar-thumb {
    background-color: rgb(156 163 175);
    border-radius: 3px;
  }
  
  .scrollbar-thin::-webkit-scrollbar-thumb:hover {
    background-color: rgb(107 114 128);
  }
}

@layer utilities {
  .text-shadow {
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
  }
  
  .text-shadow-md {
    text-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  }
  
  .backdrop-blur-xs {
    backdrop-filter: blur(2px);
  }
}
```

## 3. ダークモード設定

### 3.1 ダークモード切り替えストア
```typescript
// src/lib/stores/theme.ts
import { writable } from 'svelte/store';
import { browser } from '$app/environment';

type Theme = 'light' | 'dark' | 'system';

function createThemeStore() {
  const { subscribe, set, update } = writable<Theme>('system');

  return {
    subscribe,
    set: (theme: Theme) => {
      if (browser) {
        localStorage.setItem('theme', theme);
        applyTheme(theme);
      }
      set(theme);
    },
    init: () => {
      if (browser) {
        const stored = localStorage.getItem('theme') as Theme;
        const theme = stored || 'system';
        applyTheme(theme);
        set(theme);
      }
    }
  };
}

function applyTheme(theme: Theme) {
  const root = document.documentElement;
  const isDark = theme === 'dark' || 
    (theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);
  
  if (isDark) {
    root.classList.add('dark');
  } else {
    root.classList.remove('dark');
  }
}

export const theme = createThemeStore();
```

### 3.2 テーマ切り替えコンポーネント
```svelte
<!-- src/lib/components/common/ThemeToggle.svelte -->
<script lang="ts">
  import { theme } from '$lib/stores/theme';
  
  function toggleTheme() {
    theme.update(current => {
      const themes: Array<'light' | 'dark' | 'system'> = ['light', 'dark', 'system'];
      const currentIndex = themes.indexOf(current);
      return themes[(currentIndex + 1) % themes.length];
    });
  }
</script>

<button 
  on:click={toggleTheme}
  class="p-2 rounded-md text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-100 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
  title="テーマ切り替え"
>
  {#if $theme === 'light'}
    <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
      <path fill-rule="evenodd" d="M10 2a1 1 0 011 1v1a1 1 0 11-2 0V3a1 1 0 011-1zm4 8a4 4 0 11-8 0 4 4 0 018 0zm-.464 4.95l.707.707a1 1 0 001.414-1.414l-.707-.707a1 1 0 00-1.414 1.414zm2.12-10.607a1 1 0 010 1.414l-.706.707a1 1 0 11-1.414-1.414l.707-.707a1 1 0 011.414 0zM17 11a1 1 0 100-2h-1a1 1 0 100 2h1zm-7 4a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1zM5.05 6.464A1 1 0 106.465 5.05l-.708-.707a1 1 0 00-1.414 1.414l.707.707zm1.414 8.486l-.707.707a1 1 0 01-1.414-1.414l.707-.707a1 1 0 011.414 1.414zM4 11a1 1 0 100-2H3a1 1 0 000 2h1z" clip-rule="evenodd" />
    </svg>
  {:else if $theme === 'dark'}
    <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
      <path d="M17.293 13.293A8 8 0 016.707 2.707a8.001 8.001 0 1010.586 10.586z" />
    </svg>
  {:else}
    <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
      <path fill-rule="evenodd" d="M3 5a2 2 0 012-2h10a2 2 0 012 2v8a2 2 0 01-2 2h-2.22l.123.489.804.804A1 1 0 0113 18H7a1 1 0 01-.707-1.707l.804-.804L7.22 15H5a2 2 0 01-2-2V5zm5.771 7H5V5h10v7H8.771z" clip-rule="evenodd" />
    </svg>
  {/if}
</button>
```

## 4. レスポンシブブレークポイント

```css
/* Tailwind デフォルトブレークポイント */
sm: 640px   /* タブレット縦向き */
md: 768px   /* タブレット横向き */
lg: 1024px  /* ラップトップ */
xl: 1280px  /* デスクトップ */
2xl: 1536px /* 大画面デスクトップ */
```

## 5. アニメーション設定

```javascript
// tailwind.config.js に追加
module.exports = {
  theme: {
    extend: {
      animation: {
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'bounce-subtle': 'bounce 2s infinite',
        'fade-in': 'fadeIn 0.5s ease-in-out',
        'slide-up': 'slideUp 0.3s ease-out',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0', transform: 'translateY(10px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        slideUp: {
          '0%': { transform: 'translateY(100%)' },
          '100%': { transform: 'translateY(0)' },
        }
      }
    }
  }
}
```

## 6. 共通コンポーネントの活用例

```svelte
<!-- フォーム要素の統一 -->
<input class="form-input" />
<select class="form-select">...</select>
<label class="form-label">...</label>

<!-- ボタンの統一 -->
<button class="btn-primary">接続</button>
<button class="btn-danger">切断</button>
<button class="btn-secondary">設定</button>

<!-- カードレイアウト -->
<div class="card">
  <div class="card-header">
    <h3>接続設定</h3>
  </div>
  <div class="card-body">
    <!-- コンテンツ -->
  </div>
</div>
```