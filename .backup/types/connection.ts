export interface ConnectionConfig {
  id: string;
  name: string;
  connection_type: ConnectionType;
  serial_config?: SerialConfig;
  tcp_config?: TcpConfig;
  created_at: string;
  updated_at: string;
}

export enum ConnectionType {
  Serial = 'Serial',
  Tcp = 'Tcp',
}

export interface SerialConfig {
  port: string;
  baud_rate: number;
  data_bits: DataBits;
  stop_bits: StopBits;
  parity: Parity;
  flow_control: FlowControl;
}

export interface TcpConfig {
  host: string;
  port: number;
  timeout: number; // milliseconds
  keep_alive: boolean;
}

export enum DataBits {
  Five = 'Five',
  Six = 'Six',
  Seven = 'Seven',
  Eight = 'Eight',
}

export enum StopBits {
  One = 'One',
  OnePointFive = 'OnePointFive',
  Two = 'Two',
}

export enum Parity {
  None = 'None',
  Even = 'Even',
  Odd = 'Odd',
  Mark = 'Mark',
  Space = 'Space',
}

export enum FlowControl {
  None = 'None',
  Software = 'Software',
  Hardware = 'Hardware',
}

export enum ConnectionStatus {
  Disconnected = 'Disconnected',
  Connecting = 'Connecting',
  Connected = 'Connected',
  Error = 'Error',
}

// UI用の選択肢オプション
export interface SelectOption<T = string> {
  value: T;
  label: string;
  disabled?: boolean;
}

// 接続関連のUI状態
export interface ConnectionState {
  currentConnection: ConnectionConfig | null;
  connectionStatus: ConnectionStatus;
  availablePorts: string[];
  profiles: ConnectionConfig[];
  isLoading: boolean;
  error: string | null;
}

// シリアルポート情報
export interface SerialPortInfo {
  port_name: string;
  port_type?: string;
  vid?: number;
  pid?: number;
  serial_number?: string;
  manufacturer?: string;
  product?: string;
}

// 定数
export const BAUD_RATES = [
  1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600,
] as const;

export const DATA_BITS_OPTIONS: SelectOption<DataBits>[] = [
  { value: DataBits.Five, label: '5' },
  { value: DataBits.Six, label: '6' },
  { value: DataBits.Seven, label: '7' },
  { value: DataBits.Eight, label: '8' },
];

export const STOP_BITS_OPTIONS: SelectOption<StopBits>[] = [
  { value: StopBits.One, label: '1' },
  { value: StopBits.OnePointFive, label: '1.5' },
  { value: StopBits.Two, label: '2' },
];

export const PARITY_OPTIONS: SelectOption<Parity>[] = [
  { value: Parity.None, label: 'なし' },
  { value: Parity.Even, label: '偶数' },
  { value: Parity.Odd, label: '奇数' },
  { value: Parity.Mark, label: 'マーク' },
  { value: Parity.Space, label: 'スペース' },
];

export const FLOW_CONTROL_OPTIONS: SelectOption<FlowControl>[] = [
  { value: FlowControl.None, label: 'なし' },
  { value: FlowControl.Software, label: 'ソフトウェア' },
  { value: FlowControl.Hardware, label: 'ハードウェア' },
];

// ヘルパー関数
export function createDefaultSerialConfig(): SerialConfig {
  return {
    port: '',
    baud_rate: 115200,
    data_bits: DataBits.Eight,
    stop_bits: StopBits.One,
    parity: Parity.None,
    flow_control: FlowControl.None,
  };
}

export function createDefaultTcpConfig(): TcpConfig {
  return {
    host: '127.0.0.1',
    port: 8080,
    timeout: 5000,
    keep_alive: true,
  };
}

export function createNewConnectionConfig(
  name: string,
  type: ConnectionType,
  config: SerialConfig | TcpConfig
): Omit<ConnectionConfig, 'id' | 'created_at' | 'updated_at'> {
  const now = new Date().toISOString();
  return {
    name,
    connection_type: type,
    ...(type === ConnectionType.Serial
      ? { serial_config: config as SerialConfig }
      : { tcp_config: config as TcpConfig }),
  };
}

export function getConnectionDisplayName(config: ConnectionConfig): string {
  if (config.connection_type === ConnectionType.Serial && config.serial_config) {
    return `${config.name} (${config.serial_config.port})`;
  } else if (config.connection_type === ConnectionType.Tcp && config.tcp_config) {
    return `${config.name} (${config.tcp_config.host}:${config.tcp_config.port})`;
  }
  return config.name;
}

export function validateSerialConfig(config: SerialConfig): string[] {
  const errors: string[] = [];
  
  if (!config.port.trim()) {
    errors.push('ポートを選択してください');
  }
  
  if (config.baud_rate <= 0) {
    errors.push('有効なボーレートを入力してください');
  }
  
  return errors;
}

export function validateTcpConfig(config: TcpConfig): string[] {
  const errors: string[] = [];
  
  if (!config.host.trim()) {
    errors.push('ホストアドレスを入力してください');
  }
  
  if (config.port <= 0 || config.port > 65535) {
    errors.push('有効なポート番号（1-65535）を入力してください');
  }
  
  if (config.timeout <= 0) {
    errors.push('有効なタイムアウト値を入力してください');
  }
  
  return errors;
}