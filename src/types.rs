use std::fmt;
use std::net::Ipv4Addr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GError {
    Win32(u32, String),
    Internal(String),
}

impl fmt::Display for GError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GError::Win32(code, msg) => write!(f, "Win32 Error ({}): {}", code, msg),
            GError::Internal(msg) => write!(f, "Internal Error: {}", msg),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanStatus {
    Scanning,
    Online,
    Offline,
    SystemError(GError),
}

impl fmt::Display for ScanStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScanStatus::Scanning => write!(f, "Scanning..."),
            ScanStatus::Online => write!(f, "Online"),
            ScanStatus::Offline => write!(f, "Offline"),
            ScanStatus::SystemError(e) => write!(f, "Error: {}", e),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub ip: Ipv4Addr,
    pub hostname: Option<String>,
    pub mac: Option<String>,
    pub vendor: Option<String>,
    pub status: ScanStatus,
    pub open_ports: Vec<u16>,
}

impl ScanResult {
    pub fn new(ip: Ipv4Addr) -> Self {
        Self {
            ip,
            hostname: None,
            mac: None,
            vendor: None,
            status: ScanStatus::Scanning,
            open_ports: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BridgeMessage {
    StartScan(String),
    ScanUpdate(ScanResult),
    ScanComplete,
    Progress(u8),
    Error(GError),
}
