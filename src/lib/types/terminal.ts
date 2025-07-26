export interface TerminalMessage {
  id: string;
  timestamp: string;
  direction: MessageDirection;
  content: string;
  encoding: string;
}

export enum MessageDirection {
  Sent = 'Sent',
  Received = 'Received',
}

export interface TerminalConfig {
  encoding: string;
  line_ending: LineEnding;
  echo_input: boolean;
  show_timestamp: boolean;
  font_family: string;
  font_size: number;
  theme: TerminalTheme;
  max_history_size: number;
  auto_scroll: boolean;
}

export enum LineEnding {
  Cr = 'Cr',
  Lf = 'Lf',
  CrLf = 'CrLf',
}

export interface TerminalTheme {
  background_color: string;
  text_color: string;
  input_color: string;
  timestamp_color: string;
  sent_color: string;
  received_color: string;
  error_color: string;
}

export interface CommandHistory {
  commands: string[];
  max_size: number;
  current_index: number | null;
}

// UI用の状態
export interface TerminalState {
  messages: TerminalMessage[];
  commandHistory: CommandHistory;
  currentInput: string;
  config: TerminalConfig;
  isLogging: boolean;
  selectedMessages: string[];
  searchQuery: string;
  filteredMessages: TerminalMessage[];
}

// 文字エンコーディング選択肢
export interface EncodingOption {
  value: string;
  label: string;
  description?: string;
}

export const ENCODING_OPTIONS: EncodingOption[] = [
  { value: 'UTF-8', label: 'UTF-8', description: 'Unicode（推奨）' },
  { value: 'ASCII', label: 'ASCII', description: 'ASCII 7-bit' },
  { value: 'Shift_JIS', label: 'Shift_JIS', description: '日本語（Windows）' },
  { value: 'EUC-JP', label: 'EUC-JP', description: '日本語（Unix）' },
  { value: 'ISO-2022-JP', label: 'ISO-2022-JP', description: '日本語（JIS）' },
  { value: 'UTF-16', label: 'UTF-16', description: 'Unicode 16-bit' },
];

export const LINE_ENDING_OPTIONS = [
  { value: LineEnding.CrLf, label: 'CR+LF (\\r\\n)', description: 'Windows標準' },
  { value: LineEnding.Lf, label: 'LF (\\n)', description: 'Unix/Linux標準' },
  { value: LineEnding.Cr, label: 'CR (\\r)', description: 'Mac (Classic)' },
];

export const FONT_FAMILIES = [
  'Fira Code',
  'Consolas',
  'Monaco',
  'Menlo',
  'Ubuntu Mono',
  'Source Code Pro',
  'Courier New',
  'monospace',
];

export const FONT_SIZES = [10, 11, 12, 13, 14, 16, 18, 20, 22, 24];

// デフォルト値
export function createDefaultTerminalConfig(): TerminalConfig {
  return {
    encoding: 'UTF-8',
    line_ending: LineEnding.CrLf,
    echo_input: true,
    show_timestamp: true,
    font_family: 'Fira Code',
    font_size: 14,
    theme: createDefaultTerminalTheme(),
    max_history_size: 1000,
    auto_scroll: true,
  };
}

export function createDefaultTerminalTheme(): TerminalTheme {
  return {
    background_color: '#1a1b26',
    text_color: '#a9b1d6',
    input_color: '#24283b',
    timestamp_color: '#565f89',
    sent_color: '#7aa2f7',
    received_color: '#9ece6a',
    error_color: '#f7768e',
  };
}

export function createDefaultCommandHistory(): CommandHistory {
  return {
    commands: [],
    max_size: 100,
    current_index: null,
  };
}

// ヘルパー関数
export function formatTimestamp(timestamp: string, config: TerminalConfig): string {
  if (!config.show_timestamp) return '';
  
  const date = new Date(timestamp);
  return date.toLocaleTimeString('ja-JP', {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    fractionalSecondDigits: 3,
  });
}

export function getMessageDisplayContent(message: TerminalMessage): string {
  // 制御文字を表示可能な文字に変換
  return message.content
    .replace(/\r\n/g, '↵\n')
    .replace(/\r/g, '↵')
    .replace(/\n/g, '↵\n')
    .replace(/\t/g, '→');
}

export function searchMessages(
  messages: TerminalMessage[],
  query: string,
  caseSensitive = false
): TerminalMessage[] {
  if (!query.trim()) return messages;
  
  const searchTerm = caseSensitive ? query : query.toLowerCase();
  
  return messages.filter((message) => {
    const content = caseSensitive ? message.content : message.content.toLowerCase();
    return content.includes(searchTerm);
  });
}

export function exportMessagesToText(
  messages: TerminalMessage[],
  config: TerminalConfig
): string {
  return messages
    .map((message) => {
      const timestamp = formatTimestamp(message.timestamp, config);
      const direction = message.direction === MessageDirection.Sent ? '送信' : '受信';
      const prefix = config.show_timestamp ? `[${timestamp}] ${direction}: ` : `${direction}: `;
      return prefix + message.content;
    })
    .join('\n');
}

export function exportMessagesToCSV(messages: TerminalMessage[]): string {
  const headers = ['タイムスタンプ', '方向', '内容', 'エンコーディング'];
  const rows = messages.map((message) => [
    message.timestamp,
    message.direction === MessageDirection.Sent ? '送信' : '受信',
    `"${message.content.replace(/"/g, '""')}"`, // CSVエスケープ
    message.encoding,
  ]);
  
  return [headers, ...rows].map((row) => row.join(',')).join('\n');
}

export function getLineEndingDisplayText(lineEnding: LineEnding): string {
  switch (lineEnding) {
    case LineEnding.Cr:
      return 'CR (\\r)';
    case LineEnding.Lf:
      return 'LF (\\n)';
    case LineEnding.CrLf:
      return 'CR+LF (\\r\\n)';
    default:
      return 'Unknown';
  }
}

export function validateTerminalConfig(config: TerminalConfig): string[] {
  const errors: string[] = [];
  
  if (config.font_size < 8 || config.font_size > 32) {
    errors.push('フォントサイズは8-32の範囲で指定してください');
  }
  
  if (config.max_history_size < 10 || config.max_history_size > 10000) {
    errors.push('履歴サイズは10-10000の範囲で指定してください');
  }
  
  // カラーコードの検証（簡易）
  const colorPattern = /^#[0-9A-Fa-f]{6}$/;
  const colorFields = [
    'background_color',
    'text_color',
    'input_color',
    'timestamp_color',
    'sent_color',
    'received_color',
    'error_color',
  ] as const;
  
  for (const field of colorFields) {
    const color = config.theme[field];
    if (!colorPattern.test(color)) {
      errors.push(`${field}の色コードが無効です`);
    }
  }
  
  return errors;
}