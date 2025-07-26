# Tailwind CSS v4 セットアップガイド

## 1. 概要

このプロジェクトではTailwind CSS v4を使用しています。v4では設定方法が大幅に簡素化され、より直感的な設定が可能になりました。

## 2. 現在の設定状況

### 2.1 インストール済みパッケージ
```json
{
  "devDependencies": {
    "@tailwindcss/vite": "^4.1.11",
    "tailwindcss": "^4.1.11"
  }
}
```

### 2.2 Vite設定 (vite.config.ts)
```typescript
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [
    sveltekit(),
    tailwindcss() // Tailwind CSS v4 Viteプラグイン
  ]
});
```

### 2.3 CSS設定 (src/app.css)
```css
@import "tailwindcss";

/* カスタムスタイル */
:root {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}
```

## 3. Tailwind CSS v4の主な変更点

### 3.1 設定ファイルの簡素化
- `tailwind.config.js`は不要（オプション）
- `postcss.config.js`は不要
- 設定は`@config`ディレクティブまたはCSSファイル内で行う

### 3.2 @import構文の変更
```css
/* v3 */
@tailwind base;
@tailwind components;
@tailwind utilities;

/* v4 */
@import "tailwindcss";
```

### 3.3 カスタムテーマの定義方法

#### CSSファイル内での設定
```css
@import "tailwindcss";

@theme {
  --color-terminal-bg: #1a1b26;
  --color-terminal-text: #a9b1d6;
  --color-terminal-input: #24283b;
  --color-terminal-border: #414868;
  --color-terminal-success: #9ece6a;
  --color-terminal-error: #f7768e;
  --color-terminal-warning: #e0af68;
  --color-terminal-info: #7aa2f7;
  
  --color-connection-connected: #9ece6a;
  --color-connection-connecting: #e0af68;
  --color-connection-disconnected: #565f89;
  --color-connection-error: #f7768e;
}
```

#### 設定ファイルでの設定（オプション）
```javascript
// tailwind.config.js（必要に応じて）
export default {
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
      }
    }
  }
}
```

## 4. プロジェクト推奨設定

### 4.1 完全なapp.css
```css
@import "tailwindcss";

/* カスタムテーマ定義 */
@theme {
  /* ターミナルカラー */
  --color-terminal-bg: #1a1b26;
  --color-terminal-text: #a9b1d6;
  --color-terminal-input: #24283b;
  --color-terminal-border: #414868;
  --color-terminal-success: #9ece6a;
  --color-terminal-error: #f7768e;
  --color-terminal-warning: #e0af68;
  --color-terminal-info: #7aa2f7;
  
  /* 接続状態カラー */
  --color-connection-connected: #9ece6a;
  --color-connection-connecting: #e0af68;
  --color-connection-disconnected: #565f89;
  --color-connection-error: #f7768e;
  
  /* フォント設定 */
  --font-mono: 'Fira Code', 'SF Mono', 'Monaco', 'Inconsolata', 'Fira Code', 'Fira Mono', 'Roboto Mono', monospace;
}

/* ベーススタイル */
:root {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

/* カスタムコンポーネント */
@layer components {
  .terminal-container {
    @apply bg-terminal-bg border border-terminal-border rounded-lg overflow-hidden h-full flex flex-col;
  }
  
  .terminal-output {
    @apply flex-1 p-4 overflow-y-auto text-terminal-text font-mono text-sm leading-relaxed;
  }
  
  .terminal-input {
    @apply bg-terminal-input border-t border-terminal-border p-3 flex items-center gap-2;
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
}

/* ユーティリティ */
@layer utilities {
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
}
```

## 5. 使用例

### 5.1 基本的なクラス使用
```svelte
<!-- ターミナルコンポーネント -->
<div class="terminal-container">
  <div class="terminal-output scrollbar-thin">
    <!-- メッセージ表示 -->
  </div>
  <div class="terminal-input">
    <input 
      class="bg-transparent text-terminal-text font-mono flex-1 border-0 focus:ring-0" 
      placeholder="コマンドを入力..."
    />
  </div>
</div>

<!-- 接続状態表示 -->
<span class="connection-status connected">
  <span class="w-2 h-2 rounded-full bg-current mr-1"></span>
  接続中
</span>
```

### 5.2 ダークモード対応
```svelte
<!-- Tailwind CSS v4では自動的にダークモードに対応 -->
<div class="bg-white dark:bg-gray-900 text-gray-900 dark:text-white">
  <!-- コンテンツ -->
</div>
```

## 6. v3からv4への移行時の注意点

### 6.1 削除されたファイル
- ✅ `postcss.config.js` - 削除済み
- ✅ 旧`tailwind.config.js` - 削除済み

### 6.2 変更が必要な記述
```css
/* 変更前（v3） */
@tailwind base;
@tailwind components;
@tailwind utilities;

/* 変更後（v4）- 既に適用済み */
@import "tailwindcss";
```

### 6.3 カスタムカラーの参照方法
```css
/* v3 */
.text-terminal-text { color: var(--color-terminal-text); }

/* v4 */
.text-terminal-text { /* 自動的に適用される */ }
```

## 7. パフォーマンス最適化

### 7.1 v4の利点
- より高速なビルド
- 自動的な最適化
- ツリーシェイキングの改善
- 設定ファイルの簡素化

### 7.2 推奨設定
```typescript
// vite.config.ts
export default defineConfig({
  plugins: [
    sveltekit(),
    tailwindcss() // v4プラグインは自動最適化
  ]
});
```

## 8. トラブルシューティング

### 8.1 よくある問題

#### スタイルが適用されない
```bash
# キャッシュクリア
rm -rf .svelte-kit node_modules/.vite
yarn install
yarn dev
```

#### カスタムカラーが認識されない
```css
/* @themeディレクティブ内で定義されているか確認 */
@theme {
  --color-custom: #ffffff;
}
```

### 8.2 デバッグ方法
```bash
# Tailwind CSS v4のビルドログを確認
yarn build --verbose
```

## 9. 参考リンク

- [Tailwind CSS v4 公式ドキュメント](https://tailwindcss.com/docs/v4-beta)
- [Viteプラグイン設定](https://tailwindcss.com/docs/guides/vite)
- [テーマカスタマイゼーション](https://tailwindcss.com/docs/theme)