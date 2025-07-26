# 技術設計書 - 組み込み機器開発ツール（ターミナルアプリ）

## 1. アーキテクチャ概要

### 1.1 システム構成
```
┌─────────────────────────────────────────┐
│                Frontend                 │
│            (SvelteKit + TS)             │
├─────────────────────────────────────────┤
│              Tauri Bridge               │
├─────────────────────────────────────────┤
│                Backend                  │
│               (Rust)                    │
├─────────────────────────────────────────┤
│          Communication Layer            │
│        (Serial / TCP Handler)          │
└─────────────────────────────────────────┘
```

### 1.2 レイヤー責任
- **Frontend**: UI/UX、ユーザー入力処理、状態表示
- **Tauri Bridge**: フロントエンド-バックエンド間の通信
- **Backend**: ビジネスロジック、設定管理、ファイルI/O
- **Communication Layer**: シリアル/TCP通信の実装

## 2. ディレクトリ構造 (実装済み)

```
src/
├── lib/
│   ├── components/           # UIコンポーネント (実装済み)
│   │   ├── layout/          # レイアウトコンポーネント
│   │   │   ├── AppShell.svelte     # メインアプリケーションシェル
│   │   │   └── Sidebar.svelte      # サイドバーレイアウト
│   │   ├── terminal/        # ターミナル関連コンポーネント
│   │   │   ├── Terminal.svelte     # メインターミナルコンポーネント
│   │   │   ├── InputArea.svelte    # 入力エリア
│   │   │   ├── MessageList.svelte  # メッセージリスト
│   │   │   └── MessageItem.svelte  # 個別メッセージアイテム
│   │   ├── connection/      # 接続関連コンポーネント
│   │   │   ├── ConnectionPanel.svelte # 接続パネル統合
│   │   │   ├── ConnectionForm.svelte  # 接続設定フォーム
│   │   │   └── ConnectionStatus.svelte # 接続状態表示
│   │   └── settings/ (予定) # 設定管理コンポーネント
│   │       ├── SettingsPanel.svelte
│   │       ├── ProfileManager.svelte
│   │       └── AppSettings.svelte
│   ├── stores/               # 状態管理 (実装済み)
│   │   ├── connection.ts     # 接続状態管理
│   │   ├── terminal.ts       # ターミナル状態管理
│   │   ├── settings.ts       # 設定管理
│   │   └── logs.ts (予定)    # ログ管理
│   ├── types/                # 型定義 (実装済み)
│   │   ├── connection.ts     # 接続関連型定義
│   │   ├── terminal.ts       # ターミナル関連型定義
│   │   └── settings.ts       # 設定関連型定義
│   └── utils/ (予定)         # ユーティリティ
│       ├── formatters.ts
│       └── validators.ts
├── routes/
│   ├── +layout.svelte       # 基本レイアウト
│   └── +page.svelte         # メインページ (AppShell統合)
├── app.html                 # HTMLテンプレート
└── app.css                  # Tailwind CSS v4設定

src-tauri/
├── src/
│   ├── commands/             # Tauriコマンド (実装済み)
│   │   ├── mod.rs            # コマンドモジュール
│   │   ├── connection.rs     # 接続関連コマンド
│   │   ├── terminal.rs       # ターミナル関連コマンド
│   │   └── settings.rs       # 設定関連コマンド
│   ├── communication/        # 通信層 (実装済み)
│   │   ├── mod.rs            # 通信抽象化レイヤー
│   │   ├── serial.rs         # シリアル通信実装
│   │   └── tcp.rs            # TCP通信実装
│   ├── models/               # データ構造 (実装済み)
│   │   ├── mod.rs            # モデルモジュール
│   │   ├── connection.rs     # 接続設定データ構造
│   │   ├── settings.rs       # アプリ設定データ構造
│   │   └── terminal.rs       # ターミナルデータ構造
│   ├── services/             # ビジネスロジック (一部実装済み)
│   │   ├── mod.rs            # サービスモジュール
│   │   ├── connection_manager.rs (予定) # 接続管理サービス
│   │   ├── settings_manager.rs (予定)   # 設定管理サービス
│   │   └── log_manager.rs (予定)        # ログ管理サービス
│   ├── utils/                # ユーティリティ (予定)
│   │   ├── mod.rs            # ユーティリティモジュール
│   │   ├── encryption.rs (予定) # 暗号化ユーティリティ
│   │   └── file_utils.rs (予定) # ファイル操作ユーティリティ
│   ├── lib.rs                # ライブラリルート
│   └── main.rs               # アプリケーションエントリーポイント
├── Cargo.toml                # Rust依存関係設定
└── tauri.conf.json           # Tauri設定
```

## 3. データモデル設計

