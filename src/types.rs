//! Core types shared across the scanner library.
//!
//! Defines [`GError`], [`ScanStatus`], [`ScanResult`], and [`BridgeMessage`].

use std::fmt;
use std::net::Ipv4Addr;

/// Application-wide error type.
///
/// Captures both Win32 API errors (with numeric code) and internal
/// application-level errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GError {
    /// A Win32 API error with its error code and descriptive message.
    Win32(u32, String),
    /// An application-level error with a descriptive message.
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

/// Status of a specific IP scan.
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

/// Result of scanning a single IP address.
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

/// Messages exchanged between the UI and the scanner bridge.
#[derive(Debug, Clone)]
pub enum BridgeMessage {
    StartScan(String),
    /// Start a scan using typed IP addresses (no string parsing needed).
    StartScanRange(Ipv4Addr, Ipv4Addr),
    /// Request cancellation of the currently running scan.
    StopScan,
    ScanUpdate(ScanResult),
    /// Sent when a scan is completed successfully.
    ScanComplete,
    /// Sent when a scan is cancelled before completion.
    ScanCancelled,
    Progress(u8),
    Error(GError),
}
