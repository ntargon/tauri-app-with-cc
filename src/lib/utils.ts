// ユーティリティ関数

export function generateId(): string {
  return crypto.randomUUID();
}

export function formatTimestamp(date: Date): string {
  return date.toLocaleTimeString('ja-JP', {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    fractionalSecondDigits: 3
  });
}

export function validateSerialPort(port: string): boolean {
  return port.trim().length > 0;
}

export function validateTcpConnection(host: string, port: number): boolean {
  const hostValid = host.trim().length > 0;
  const portValid = port > 0 && port <= 65535;
  return hostValid && portValid;
}

export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

export function hexToString(hex: string): string {
  try {
    return hex.replace(/[^0-9A-Fa-f]/g, '').replace(/.{2}/g, (byte) => 
      String.fromCharCode(parseInt(byte, 16))
    );
  } catch {
    return hex;
  }
}

export function stringToHex(str: string): string {
  return str.split('').map(char => 
    char.charCodeAt(0).toString(16).padStart(2, '0')
  ).join(' ').toUpperCase();
}