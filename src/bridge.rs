//! UI↔Scanner bridge orchestrator.
//!
//! [`Bridge`] spawns a background thread with a Tokio runtime and provides
//! channel-based communication for any frontend (GUI, TUI, CLI).

use crate::net::NetUtils;
use crate::scanner::Scanner;
use crate::types::{BridgeMessage, GError};
use crossbeam_channel::{unbounded, Receiver};
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{channel as tokio_channel, Sender as TokioSender};

/// Orchestrator that bridges a frontend to the async scanner.
///
/// Spawns a background thread with a Tokio runtime. Commands are sent via
/// [`cmd_tx`](Bridge::cmd_tx) and results received via [`ui_rx`](Bridge::ui_rx).
pub struct Bridge {
    /// Receiver for messages directed to the UI.
    pub ui_rx: Receiver<BridgeMessage>,
    /// Sender for commands directed to the scanner.
    pub cmd_tx: TokioSender<BridgeMessage>,
}

impl Default for Bridge {
    fn default() -> Self {
        Self::new()
    }
}

impl Bridge {
    /// Creates a new bridge, spawning the background scanner thread.
    pub fn new() -> Self {
        let (ui_tx, ui_rx) = unbounded::<BridgeMessage>();
        let (cmd_tx, mut cmd_rx) = tokio_channel::<BridgeMessage>(32);

        thread::spawn(move || {
            let rt = match Runtime::new() {
                Ok(r) => r,
                Err(e) => {
                    let _ = ui_tx.send(BridgeMessage::Error(GError::Internal(format!(
                        "Failed to create tokio runtime: {}",
                        e
                    ))));
                    return;
                }
            };

            rt.block_on(async move {
                let (scanner_tx, mut scanner_rx) = tokio_channel::<BridgeMessage>(100);

                let ui_tx_clone = ui_tx.clone();
                tokio::spawn(async move {
                    while let Some(msg) = scanner_rx.recv().await {
                        let _ = ui_tx_clone.send(msg);
                    }
                });

                // Instantiate real NetUtils and inject as NetworkProvider trait object
                let net_utils = Arc::new(NetUtils::new());
                let scanner = Arc::new(Scanner::new(net_utils, scanner_tx));

                while let Some(msg) = cmd_rx.recv().await {
                    if let BridgeMessage::StartScan(range) = msg {
                        match Self::parse_range(&range) {
                            Ok((start, end)) => {
                                let scanner_clone = scanner.clone();
                                tokio::spawn(async move {
                                    scanner_clone.scan_range(start, end).await;
                                });
                            }
                            Err(e) => {
                                let _ = ui_tx.send(BridgeMessage::Error(GError::Internal(e)));
                            }
                        }
                    }
                }
            });
        });

        Self { ui_rx, cmd_tx }
    }

    /// Parses an IP range string.
    /// Supported: "192.168.1.1", "192.168.1.1-255", "192.168.1.1-192.168.1.50"
    pub fn parse_range(range: &str) -> Result<(Ipv4Addr, Ipv4Addr), String> {
        let parts: Vec<&str> = range.split('-').collect();
        if parts.is_empty() {
            return Err("Empty range".to_string());
        }

        let start_str = parts[0].trim();
        let start = Ipv4Addr::from_str(start_str)
            .map_err(|_| format!("Invalid Start IP: '{}'", start_str))?;

        if parts.len() == 1 {
            return Ok((start, start));
        }

        let end_part = parts[1].trim();
        if let Ok(end) = Ipv4Addr::from_str(end_part) {
            if end < start {
                Err(format!(
                    "End IP ({}) cannot be less than Start IP ({})",
                    end, start
                ))
            } else {
                Ok((start, end))
            }
        } else if let Ok(last_octet) = end_part.parse::<u8>() {
            let octets = start.octets();
            let end = Ipv4Addr::new(octets[0], octets[1], octets[2], last_octet);
            if end < start {
                Err(format!(
                    "End IP ({}) cannot be less than Start IP ({})",
                    end, start
                ))
            } else {
                Ok((start, end))
            }
        } else {
            Err(format!("Invalid End IP or Octet: '{}'", end_part))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_parse_single_ip() {
        let res = Bridge::parse_range("192.168.1.1");
        assert_eq!(
            res,
            Ok((Ipv4Addr::new(192, 168, 1, 1), Ipv4Addr::new(192, 168, 1, 1)))
        );
    }

    #[test]
    fn test_parse_octet_range() {
        let res = Bridge::parse_range("192.168.1.1-50");
        assert_eq!(
            res,
            Ok((
                Ipv4Addr::new(192, 168, 1, 1),
                Ipv4Addr::new(192, 168, 1, 50)
            ))
        );
    }

    #[test]
    fn test_parse_full_range() {
        let res = Bridge::parse_range("10.0.0.1 - 10.0.0.255");
        assert_eq!(
            res,
            Ok((Ipv4Addr::new(10, 0, 0, 1), Ipv4Addr::new(10, 0, 0, 255)))
        );
    }

    #[test]
    fn test_parse_invalid() {
        assert!(Bridge::parse_range("not-an-ip").is_err());
        assert!(Bridge::parse_range("192.168.1.1-abc").is_err());
        assert!(Bridge::parse_range("").is_err());
        assert!(Bridge::parse_range("192.168.1.10-5").is_err()); // End < Start
    }

    #[test]
    fn test_parse_ui_generated_range_randomized() {
        let mut rng = rand::thread_rng();

        for _ in 0..100 {
            // 1. Generate Random Start IP
            let o1 = rng.gen_range(1..224);
            let o2 = rng.gen_range(0..255);
            let o3 = rng.gen_range(0..255);
            let o4 = rng.gen_range(0..200); // Leave room for end range
            let start = Ipv4Addr::new(o1, o2, o3, o4);

            // 2. Generate Random End IP (Octet or Full)
            let use_full_ip = rng.gen_bool(0.5);
            let end_val = rng.gen_range(o4..255); // Ensure >= start

            let input_str = if use_full_ip {
                let end = Ipv4Addr::new(o1, o2, o3, end_val);
                format!("{}-{}", start, end)
            } else {
                format!("{}-{}", start, end_val)
            };

            // 3. Test Parsing
            let res = Bridge::parse_range(&input_str);
            assert!(
                res.is_ok(),
                "Failed to parse generated input: {}",
                input_str
            );
            let (s, e) = res.unwrap();
            assert_eq!(s, start);
            assert!(e >= s);
        }

        // Test Garbage
        for _ in 0..20 {
            let garbage: String = (0..10).map(|_| rng.gen_range(b'a'..b'z') as char).collect();
            assert!(Bridge::parse_range(&garbage).is_err());
        }
    }
}
