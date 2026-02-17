//! Network primitives for IP scanning on Windows.
//!
//! Provides the [`NetworkProvider`] trait and the [`NetUtils`] implementation
//! using Win32 APIs (`IcmpSendEcho`, `SendARP`) and Tokio for port scanning.

use crate::types::GError;
use lazy_static::lazy_static;
use std::ffi::c_void;
use std::future::Future;
use std::mem;
use std::net::Ipv4Addr;
use std::pin::Pin;
use std::time::Duration;
use tokio::net::TcpStream;
use windows::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::NetworkManagement::IpHelper::{
    IcmpCloseHandle, IcmpCreateFile, IcmpSendEcho, SendARP, ICMP_ECHO_REPLY,
};

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

lazy_static! {
    static ref OUI_DB: Option<mac_oui::Oui> = mac_oui::Oui::default().ok();
}

/// RAII wrapper for Win32 handles.
struct SafeHandle(HANDLE);

impl SafeHandle {
    fn new(h: HANDLE) -> Result<Self, GError> {
        if h == INVALID_HANDLE_VALUE || h.0 == 0 {
            Err(GError::Internal("Invalid Win32 Handle".to_string()))
        } else {
            Ok(SafeHandle(h))
        }
    }
}

impl Drop for SafeHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = IcmpCloseHandle(self.0);
        }
    }
}

/// Trait to abstract network operations, enabling mocking for tests.
pub trait NetworkProvider: Send + Sync {
    /// Sends an ICMP echo request. Returns `true` if the host responds.
    fn ping(&self, ip: Ipv4Addr) -> Result<bool, GError>;
    /// Resolves the MAC address via ARP. Returns `None` if unreachable.
    fn resolve_mac(&self, ip: Ipv4Addr) -> Result<Option<String>, GError>;
    /// Performs reverse DNS lookup. Returns `None` if no hostname found.
    fn resolve_hostname(&self, ip: Ipv4Addr) -> Result<Option<String>, GError>;
    /// Looks up the OUI vendor name for a given MAC address.
    fn resolve_vendor(&self, mac: &str) -> Option<String>;
    /// Probes a TCP port. Returns `true` if the port is open.
    fn scan_port(&self, ip: Ipv4Addr, port: u16) -> BoxFuture<'_, bool>;
}

/// Implementation of [`NetworkProvider`] using standard Windows APIs.
pub struct NetUtils;

impl NetUtils {
    /// Creates a new instance of [`NetUtils`].
    pub fn new() -> Self {
        Self
    }
}

impl Default for NetUtils {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkProvider for NetUtils {
    fn resolve_mac(&self, ip: Ipv4Addr) -> Result<Option<String>, GError> {
        let dest_ip_final = u32::from_le_bytes(ip.octets());
        // Win32 SendARP requires MAXLEN_PHYSADDR (8) bytes minimum, even if MAC is 6.
        let mut mac_buffer = [0u8; 8];
        let mut mac_len = mac_buffer.len() as u32;

        let res = unsafe {
            SendARP(
                dest_ip_final,
                0,
                mac_buffer.as_mut_ptr() as *mut c_void,
                &mut mac_len,
            )
        };

        if res == 0 {
            if mac_len >= 6 {
                let mac_str = format!(
                    "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                    mac_buffer[0],
                    mac_buffer[1],
                    mac_buffer[2],
                    mac_buffer[3],
                    mac_buffer[4],
                    mac_buffer[5]
                );
                Ok(Some(mac_str))
            } else {
                // Should not happen for Ethernet, but handle safely
                log::error!(
                    "SendARP succeeded but returned invalid mac_len: {}",
                    mac_len
                );
                Ok(None)
            }
        } else if res == 67 || res == 1168 {
            // 67 = ERROR_BAD_NET_NAME (Host not found), 1168 = ERROR_NOT_FOUND
            Ok(None)
        } else {
            log::error!("SendARP failed for {} with error code: {}", ip, res);
            Err(GError::Win32(res, "SendARP failed".to_string()))
        }
    }

