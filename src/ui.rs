use crate::types::{BridgeMessage, ScanResult};
use log::error;
use native_windows_derive::NwgUi;
use native_windows_gui as nwg;
use nwg::NativeUi;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::Sender as TokioSender;

#[derive(Default, NwgUi)]
pub struct RageScannerApp {
    #[nwg_resource(family: "Segoe UI", size: 16)]
    font: nwg::Font,

    #[nwg_control(size: (700, 500), position: (300, 300), title: "RageScanner - Windows IP Scanner", flags: "WINDOW|VISIBLE")]
    #[nwg_events( OnWindowClose: [RageScannerApp::exit] )]
    window: nwg::Window,

    #[nwg_layout(parent: window, spacing: 3)]
    layout: nwg::GridLayout,

    // Row 0: Start IP
    #[nwg_control(text: "Start IP:", h_align: nwg::HTextAlign::Right)]
    #[nwg_layout_item(layout: layout, col: 0, row: 0, row_span: 2)]
    label_start: nwg::Label,

    #[nwg_control(text: "192.168.1.1")]
    #[nwg_layout_item(layout: layout, col: 1, row: 0, row_span: 2)]
    start_ip_input: nwg::TextInput,

    // Row 0: End IP
    #[nwg_control(text: "End IP:", h_align: nwg::HTextAlign::Right)]
    #[nwg_layout_item(layout: layout, col: 2, row: 0, row_span: 2)]
    label_end: nwg::Label,

    #[nwg_control(text: "255")]
    #[nwg_layout_item(layout: layout, col: 3, row: 0, row_span: 2)]
    end_ip_input: nwg::TextInput,

    #[nwg_control(text: "Scan")]
    #[nwg_layout_item(layout: layout, col: 4, row: 0, row_span: 2)]
    #[nwg_events( OnButtonClick: [RageScannerApp::start_scan] )]
    scan_btn: nwg::Button,

    #[nwg_control(list_style: nwg::ListViewStyle::Detailed)]
    #[nwg_layout_item(layout: layout, col: 0, row: 2, col_span: 5, row_span: 16)]
    list_view: nwg::ListView,

    #[nwg_control(range: 0..100, pos: 0)]
    #[nwg_layout_item(layout: layout, col: 0, row: 18, col_span: 5)]
    progress_bar: nwg::ProgressBar,

    #[nwg_control(text: "Ready")]
    #[nwg_layout_item(layout: layout, col: 0, row: 19, col_span: 5)]
    status_bar: nwg::StatusBar,

    #[nwg_control]
    #[nwg_events(OnNotice: [RageScannerApp::handle_ui_message])]
    ui_notice: nwg::Notice,

    #[nwg_control]
    #[nwg_events(OnNotice: [RageScannerApp::clear_results])]
    clear_notice: nwg::Notice,

    // App State
    cmd_tx: Option<TokioSender<BridgeMessage>>,
    ui_rx: Option<Arc<crossbeam_channel::Receiver<BridgeMessage>>>,
    scan_in_progress: Arc<AtomicBool>,
    scan_results: RefCell<Vec<ScanResult>>,
}

impl RageScannerApp {
    fn init_list_view(&self) {
        self.list_view.insert_column("Status");
        self.list_view.insert_column("Hostname");
        self.list_view.insert_column("IP Address");
        self.list_view.insert_column("MAC Address");
        self.list_view.insert_column("Vendor");
        self.list_view.insert_column("Open Ports");

        self.list_view.set_headers_enabled(true);
        self.list_view.set_column_width(0, 80);
        self.list_view.set_column_width(1, 120);
        self.list_view.set_column_width(2, 100);
        self.list_view.set_column_width(3, 120);
        self.list_view.set_column_width(4, 120);
        self.list_view.set_column_width(5, 120);
    }

    fn start_scan(&self) {
        if self.scan_in_progress.load(Ordering::SeqCst) {
            return;
        }

        let start = self.start_ip_input.text();
        let end = self.end_ip_input.text();

        if start.is_empty() || end.is_empty() {
            nwg::modal_error_message(
                &self.window,
                "Error",
                "Please enter both Start and End IP/Octet.",
            );
            return;
        }

        let range = format!("{}-{}", start, end);

        // Clear previous results buffer
        self.scan_results.borrow_mut().clear();

        self.clear_notice.sender().notice();

        if let Some(tx) = &self.cmd_tx {
            let tx = tx.clone();
            self.scan_in_progress.store(true, Ordering::SeqCst);
            self.scan_btn.set_enabled(false);
            self.progress_bar.set_pos(0);
            self.status_bar.set_text(0, "Scanning...");

            // Use blocking_send to bridge sync -> async safely.
            // We handle the error by logging it, ensuring the app doesn't panic if the channel is closed.
            if let Err(e) = tx.blocking_send(BridgeMessage::StartScan(range)) {
                error!("Failed to send StartScan command: {}", e);
                nwg::modal_error_message(
                    &self.window,
                    "Internal Error",
                    &format!("Failed to start scan: {}", e),
                );
            }
        }
    }

