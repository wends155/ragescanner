#![cfg_attr(not(test), windows_subsystem = "windows")]

mod ui;

use log::LevelFilter;
use ragescanner::bridge::Bridge;
use simplelog::{Config, WriteLogger};
use std::fs::File;
use std::panic;
use windows::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxA};

fn main() {
    // 1. Initialize Logging
    let log_level = if cfg!(debug_assertions) {
        LevelFilter::Debug
    } else {
        LevelFilter::Error
    };

    let _ = WriteLogger::init(
        log_level,
        Config::default(),
        File::create("ragescanner.log")
            .unwrap_or_else(|_| File::create("ragescanner.err").unwrap()),
    );

    log::info!("Application Started");

    // 2. Set Global Panic Hook
    panic::set_hook(Box::new(|info| {
        let msg = match info.payload().downcast_ref::<&str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Unknown panic",
            },
        };

        let location = info
            .location()
            .map(|l| format!(" at {}:{}", l.file(), l.line()))
            .unwrap_or_default();
        let err_msg = format!("Application Panicked:\n{}{}", msg, location);

        log::error!("{}", err_msg);

        unsafe {
            let title = b"RageScanner Crash\0";
            let body = format!("{}\0", err_msg);
            MessageBoxA(
                None,
                windows::core::PCSTR(body.as_ptr()),
                windows::core::PCSTR(title.as_ptr()),
                MB_OK | MB_ICONERROR,
            );
        }
    }));

    let bridge = Bridge::new();
    ui::run_app(bridge.cmd_tx, bridge.ui_rx);
}
