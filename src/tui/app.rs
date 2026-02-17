use crate::types::{BridgeMessage, ScanResult};
use ratatui::crossterm::event::KeyCode;
use ratatui::widgets::TableState;
use tokio::sync::mpsc::Sender;

#[derive(PartialEq, Eq, Debug)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(PartialEq, Eq, Debug)]
pub enum ScanState {
    Idle,
    Scanning,
    Complete,
    Cancelled,
}

pub struct App {
    pub input: String,
    pub input_mode: InputMode,
    pub results: Vec<ScanResult>,
    pub table_state: TableState,
    pub progress: u8,
    pub scan_state: ScanState,
    pub error: Option<String>,
    pub show_detail: bool,
    pub should_quit: bool,
    pub filter_online: bool,
    pub cmd_tx: Sender<BridgeMessage>,
}

impl App {
    pub fn new(cmd_tx: Sender<BridgeMessage>) -> Self {
        Self {
            input: String::from("192.168.1.1-255"),
            input_mode: InputMode::Normal,
            results: Vec::new(),
            table_state: TableState::default(),
            progress: 0,
            scan_state: ScanState::Idle,
            error: None,
            show_detail: false,
            should_quit: false,
            filter_online: false,
            cmd_tx,
        }
    }

    pub fn filtered_results(&self) -> Vec<&ScanResult> {
        if self.filter_online {
            self.results
                .iter()
                .filter(|r| r.status == crate::types::ScanStatus::Online)
                .collect()
        } else {
            self.results.iter().collect()
        }
    }

    pub fn start_scan(&mut self) {
        self.results.clear();
        self.progress = 0;
        self.scan_state = ScanState::Scanning;
        self.error = None;
        let _ = self
            .cmd_tx
            .try_send(BridgeMessage::StartScan(self.input.clone()));
    }

    pub fn stop_scan(&mut self) {
        let _ = self.cmd_tx.try_send(BridgeMessage::StopScan);
    }

    pub fn next_row(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.filtered_results().len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn previous_row(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_results().len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn sort_results(&mut self) {
        self.results.sort_by(|a, b| a.ip.cmp(&b.ip));
    }

    /// Processes a key press event and updates application state.
    ///
    /// Delegates to the current mode's handler:
    /// - **Editing**: character input, backspace, enter (start scan), escape.
    /// - **Detail view**: escape/q to close popup.
    /// - **Normal**: quit, edit mode, stop scan, navigation, detail view, filter.
    ///
    /// # Parameters
    /// - `code`: The `KeyCode` of the pressed key.
    pub fn on_key(&mut self, code: KeyCode) {
        if self.input_mode == InputMode::Editing {
            match code {
                KeyCode::Enter => {
                    self.input_mode = InputMode::Normal;
                    self.start_scan();
                }
                KeyCode::Char(c) => self.input.push(c),
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Esc => self.input_mode = InputMode::Normal,
                _ => {}
            }
        } else if self.show_detail {
            if code == KeyCode::Esc || code == KeyCode::Char('q') {
                self.show_detail = false;
            }
        } else {
            match code {
                KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                KeyCode::Char('i') | KeyCode::Char('e') => self.input_mode = InputMode::Editing,
                KeyCode::Char('s') => self.stop_scan(),
                KeyCode::Char('j') | KeyCode::Down => self.next_row(),
                KeyCode::Char('k') | KeyCode::Up => self.previous_row(),
                KeyCode::Enter => self.show_detail = true,
                KeyCode::Tab => self.filter_online = !self.filter_online,
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::KeyCode;

    fn test_app() -> App {
        let (tx, _rx) = tokio::sync::mpsc::channel(8);
        App::new(tx)
    }

    #[test]
    fn test_char_input_appends() {
        let mut app = test_app();
        app.input_mode = InputMode::Editing;
        app.input.clear();
        app.on_key(KeyCode::Char('A'));
        app.on_key(KeyCode::Char('B'));
        assert_eq!(app.input, "AB");
    }

    #[test]
    fn test_backspace_removes_last() {
        let mut app = test_app();
        app.input_mode = InputMode::Editing;
        app.input = "ABC".into();
        app.on_key(KeyCode::Backspace);
        assert_eq!(app.input, "AB");
    }

    #[test]
    fn test_backspace_on_empty_is_noop() {
        let mut app = test_app();
        app.input_mode = InputMode::Editing;
        app.input.clear();
        app.on_key(KeyCode::Backspace);
        assert_eq!(app.input, "");
    }

    #[test]
    fn test_enter_starts_scan() {
        let mut app = test_app();
        app.input_mode = InputMode::Editing;
        app.on_key(KeyCode::Enter);
        assert_eq!(app.input_mode, InputMode::Normal);
        assert_eq!(app.scan_state, ScanState::Scanning);
    }

    #[test]
    fn test_esc_exits_editing() {
        let mut app = test_app();
        app.input_mode = InputMode::Editing;
        app.on_key(KeyCode::Esc);
        assert_eq!(app.input_mode, InputMode::Normal);
        assert!(!app.should_quit);
    }

    #[test]
    fn test_q_quits_in_normal_mode() {
        let mut app = test_app();
        app.on_key(KeyCode::Char('q'));
        assert!(app.should_quit);
    }

    #[test]
    fn test_esc_closes_detail_popup() {
        let mut app = test_app();
        app.show_detail = true;
        app.on_key(KeyCode::Esc);
        assert!(!app.show_detail);
        assert!(!app.should_quit);
    }

    #[test]
    fn test_tab_toggles_filter() {
        let mut app = test_app();
        assert!(!app.filter_online);
        app.on_key(KeyCode::Tab);
        assert!(app.filter_online);
        app.on_key(KeyCode::Tab);
        assert!(!app.filter_online);
    }
}