    fn clear_results(&self) {
        self.list_view.clear();
    }

    fn handle_ui_message(&self) {
        if let Some(rx) = &self.ui_rx {
            let mut count = 0;
            // Process max 50 messages per tick to keep UI responsive
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    BridgeMessage::ScanUpdate(res) => {
                        // Buffer the result
                        self.scan_results.borrow_mut().push(res.clone());
                        // Update UI immediately (streaming view)
                        self.update_list(res);
                    }
                    BridgeMessage::ScanComplete => {
                        self.scan_in_progress.store(false, Ordering::SeqCst);
                        self.scan_btn.set_enabled(true);
                        self.status_bar.set_text(0, "Scan Complete - Sorting...");

                        // Sort results by IP
                        let mut results = self.scan_results.borrow_mut();
                        results.sort_by_key(|r| r.ip);

                        // Refresh List View
                        self.list_view.clear();
                        for res in results.iter() {
                            self.update_list(res.clone());
                        }

                        self.status_bar.set_text(0, "Scan Complete");
                        self.progress_bar.set_pos(100);
                    }
                    BridgeMessage::Progress(p) => {
                        self.progress_bar.set_pos(p as u32);
                    }
                    BridgeMessage::Error(e) => {
                        self.scan_in_progress.store(false, Ordering::SeqCst);
                        self.scan_btn.set_enabled(true);
                        self.status_bar.set_text(0, &format!("Error: {}", e));
                        nwg::modal_error_message(&self.window, "Scan Error", &e.to_string());
                    }
                    _ => {}
                }

                count += 1;
                if count >= 50 {
                    // Yield to let UI process window messages, then re-trigger
                    self.ui_notice.sender().notice();
                    break;
                }
            }
        }
    }

    fn update_list(&self, res: ScanResult) {
        let index = self.list_view.len();
        self.list_view.insert_item(nwg::InsertListViewItem {
            index: Some(index as i32),
            column_index: 0,
            text: Some(res.status.to_string()),
            image: None,
        });

        self.list_view.update_item(
            index,
            nwg::InsertListViewItem {
                index: Some(index as i32),
                column_index: 1,
                text: Some(res.hostname.unwrap_or_default()),
                image: None,
            },
        );

        self.list_view.update_item(
            index,
            nwg::InsertListViewItem {
                index: Some(index as i32),
                column_index: 2,
                text: Some(res.ip.to_string()),
                image: None,
            },
        );

        self.list_view.update_item(
            index,
            nwg::InsertListViewItem {
                index: Some(index as i32),
                column_index: 3,
                text: Some(res.mac.unwrap_or_default()),
                image: None,
            },
        );

        self.list_view.update_item(
            index,
            nwg::InsertListViewItem {
                index: Some(index as i32),
                column_index: 4,
                text: Some(res.vendor.unwrap_or_default()),
                image: None,
            },
        );

        let ports_str = res
            .open_ports
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        self.list_view.update_item(
            index,
            nwg::InsertListViewItem {
                index: Some(index as i32),
                column_index: 5,
                text: Some(ports_str),
                image: None,
            },
        );
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
}

pub fn run_app(
    cmd_tx: TokioSender<BridgeMessage>,
    ui_rx: crossbeam_channel::Receiver<BridgeMessage>,
) {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let app = RageScannerApp::build_ui(RageScannerApp {
        cmd_tx: Some(cmd_tx),
        ui_rx: Some(Arc::new(ui_rx)),
        scan_in_progress: Arc::new(AtomicBool::new(false)),
        ..Default::default()
    })
    .expect("Failed to build UI");

    app.init_list_view();

    let ui_notice = app.ui_notice.sender();
    let rx = app.ui_rx.as_ref().unwrap().clone();

    std::thread::spawn(move || loop {
        if !rx.is_empty() {
            ui_notice.notice();
            // Rate-limit the pump to prevent flooding the Windows Message Queue.
            // This allows the channel to buffer messages naturally.
            std::thread::sleep(std::time::Duration::from_millis(50));
        } else {
            // Idle poll to check for new messages
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    nwg::dispatch_thread_events();
}
