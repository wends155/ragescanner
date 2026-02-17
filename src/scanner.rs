//! Async scan engine with semaphore-controlled concurrency.
//!
//! The [`Scanner`] struct orchestrates per-IP scanning (ping, ARP, DNS,
//! port scan) and streams results via a Tokio channel.

use crate::net::NetworkProvider;
use crate::types::{BridgeMessage, GError, ScanResult, ScanStatus};
use std::net::Ipv4Addr;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::Semaphore;

/// Async scan engine that probes IPs for reachability, MAC, hostname, and open ports.
pub struct Scanner {
    net_utils: Arc<dyn NetworkProvider>,
    tx_bridge: Sender<BridgeMessage>,
}

const COMMON_PORTS: &[u16] = &[
    21, 22, 23, 25, 53, 80, 110, 135, 139, 443, 445, 1433, 3306, 3389, 5432, 8080,
];

const MAX_CONCURRENT_TASKS: usize = 100;

impl Scanner {
    /// Creates a new scanner with the given network provider and result channel.
    pub fn new(net_utils: Arc<dyn NetworkProvider>, tx_bridge: Sender<BridgeMessage>) -> Self {
        Self {
            net_utils,
            tx_bridge,
        }
    }

    /// Scans a contiguous range of IPv4 addresses.
    ///
    /// Sends [`BridgeMessage::ScanUpdate`], [`BridgeMessage::Progress`], and
    /// [`BridgeMessage::ScanComplete`] through the channel.
    pub async fn scan_range(&self, start_ip: Ipv4Addr, end_ip: Ipv4Addr) {
        let start_u32: u32 = u32::from(start_ip);
        let end_u32: u32 = u32::from(end_ip);

        // Simple validation
        if start_u32 > end_u32 {
            let _ = self
                .tx_bridge
                .send(BridgeMessage::Error(GError::Internal(
                    "Invalid IP Range".to_string(),
                )))
                .await;
            return;
        }

        log::info!(
            "Starting scan for range: {} - {} (Total: {})",
            start_ip,
            end_ip,
            end_u32 - start_u32 + 1
        );
        let total_ips = end_u32 - start_u32 + 1;
        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_TASKS));
        let mut tasks = tokio::task::JoinSet::new();

        for (idx, i) in (start_u32..=end_u32).enumerate() {
            let ip = Ipv4Addr::from(i);
            let semaphore_clone = semaphore.clone();
            let permit_res = semaphore_clone.acquire_owned().await;

            let permit = match permit_res {
                Ok(p) => p,
                Err(e) => {
                    let _ = self
                        .tx_bridge
                        .send(BridgeMessage::Error(GError::Internal(format!(
                            "Semaphore closed: {}",
                            e
                        ))))
                        .await;
                    break;
                }
            };

            let net_utils = self.net_utils.clone();
            let tx = self.tx_bridge.clone();
            let current_count = idx as u32 + 1;

            tasks.spawn(async move {
                let _permit = permit;
                let mut result = ScanResult::new(ip);
                log::info!("Scanning: {}", ip);

                let net_utils_blocking = net_utils.clone();
                let blocking_task = tokio::task::spawn_blocking(move || {
                    let mut is_online = false;
                    let mut system_error = None;

                    // Try Ping
                    match net_utils_blocking.ping(ip) {
                        Ok(true) => is_online = true,
                        Ok(false) => {}
                        Err(e) => system_error = Some(e),
                    }

                    // Try ARP
                    if system_error.is_none() {
                        match net_utils_blocking.resolve_mac(ip) {
                            Ok(Some(mac)) => {
                                let hostname =
                                    net_utils_blocking.resolve_hostname(ip).unwrap_or(None);
                                let vendor = net_utils_blocking.resolve_vendor(&mac);
                                return Ok((true, Some(mac), hostname, vendor));
                            }
                            Ok(None) => {}
                            Err(e) => system_error = Some(e),
                        }
                    }

                    if let Some(err) = system_error {
                        Err(err)
                    } else {
                        let hostname = net_utils_blocking.resolve_hostname(ip).unwrap_or(None);
                        Ok((is_online, None, hostname, None))
                    }
                })
                .await;

                match blocking_task {
                    Ok(Ok((is_online, mac, hostname, vendor))) => {
                        log::info!("Scan result for {}: online={}", ip, is_online);
                        // Force reporting for debugging
                        if true {
                            // was if is_online
                            if is_online {
                                result.status = ScanStatus::Online;
                            } else {
                                result.status = ScanStatus::Offline;
                            }
                            result.mac = mac;
                            result.hostname = hostname;
                            result.vendor = vendor;

                            // Port Scan (Async)
                            if is_online {
                                let mut open_ports = Vec::new();
                                for &port in COMMON_PORTS {
                                    if net_utils.scan_port(ip, port).await {
                                        open_ports.push(port);
                                    }
                                }
                                result.open_ports = open_ports;
                            }

                            let _ = tx.send(BridgeMessage::ScanUpdate(result)).await;
                        }
                    }
                    Ok(Err(e)) => {
                        log::error!("System error scanning {}: {}", ip, e);
                        result.status = ScanStatus::SystemError(e);
                        let _ = tx.send(BridgeMessage::ScanUpdate(result)).await;
                    }
                    Err(e) => {
                        result.status = ScanStatus::SystemError(GError::Internal(format!(
                            "Task failed: {}",
                            e
                        )));
                        let _ = tx.send(BridgeMessage::ScanUpdate(result)).await;
                    }
                }

                // Progress Update
                let progress = (current_count as f32 / total_ips as f32 * 100.0) as u8;
                let _ = tx.send(BridgeMessage::Progress(progress)).await;
            });
        }

        while tasks.join_next().await.is_some() {}
        log::info!("Scan complete.");
        let _ = self.tx_bridge.send(BridgeMessage::ScanComplete).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::MockNet;
    use tokio::sync::mpsc::channel;

    #[tokio::test]
    async fn test_scanner_with_ports_and_progress() {
        let (tx, mut rx) = channel(100);
        let scanner = Scanner::new(Arc::new(MockNet), tx);

        let start = Ipv4Addr::new(192, 168, 1, 1);
        let end = Ipv4Addr::new(192, 168, 1, 1); // Single IP

        scanner.scan_range(start, end).await;

        let mut found_online = false;
        let mut found_progress = false;
        let mut complete = false;

        while let Some(msg) = rx.recv().await {
            match msg {
                BridgeMessage::ScanUpdate(res) => {
                    if res.ip == Ipv4Addr::new(192, 168, 1, 1) {
                        assert_eq!(res.status, ScanStatus::Online);
                        assert!(res.open_ports.contains(&80));
                        found_online = true;
                    }
                }
                BridgeMessage::Progress(p) => {
                    assert!(p <= 100);
                    found_progress = true;
                }
                BridgeMessage::ScanComplete => {
                    complete = true;
                    break;
                }
                _ => {}
            }
        }

        assert!(found_online);
        assert!(found_progress);
        assert!(complete);
    }
}
