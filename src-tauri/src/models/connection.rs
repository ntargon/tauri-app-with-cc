use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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
    #[serde(with = "duration_serde")]
    pub timeout: Duration,
    pub keep_alive: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DataBits {
    Five,
    Six,
    Seven,
    Eight,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum StopBits {
    One,
    OnePointFive,
    Two,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Parity {
    None,
    Even,
    Odd,
    Mark,
    Space,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum FlowControl {
    None,
    Software,
    Hardware,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

// Duration serialization helper
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_millis().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

impl Default for SerialConfig {
    fn default() -> Self {
        Self {
            port: String::new(),
            baud_rate: 115200,
            data_bits: DataBits::Eight,
            stop_bits: StopBits::One,
            parity: Parity::None,
            flow_control: FlowControl::None,
        }
    }
}

impl Default for TcpConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            timeout: Duration::from_secs(5),
            keep_alive: true,
        }
    }
}

impl ConnectionConfig {
    pub fn new_serial(name: String, serial_config: SerialConfig) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            connection_type: ConnectionType::Serial,
            serial_config: Some(serial_config),
            tcp_config: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn new_tcp(name: String, tcp_config: TcpConfig) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            connection_type: ConnectionType::Tcp,
            serial_config: None,
            tcp_config: Some(tcp_config),
            created_at: now,
            updated_at: now,
        }
    }
}

// Convert between our types and serialport types
impl From<DataBits> for serialport::DataBits {
    fn from(value: DataBits) -> Self {
        match value {
            DataBits::Five => serialport::DataBits::Five,
            DataBits::Six => serialport::DataBits::Six,
            DataBits::Seven => serialport::DataBits::Seven,
            DataBits::Eight => serialport::DataBits::Eight,
        }
    }
}

impl From<StopBits> for serialport::StopBits {
    fn from(value: StopBits) -> Self {
        match value {
            StopBits::One => serialport::StopBits::One,
            StopBits::OnePointFive => serialport::StopBits::Two, // Note: serialport crate doesn't have 1.5
            StopBits::Two => serialport::StopBits::Two,
        }
    }
}

impl From<Parity> for serialport::Parity {
    fn from(value: Parity) -> Self {
        match value {
            Parity::None => serialport::Parity::None,
            Parity::Even => serialport::Parity::Even,
            Parity::Odd => serialport::Parity::Odd,
            Parity::Mark => serialport::Parity::None, // Note: serialport crate doesn't have Mark/Space
            Parity::Space => serialport::Parity::None,
        }
    }
}

impl From<FlowControl> for serialport::FlowControl {
    fn from(value: FlowControl) -> Self {
        match value {
            FlowControl::None => serialport::FlowControl::None,
            FlowControl::Software => serialport::FlowControl::Software,
            FlowControl::Hardware => serialport::FlowControl::Hardware,
        }
    }
}