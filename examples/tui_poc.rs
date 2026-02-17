use ragescanner::bridge::Bridge;
use ragescanner::types::BridgeMessage;
use std::io::{self, Write};

fn main() {
    println!("--- RageScanner TUI PoC ---");
    
    let bridge = Bridge::new();
    
    // Start scan for a small local range using the typed API
    let start = std::net::Ipv4Addr::new(127, 0, 0, 1);
    let end = std::net::Ipv4Addr::new(127, 0, 0, 5);
    println!("Scanning range: {}-{}", start, end);
    
    if let Err(e) = bridge.cmd_tx.blocking_send(BridgeMessage::StartScanRange(start, end)) {
        eprintln!("Failed to start scan: {}", e);
        return;
    }

    while let Ok(msg) = bridge.ui_rx.recv() {
        match msg {
            BridgeMessage::ScanUpdate(res) => {
                println!("[{}] IP: {} | Status: {} | Host: {:?} | MAC: {:?} | Vendor: {:?} | Ports: {:?}", 
                    res.ip, res.ip, res.status, res.hostname, res.mac, res.vendor, res.open_ports);
            }
            BridgeMessage::Progress(p) => {
                print!("\rProgress: {}%   ", p);
                io::stdout().flush().unwrap();
            }
            BridgeMessage::ScanComplete => {
                println!("\nScan Complete!");
                break;
            }
            BridgeMessage::ScanCancelled => {
                println!("\nScan Cancelled!");
                break;
            }
            BridgeMessage::Error(e) => {
                eprintln!("\nError: {}", e);
                break;
            }
            _ => {}
        }
    }
}
