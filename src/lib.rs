//! # RageScanner
//!
//! A high-performance, asynchronous IP and port scanner library for Windows.
//!
//! Provides ICMP ping, ARP-based MAC resolution, OUI vendor lookup,
//! reverse DNS, and TCP port scanning — all orchestrated via the
//! [`bridge::Bridge`] struct.
//!
//! # Example
//!
//! ```no_run
//! use ragescanner::bridge::Bridge;
//! use ragescanner::types::BridgeMessage;
//!
//! let bridge = Bridge::new();
//! bridge.cmd_tx.blocking_send(BridgeMessage::StartScan("192.168.1.1-255".into())).unwrap();
//! while let Ok(msg) = bridge.ui_rx.recv() {
//!     match msg {
//!         BridgeMessage::ScanUpdate(r) => println!("{}: {}", r.ip, r.status),
//!         BridgeMessage::ScanComplete => break,
//!         _ => {}
//!     }
//! }
//! ```

pub mod bridge;
pub mod net;
pub mod scanner;
pub mod types;
