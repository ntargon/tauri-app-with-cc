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

## 2. ディレクトリ構造

```
src/
├── lib/
│   ├── components/           # UIコンポーネント
│   │   ├── Terminal/
│   │   │   ├── Terminal.svelte
│   │   │   ├── TerminalInput.svelte
│   │   │   └── TerminalOutput.svelte
│   │   ├── ConnectionPanel/
│   │   │   ├── ConnectionPanel.svelte
│   │   │   ├── SerialConfig.svelte
│   │   │   └── TcpConfig.svelte
│   │   ├── Settings/
│   │   │   ├── SettingsPanel.svelte
│   │   │   ├── ProfileManager.svelte
│   │   │   └── AppSettings.svelte
│   │   └── common/
│   │       ├── Button.svelte
│   │       ├── Input.svelte
│   │       └── Select.svelte
│   ├── stores/               # 状態管理
│   │   ├── connection.ts
│   │   ├── terminal.ts
│   │   ├── settings.ts
│   │   └── logs.ts
│   ├── types/                # 型定義
│   │   ├── connection.ts
│   │   ├── terminal.ts
│   │   └── settings.ts
│   └── utils/                # ユーティリティ
│       ├── formatters.ts
│       └── validators.ts
├── routes/
│   └── +layout.svelte
└── app.html

src-tauri/
├── src/
│   ├── commands/             # Tauriコマンド
│   │   ├── connection.rs
│   │   ├── terminal.rs
│   │   └── settings.rs
│   ├── communication/        # 通信層
│   │   ├── mod.rs
│   │   ├── serial.rs
│   │   └── tcp.rs
│   ├── models/               # データ構造
│   │   ├── connection.rs
│   │   ├── settings.rs
│   │   └── terminal.rs
│   ├── services/             # ビジネスロジック
│   │   ├── connection_manager.rs
│   │   ├── settings_manager.rs
│   │   └── log_manager.rs
│   ├── utils/                # ユーティリティ
│   │   ├── encryption.rs
│   │   └── file_utils.rs
│   └── lib.rs
├── Cargo.toml
└── tauri.conf.json
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

### 6.1 TailwindCSSデザインシステム

#### 6.1.1 カラーパレット
```javascript
// tailwind.config.js
module.exports = {
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

#### 6.2.1 ターミナルコンポーネント
```svelte
<!-- Terminal.svelte -->
<script lang="ts">
  import TerminalOutput from './TerminalOutput.svelte';
  import TerminalInput from './TerminalInput.svelte';
  import { terminal } from '$lib/stores/terminal';
  import { connection } from '$lib/stores/connection';
</script>

<div class="terminal-container">
  <TerminalOutput messages={$terminal.messages} config={$terminal.config} />
  <TerminalInput 
    bind:value={$terminal.currentInput}
    history={$terminal.commandHistory}
    disabled={$connection.connectionStatus !== 'connected'}
    on:send={handleSend}
  />
</div>
```

#### 6.2.2 接続パネル
```svelte
<!-- ConnectionPanel.svelte -->
<script lang="ts">
  import SerialConfig from './SerialConfig.svelte';
  import TcpConfig from './TcpConfig.svelte';
  import { connection } from '$lib/stores/connection';
  
  let selectedType: 'serial' | 'tcp' = 'serial';
</script>

<div class="connection-panel">
  <!-- タブ切り替え -->
  <div class="flex bg-gray-100 dark:bg-gray-700 rounded-lg p-1 mb-4">
    <button 
      class="flex-1 px-3 py-2 text-sm font-medium rounded-md transition-colors {selectedType === 'serial' 
        ? 'bg-white dark:bg-gray-600 text-gray-900 dark:text-white shadow-sm' 
        : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'}"
      on:click={() => selectedType = 'serial'}
    >
      シリアル通信
    </button>
    <button 
      class="flex-1 px-3 py-2 text-sm font-medium rounded-md transition-colors {selectedType === 'tcp' 
        ? 'bg-white dark:bg-gray-600 text-gray-900 dark:text-white shadow-sm' 
        : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'}"
      on:click={() => selectedType = 'tcp'}
    >
      TCP通信
    </button>
  </div>
  
  <!-- 接続設定 -->
  {#if selectedType === 'serial'}
    <SerialConfig />
  {:else}
    <TcpConfig />
  {/if}
  
  <!-- 接続制御 -->
  <div class="mt-6 flex items-center justify-between">
    <div class="flex items-center gap-2">
      <span class="connection-status {$connection.connectionStatus}">
        <span class="w-2 h-2 rounded-full bg-current mr-1"></span>
        {$connection.connectionStatus === 'connected' ? '接続中' : 
         $connection.connectionStatus === 'connecting' ? '接続中...' : 
         $connection.connectionStatus === 'error' ? 'エラー' : '未接続'}
      </span>
    </div>
    
    <button 
      class="px-4 py-2 text-sm font-medium rounded-md transition-colors {$connection.connectionStatus === 'connected' 
        ? 'bg-red-600 hover:bg-red-700 text-white' 
        : 'bg-blue-600 hover:bg-blue-700 text-white disabled:bg-gray-400'}"
      on:click={handleConnect} 
      disabled={$connection.connectionStatus === 'connecting'}
    >
      {$connection.connectionStatus === 'connected' ? '切断' : '接続'}
    </button>
  </div>
</div>
```

#### 6.2.3 シリアル設定コンポーネント
```svelte
<!-- SerialConfig.svelte -->
<script lang="ts">
  import Select from '../common/Select.svelte';
  import { connection } from '$lib/stores/connection';
  
  const baudRates = [9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600];
  const dataBits = [5, 6, 7, 8];
  const stopBits = [1, 1.5, 2];
  const parities = ['None', 'Even', 'Odd', 'Mark', 'Space'];
</script>

<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
  <div>
    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
      COMポート
    </label>
    <Select 
      options={$connection.availablePorts.map(port => ({ value: port, label: port }))}
      bind:value={$connection.currentConnection?.serial_config?.port}
      placeholder="ポートを選択"
      class="w-full"
    />
  </div>
  
  <div>
    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
      ボーレート
    </label>
    <Select 
      options={baudRates.map(rate => ({ value: rate, label: rate.toString() }))}
      bind:value={$connection.currentConnection?.serial_config?.baud_rate}
      class="w-full"
    />
  </div>
  
  <div>
    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
      データビット
    </label>
    <Select 
      options={dataBits.map(bits => ({ value: bits, label: bits.toString() }))}
      bind:value={$connection.currentConnection?.serial_config?.data_bits}
      class="w-full"
    />
  </div>
  
  <div>
    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
      ストップビット
    </label>
    <Select 
      options={stopBits.map(bits => ({ value: bits, label: bits.toString() }))}
      bind:value={$connection.currentConnection?.serial_config?.stop_bits}
      class="w-full"
    />
  </div>
</div>
```

### 6.3 レスポンシブデザイン
```svelte
<!-- メインレイアウト -->
<div class="h-screen bg-gray-50 dark:bg-gray-900 flex flex-col lg:flex-row">
  <!-- サイドバー（設定パネル） -->
  <div class="w-full lg:w-80 xl:w-96 border-b lg:border-b-0 lg:border-r border-gray-200 dark:border-gray-700 p-4">
    <ConnectionPanel />
  </div>
  
  <!-- メインエリア（ターミナル） -->
  <div class="flex-1 flex flex-col">
    <Terminal />
  </div>
</div>
```

## 7. データ永続化設計

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