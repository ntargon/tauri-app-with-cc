use super::{ConnectionError, ConnectionHandler, ConnectionResult};
use crate::models::{ConnectionConfig, SerialConfig, TerminalMessage};
use async_trait::async_trait;
use serialport::{SerialPort, SerialPortType};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

pub struct SerialHandler {
    config: SerialConfig,
    port: Arc<Mutex<Option<Box<dyn SerialPort>>>>,
    is_connected: Arc<Mutex<bool>>,
}

impl SerialHandler {
    pub fn new(config: SerialConfig) -> Self {
        Self {
            config,
            port: Arc::new(Mutex::new(None)),
            is_connected: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn list_available_ports() -> ConnectionResult<Vec<String>> {
        match serialport::available_ports() {
            Ok(ports) => {
                let port_names: Vec<String> = ports
                    .into_iter()
                    .map(|port| port.port_name)
                    .collect();
                
                debug!("Available serial ports: {:?}", port_names);
                Ok(port_names)
            }
            Err(e) => {
                error!("Failed to list serial ports: {}", e);
                Err(ConnectionError::SerialError(e))
            }
        }
    }

    pub async fn get_port_info() -> ConnectionResult<Vec<SerialPortInfo>> {
        match serialport::available_ports() {
            Ok(ports) => {
                let port_info: Vec<SerialPortInfo> = ports
                    .into_iter()
                    .map(|port| SerialPortInfo {
                        port_name: port.port_name,
                        port_type: match &port.port_type {
                            SerialPortType::UsbPort(info) => Some(format!(
                                "USB VID:{:04X} PID:{:04X}", 
                                info.vid, 
                                info.pid
                            )),
                            SerialPortType::BluetoothPort => Some("Bluetooth".to_string()),
                            SerialPortType::PciPort => Some("PCI".to_string()),
                            SerialPortType::Unknown => None,
                        },
                        vid: match &port.port_type {
                            SerialPortType::UsbPort(info) => Some(info.vid),
                            _ => None,
                        },
                        pid: match &port.port_type {
                            SerialPortType::UsbPort(info) => Some(info.pid),
                            _ => None,
                        },
                        serial_number: match &port.port_type {
                            SerialPortType::UsbPort(info) => info.serial_number.clone(),
                            _ => None,
                        },
                        manufacturer: match &port.port_type {
                            SerialPortType::UsbPort(info) => info.manufacturer.clone(),
                            _ => None,
                        },
                        product: match &port.port_type {
                            SerialPortType::UsbPort(info) => info.product.clone(),
                            _ => None,
                        },
                    })
                    .collect();
                
                debug!("Serial port info: {:?}", port_info);
                Ok(port_info)
            }
            Err(e) => {
                error!("Failed to get serial port info: {}", e);
                Err(ConnectionError::SerialError(e))
            }
        }
    }

    async fn create_port(&self) -> ConnectionResult<Box<dyn SerialPort>> {
        let builder = serialport::new(&self.config.port, self.config.baud_rate)
            .data_bits(self.config.data_bits.clone().into())
            .stop_bits(self.config.stop_bits.clone().into())
            .parity(self.config.parity.clone().into())
            .flow_control(self.config.flow_control.clone().into())
            .timeout(Duration::from_millis(1000));

        match builder.open() {
            Ok(port) => {
                info!("Serial port {} opened successfully", self.config.port);
                Ok(port)
            }
            Err(serialport::Error { kind, description }) => {
                error!("Failed to open serial port {}: {:?} - {}", 
                       self.config.port, kind, description);
                
                match kind {
                    serialport::ErrorKind::NoDevice => {
                        Err(ConnectionError::PortNotFound(self.config.port.clone()))
                    }
                    serialport::ErrorKind::Io(std::io::ErrorKind::PermissionDenied) => {
                        Err(ConnectionError::PermissionDenied)
                    }
                    _ => Err(ConnectionError::SerialError(serialport::Error { kind, description }))
                }
            }
        }
    }
}

#[async_trait]
impl ConnectionHandler for SerialHandler {
    async fn connect(&mut self, _config: &ConnectionConfig) -> ConnectionResult<()> {
        debug!("Attempting to connect to serial port: {}", self.config.port);

        // ポートが存在するかチェック
        let available_ports = Self::list_available_ports().await?;
        if !available_ports.contains(&self.config.port) {
            return Err(ConnectionError::PortNotFound(self.config.port.clone()));
        }

        // ポートを開く
        let port = self.create_port().await?;
        
        // ポートを保存
        {
            let mut port_guard = self.port.lock().await;
            *port_guard = Some(port);
        }

        // 接続状態を更新
        {
            let mut connected = self.is_connected.lock().await;
            *connected = true;
        }

        info!("Successfully connected to serial port: {}", self.config.port);
        Ok(())
    }

    async fn disconnect(&mut self) -> ConnectionResult<()> {
        debug!("Disconnecting from serial port: {}", self.config.port);

        // ポートを閉じる
        {
            let mut port_guard = self.port.lock().await;
            if let Some(mut port) = port_guard.take() {
                // ポートを明示的にフラッシュしてから閉じる
                let _ = port.flush();
                drop(port);
            }
        }

        // 接続状態を更新
        {
            let mut connected = self.is_connected.lock().await;
            *connected = false;
        }

        info!("Disconnected from serial port: {}", self.config.port);
        Ok(())
    }

    async fn send(&mut self, data: &[u8]) -> ConnectionResult<()> {
        let mut port_guard = self.port.lock().await;
        
        if let Some(port) = port_guard.as_mut() {
            match port.write_all(data) {
                Ok(_) => {
                    match port.flush() {
                        Ok(_) => {
                            debug!("Sent {} bytes to serial port", data.len());
                            Ok(())
                        }
                        Err(e) => {
                            error!("Failed to flush serial port: {}", e);
                            Err(ConnectionError::SendFailed(e.to_string()))
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to write to serial port: {}", e);
                    Err(ConnectionError::SendFailed(e.to_string()))
                }
            }
        } else {
            Err(ConnectionError::ConnectionClosed)
        }
    }

    async fn start_receive_loop(&mut self, tx: mpsc::UnboundedSender<TerminalMessage>) -> ConnectionResult<()> {
        let port_arc = self.port.clone();
        let is_connected_arc = self.is_connected.clone();
        let port_name = self.config.port.clone();

        tokio::spawn(async move {
            let mut buffer = [0u8; 1024];
            
            loop {
                // 接続状態をチェック
                {
                    let connected = is_connected_arc.lock().await;
                    if !*connected {
                        debug!("Receive loop stopped: not connected");
                        break;
                    }
                }

                // データを読み取り
                let result = {
                    let mut port_guard = port_arc.lock().await;
                    if let Some(port) = port_guard.as_mut() {
                        // タイムアウト付きで読み取り
                        let read_result = timeout(
                            Duration::from_millis(100),
                            tokio::task::spawn_blocking({
                                let mut port_clone = port.try_clone().unwrap();
                                move || port_clone.read(&mut buffer)
                            })
                        ).await;

                        match read_result {
                            Ok(Ok(Ok(bytes_read))) => Some(Ok(bytes_read)),
                            Ok(Ok(Err(e))) => Some(Err(e)),
                            Ok(Err(_)) => {
                                // join error
                                Some(Err(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    "Task join error"
                                )))
                            }
                            Err(_) => {
                                // timeout
                                None
                            }
                        }
                    } else {
                        debug!("Receive loop stopped: port closed");
                        break;
                    }
                };

                match result {
                    Some(Ok(bytes_read)) if bytes_read > 0 => {
                        let data = &buffer[..bytes_read];
                        let content = String::from_utf8_lossy(data).to_string();
                        
                        debug!("Received {} bytes from serial port: {:?}", bytes_read, content);
                        
                        let message = TerminalMessage::new_received(content, "UTF-8".to_string());
                        
                        if tx.send(message).is_err() {
                            warn!("Failed to send received message to channel");
                            break;
                        }
                    }
                    Some(Ok(_)) => {
                        // 0 bytes read, continue
                    }
                    Some(Err(e)) => {
                        match e.kind() {
                            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => {
                                // タイムアウトやWouldBlockは正常、続行
                            }
                            _ => {
                                error!("Serial receive error: {}", e);
                                // エラーメッセージを送信
                                let error_message = TerminalMessage::new_received(
                                    format!("Error: {}", e),
                                    "UTF-8".to_string()
                                );
                                let _ = tx.send(error_message);
                                break;
                            }
                        }
                    }
                    None => {
                        // タイムアウト、続行
                    }
                }

                // 短時間スリープしてCPU使用率を下げる
                tokio::time::sleep(Duration::from_millis(1)).await;
            }

            info!("Serial receive loop ended for port: {}", port_name);
        });

        Ok(())
    }

    fn is_connected(&self) -> bool {
        // Note: この関数は同期的なので、Arcの値を直接チェックできない
        // 実際の実装では、AtomicBoolを使用するか、別の方法を検討する必要がある
        true // 暫定的な実装
    }

    fn get_connection_info(&self) -> Option<String> {
        Some(format!(
            "Serial: {} @ {} baud, {}-{}-{}",
            self.config.port,
            self.config.baud_rate,
            match self.config.data_bits {
                crate::models::DataBits::Eight => "8",
                crate::models::DataBits::Seven => "7",
                crate::models::DataBits::Six => "6",
                crate::models::DataBits::Five => "5",
            },
            match self.config.parity {
                crate::models::Parity::None => "N",
                crate::models::Parity::Even => "E",
                crate::models::Parity::Odd => "O",
                crate::models::Parity::Mark => "M",
                crate::models::Parity::Space => "S",
            },
            match self.config.stop_bits {
                crate::models::StopBits::One => "1",
                crate::models::StopBits::OnePointFive => "1.5",
                crate::models::StopBits::Two => "2",
            }
        ))
    }
}

#[derive(Debug, Clone)]
pub struct SerialPortInfo {
    pub port_name: String,
    pub port_type: Option<String>,
    pub vid: Option<u16>,
    pub pid: Option<u16>,
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
}