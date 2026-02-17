use ragescanner::bridge::Bridge;
use ragescanner::tui::app::{App, ScanState};
use ragescanner::tui::event::{AppEvent, EventHandler};
use ragescanner::tui::ui;
use ragescanner::types::BridgeMessage;

use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
};
use std::io;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. Bridge & App setup
    let bridge = Bridge::new();
    let mut app = App::new(bridge.cmd_tx.clone());
    let mut events = EventHandler::new(bridge.ui_rx.clone());

    // 3. Main Loop
    loop {
        terminal.draw(|f| ui::render(f, &mut app))?;

        if let Some(event) = events.rx.recv().await {
            match event {
                AppEvent::Input(key) => {
                    app.on_key(key.code);
                }
                AppEvent::Tick => {}
                AppEvent::Bridge(msg) => {
                    match msg {
                        BridgeMessage::ScanUpdate(res) => {
                            // Update or add result
                            if let Some(existing) = app.results.iter_mut().find(|r| r.ip == res.ip)
                            {
                                *existing = res;
                            } else {
                                app.results.push(res);
                            }
                        }
                        BridgeMessage::Progress(p) => app.progress = p,
                        BridgeMessage::ScanComplete => {
                            app.scan_state = ScanState::Complete;
                            app.progress = 100;
                            app.sort_results();
                        }
                        BridgeMessage::ScanCancelled => app.scan_state = ScanState::Cancelled,
                        BridgeMessage::Error(e) => {
                            app.scan_state = ScanState::Idle;
                            app.error = Some(e.to_string());
                        }
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    // 4. Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