    fn resolve_vendor(&self, mac_str: &str) -> Option<String> {
        OUI_DB.as_ref().and_then(|db| {
            // mac_oui version 0.4 uses lookup_by_mac
            db.lookup_by_mac(mac_str)
                .ok()
                .flatten()
                .map(|e| e.company_name.clone())
        })
    }

    fn resolve_hostname(&self, ip: Ipv4Addr) -> Result<Option<String>, GError> {
        match dns_lookup::lookup_addr(&ip.into()) {
            Ok(hostname) => {
                if hostname == ip.to_string() {
                    Ok(None)
                } else {
                    Ok(Some(hostname))
                }
            }
            Err(_e) => Ok(None),
        }
    }

    fn ping(&self, ip: Ipv4Addr) -> Result<bool, GError> {
        let raw_handle = unsafe { IcmpCreateFile() }
            .map_err(|e| GError::Win32(0, format!("IcmpCreateFile failed: {}", e)))?;

        let handle = SafeHandle::new(raw_handle)?;

        let dest_ip = u32::from_le_bytes(ip.octets());
        let request_data = b"PingPayload";
        let request_size = request_data.len() as u16;

        let reply_size = mem::size_of::<ICMP_ECHO_REPLY>() + request_size as usize + 8;
        let mut reply_buffer = vec![0u8; reply_size];

        let ret = unsafe {
            IcmpSendEcho(
                handle.0,
                dest_ip,
                request_data.as_ptr() as *const c_void,
                request_size,
                None,
                reply_buffer.as_mut_ptr() as *mut c_void,
                reply_size as u32,
                1000,
            )
        };

        Ok(ret > 0)
    }

    fn scan_port(&self, ip: Ipv4Addr, port: u16) -> BoxFuture<'_, bool> {
        Box::pin(async move {
            let addr = format!("{}:{}", ip, port);
            matches!(
                tokio::time::timeout(Duration::from_millis(500), TcpStream::connect(addr)).await,
                Ok(Ok(_))
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_mac_safety() {
        // REGRESSION TEST: Verification that SendARP does not crash the process due to stack overflow.
        // This test calls resolve_mac on localhost. It doesn't matter if it returns Some, None, or Err.
        // It ONLY matters that it returns and doesn't segfault.
        let net = NetUtils::new();
        let _ = net.resolve_mac(Ipv4Addr::new(127, 0, 0, 1));
    }
}

/// Mock implementation of [`NetworkProvider`] for deterministic testing.
///
/// Available when the `test-support` feature is enabled.
#[cfg(any(test, feature = "test-support"))]
pub struct MockNet;

#[cfg(any(test, feature = "test-support"))]
impl NetworkProvider for MockNet {
    fn ping(&self, ip: Ipv4Addr) -> Result<bool, GError> {
        if ip == Ipv4Addr::new(192, 168, 1, 1) {
            Ok(true)
        } else if ip == Ipv4Addr::new(192, 168, 1, 2) {
            Err(GError::Internal("Simulated Failure".to_string()))
        } else {
            Ok(false)
        }
    }

    fn resolve_mac(&self, ip: Ipv4Addr) -> Result<Option<String>, GError> {
        if ip == Ipv4Addr::new(192, 168, 1, 1) {
            Ok(Some("00:11:22:33:44:55".to_string()))
        } else {
            Ok(None)
        }
    }

    fn resolve_hostname(&self, ip: Ipv4Addr) -> Result<Option<String>, GError> {
        if ip == Ipv4Addr::new(192, 168, 1, 1) {
            Ok(Some("mock-host".to_string()))
        } else {
            Ok(None)
        }
    }

    fn resolve_vendor(&self, _mac: &str) -> Option<String> {
        Some("Mock Vendor".to_string())
    }

    fn scan_port(&self, _ip: Ipv4Addr, port: u16) -> BoxFuture<'_, bool> {
        Box::pin(async move { port == 80 })
    }
}