### 3.1 接続設定
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectionConfig {
    pub id: String,
    pub name: String,
    pub connection_type: ConnectionType,
    pub serial_config: Option<SerialConfig>,
    pub tcp_config: Option<TcpConfig>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ConnectionType {
    Serial,
    Tcp,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SerialConfig {
    pub port: String,
    pub baud_rate: u32,
    pub data_bits: DataBits,
    pub stop_bits: StopBits,
    pub parity: Parity,
    pub flow_control: FlowControl,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TcpConfig {
    pub host: String,
    pub port: u16,
    pub timeout: Duration,
    pub keep_alive: bool,
}
```

### 3.2 ターミナル設定
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TerminalConfig {
    pub encoding: String,
    pub line_ending: LineEnding,
    pub echo_input: bool,
    pub show_timestamp: bool,
    pub font_family: String,
    pub font_size: u32,
    pub theme: TerminalTheme,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum LineEnding {
    Cr,
    Lf,
    CrLf,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TerminalTheme {
    pub background_color: String,
    pub text_color: String,
    pub input_color: String,
    pub timestamp_color: String,
}
```

### 3.3 通信メッセージ
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TerminalMessage {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub direction: MessageDirection,
    pub content: String,
    pub encoding: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MessageDirection {
    Sent,
    Received,
}
```

## 4. 状態管理設計

### 4.1 Svelte Stores構成
```typescript
// stores/connection.ts
export interface ConnectionState {
  currentConnection: ConnectionConfig | null;
  connectionStatus: 'disconnected' | 'connecting' | 'connected' | 'error';
  availablePorts: string[];
  profiles: ConnectionConfig[];
}

// stores/terminal.ts
export interface TerminalState {
  messages: TerminalMessage[];
  commandHistory: string[];
  currentInput: string;
  config: TerminalConfig;
  isLogging: boolean;
}

// stores/settings.ts
export interface SettingsState {
  appConfig: AppConfig;
  isDirty: boolean;
  lastSaved: Date | null;
}
```

### 4.2 状態フロー
```
User Input → Store Update → Tauri Command → Rust Backend → Response → Store Update → UI Update
```

## 5. 通信層設計

### 5.1 通信抽象化
```rust
#[async_trait]
pub trait ConnectionHandler: Send + Sync {
    async fn connect(&mut self, config: &ConnectionConfig) -> Result<(), ConnectionError>;
    async fn disconnect(&mut self) -> Result<(), ConnectionError>;
    async fn send(&mut self, data: &[u8]) -> Result<(), ConnectionError>;
    async fn receive(&mut self) -> Result<Vec<u8>, ConnectionError>;
    fn is_connected(&self) -> bool;
}

pub struct SerialHandler {
    port: Option<Box<dyn SerialPort>>,
    config: SerialConfig,
}

pub struct TcpHandler {
    stream: Option<TcpStream>,
    config: TcpConfig,
}
```

### 5.2 接続管理
```rust
pub struct ConnectionManager {
    current_handler: Option<Box<dyn ConnectionHandler>>,
    message_sender: UnboundedSender<TerminalMessage>,
    is_running: Arc<AtomicBool>,
}

impl ConnectionManager {
    pub async fn connect(&mut self, config: ConnectionConfig) -> Result<(), ConnectionError>;
    pub async fn disconnect(&mut self) -> Result<(), ConnectionError>;
    pub async fn send_message(&mut self, message: String) -> Result<(), ConnectionError>;
    pub fn start_receive_loop(&self);
}
```

## 6. UI/UXコンポーネント設計

### 6.1 Tailwind CSS v4デザインシステム (実装済み)

#### 6.1.1 カラーパレット
```css
/* app.css - Tailwind CSS v4設定 */
@import "tailwindcss";

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
}
```

#### 6.1.2 コンポーネントクラス設計
```css
/* app.css */
@tailwind base;
@tailwind components;
@tailwind utilities;

@layer components {
  .terminal-container {
    @apply bg-terminal-bg border border-terminal-border rounded-lg overflow-hidden h-full flex flex-col;
  }
  
  .terminal-output {
    @apply flex-1 p-4 overflow-y-auto text-terminal-text font-mono text-sm;
  }
  
  .terminal-input {
    @apply bg-terminal-input border-t border-terminal-border p-3 flex items-center gap-2;
  }
  
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
    @apply bg-connection-connecting/10 text-connection-connecting;
  }
  
  .connection-status.disconnected {
    @apply bg-connection-disconnected/10 text-connection-disconnected;
  }
  
  .connection-status.error {
    @apply bg-connection-error/10 text-connection-error;
  }
}
```

### 6.2 主要コンポーネント設計

#### 6.2.1 ターミナルコンポーネント (実装済み)
```svelte
<!-- Terminal.svelte -->
<script lang="ts">
  import MessageList from './MessageList.svelte';
  import InputArea from './InputArea.svelte';
  import { terminal } from '$lib/stores/terminal';
  import { connection } from '$lib/stores/connection';
</script>

<div class="flex flex-col h-full bg-gray-900 rounded-lg overflow-hidden">
  <!-- メッセージリスト表示エリア -->
  <MessageList />
  
  <!-- 入力エリア -->
  <InputArea />
</div>
```

#### 6.2.2 AppShellコンポーネント (実装済み)
```svelte
<!-- AppShell.svelte -->
<script lang="ts">
  import Sidebar from './Sidebar.svelte';
  import Terminal from '../terminal/Terminal.svelte';
</script>

<div class="flex h-screen bg-gray-100 dark:bg-gray-900">
  <!-- サイドバー -->
  <Sidebar />
  
  <!-- メインコンテンツエリア -->
  <main class="flex-1 p-4">
    <Terminal />
  </main>
</div>
```

#### 6.2.3 接続パネル (実装済み)
```svelte
<!-- ConnectionPanel.svelte -->
<script lang="ts">
  import ConnectionForm from './ConnectionForm.svelte';
  import ConnectionStatus from './ConnectionStatus.svelte';
</script>

<div class="p-4 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
  <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
    接続設定
  </h2>
  
  <!-- 接続状態表示 -->
  <ConnectionStatus />
  
  <!-- 接続設定フォーム -->
  <ConnectionForm />
</div>
```

#### 6.2.4 Sidebarコンポーネント (実装済み)
```svelte
<!-- Sidebar.svelte -->
<script lang="ts">
  import ConnectionPanel from '../connection/ConnectionPanel.svelte';
</script>

<aside class="w-80 bg-gray-50 dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 p-4">
  <div class="mb-6">
    <h1 class="text-xl font-bold text-gray-900 dark:text-white">
      組み込み開発ターミナル
    </h1>
  </div>
  
  <ConnectionPanel />
</aside>
```

## 7. 実装状況サマリー (2024年12月現在)

### 7.1 完了した機能
- ✅ **基盤アーキテクチャ**: Tauri v2 + SvelteKit + TypeScript + Tailwind CSS v4
- ✅ **データモデル**: Rust側の完全な型定義とTS側の型定義
- ✅ **通信機能**: シリアル・TCP通信の完全実装
- ✅ **UIコンポーネント**: 11のコンポーネント (layout, terminal, connection)
- ✅ **状態管理**: Svelte stores による反応的状態管理
- ✅ **レスポンシブデザイン**: Tailwind CSS v4による現代的UI

### 7.2 技術スタック詳細
```typescript
// Frontend
- SvelteKit: ^2.9.0
- Svelte: ^5.0.0  
- TypeScript: ~5.6.2
- Tailwind CSS: ^4.1.11
- Vite: ^6.0.3

// Backend  
- Tauri: ^2
- Rust 2021 Edition
- tokio: 1.0 (async runtime)
- serialport: 4.2
- serde: 1.0 (JSON serialization)
```

### 7.3 アーキテクチャの特徴
- **型安全性**: RustとTypeScriptでの完全な型定義
- **パフォーマンス**: Rustバックエンドによる高速処理
- **モダンUI**: Tailwind CSS v4による最新のデザインシステム
- **リアクティブ**: Svelte 5による効率的な状態管理

## 8. 次期開発計画

### 8.1 Phase 4: 設定管理機能 (優先度: 高)
- **ターゲット**: 2024年12月下旬〜2025年1月
- **機能**: 接続プロファイル保存、アプリ設定永続化
- **技術**: Tauri ファイルシステムAPI、JSON設定ファイル

### 8.2 Phase 5: ログ・履歴機能 (優先度: 中)
- **機能**: 通信ログ保存、コマンド履歴、エクスポート機能
- **技術**: ファイルベースログ、日付別管理

### 8.3 Phase 6: エラーハンドリング強化 (優先度: 中)
- **機能**: 包括的エラー処理、自動再接続、ユーザー通知
- **技術**: カスタムエラー型、トースト通知

## 9. データ永続化設計 (予定)

### 7.1 設定ファイル構造
```
%APPDATA%/tauri-terminal-tool/
├── config/
│   ├── app.json           # アプリケーション設定
│   └── profiles.json      # 接続プロファイル
├── logs/
│   ├── YYYY-MM-DD/        # 日別ログディレクトリ
│   │   ├── session_001.log
│   │   └── session_002.log
└── cache/
    └── recent_ports.json  # 最近使用したポート情報
```

### 7.2 設定管理
```rust
pub struct SettingsManager {
    config_dir: PathBuf,
    app_config: AppConfig,
    profiles: Vec<ConnectionConfig>,
}

impl SettingsManager {
    pub fn load_app_config(&mut self) -> Result<AppConfig, SettingsError>;
    pub fn save_app_config(&self, config: &AppConfig) -> Result<(), SettingsError>;
    pub fn load_profiles(&mut self) -> Result<Vec<ConnectionConfig>, SettingsError>;
    pub fn save_profile(&mut self, profile: &ConnectionConfig) -> Result<(), SettingsError>;
    pub fn delete_profile(&mut self, id: &str) -> Result<(), SettingsError>;
}
```

## 8. エラーハンドリング設計

### 8.1 エラータイプ定義
```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum AppError {
    Connection(ConnectionError),
    Settings(SettingsError),
    IO(String),
    Validation(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ConnectionError {
    PortNotFound(String),
    PermissionDenied,
    InvalidConfiguration,
    NetworkTimeout,
    SendFailed(String),
    ReceiveFailed(String),
}
```

### 8.2 エラー伝播
```rust
pub type AppResult<T> = Result<T, AppError>;

#[tauri::command]
pub async fn connect_to_device(config: ConnectionConfig) -> AppResult<()> {
    // 実装
}
```

## 9. パフォーマンス設計

### 9.1 非同期処理
- Tokioランタイムでの非同期通信処理
- チャネルベースのメッセージパッシング
- UI更新の最適化

### 9.2 メモリ管理
```rust
// 循環バッファでメッセージ履歴を管理
pub struct MessageBuffer {
    messages: VecDeque<TerminalMessage>,
    max_size: usize,
}

impl MessageBuffer {
    pub fn push(&mut self, message: TerminalMessage) {
        if self.messages.len() >= self.max_size {
            self.messages.pop_front();
        }
        self.messages.push_back(message);
    }
}
```

## 10. セキュリティ設計

### 10.1 設定暗号化
```rust
pub struct EncryptionService {
    key: [u8; 32],
}

impl EncryptionService {
    pub fn encrypt_settings(&self, data: &str) -> Result<String, EncryptionError>;
    pub fn decrypt_settings(&self, encrypted: &str) -> Result<String, EncryptionError>;
}
```

### 10.2 ログセキュリティ
- 機密データのマスキング機能
- ログファイルのアクセス制限
- 自動ログローテーション

## 11. テスト設計

### 11.1 テスト構成
```
tests/
├── unit/
│   ├── communication/
│   ├── services/
│   └── models/
├── integration/
│   ├── connection_flow.rs
│   └── settings_management.rs
└── e2e/
    └── terminal_operations.rs
```

### 11.2 モック設計
```rust
pub struct MockConnectionHandler {
    connected: bool,
    send_responses: VecDeque<Vec<u8>>,
}

#[async_trait]
impl ConnectionHandler for MockConnectionHandler {
    // テスト用実装
}
```

## 12. 開発環境設定

### 12.1 依存関係

#### Rust依存関係 (Cargo.toml)
```toml
[dependencies]
tauri = { version = "2.0", features = ["api-all"] }
tokio = { version = "1.0", features = ["full"] }
serialport = "4.2"
tokio-tcp = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4"] }
aes-gcm = "0.10"
tracing = "0.1"
```

#### Frontend依存関係 (package.json)
```json
{
  "devDependencies": {
    "@tailwindcss/forms": "^0.5.7",
    "@tailwindcss/typography": "^0.5.10",
    "autoprefixer": "^10.4.16",
    "postcss": "^8.4.32",
    "tailwindcss": "^3.3.6"
  }
}
```

### 12.2 開発ツール設定
- **Rust**: clippy, rustfmt
- **TypeScript**: ESLint, Prettier
- **CSS**: TailwindCSS + PostCSS + Autoprefixer
- **テスト**: cargo test, vitest
- **CI/CD**: GitHub Actions

## 13. 実装フェーズ計画

### Phase 1: 基本シリアル通信
- シリアルポート検出・接続
- 基本的な送受信機能
- 簡単なUI

### Phase 2: TCP通信対応
- TCP接続機能
- 通信方式切り替えUI
- エラーハンドリング強化

### Phase 3: 設定管理
- プロファイル管理
- 設定の永続化
- インポート/エクスポート

### Phase 4: ログ・履歴機能
- 通信ログ記録
- コマンド履歴
- ログエクスポート

### Phase 5: UI/UX改善
- テーマシステム
- レスポンシブ対応
- キーボードショートカット

### Phase 6: 高度機能
- マクロ機能
- ファイル転送
- プラグインシステム