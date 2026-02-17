use crate::types::BridgeMessage;
use crossterm::event::{Event as CrosstermEvent, KeyEvent};
use futures::{FutureExt, StreamExt};
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Input(KeyEvent),
    Tick,
    Bridge(BridgeMessage),
}

pub struct EventHandler {
    pub rx: mpsc::UnboundedReceiver<AppEvent>,
}

impl EventHandler {
    pub fn new(bridge_rx: crossbeam_channel::Receiver<BridgeMessage>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let tx_clone = tx.clone();

        // 1. Crossterm events + Ticks
        tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick_interval = tokio::time::interval(Duration::from_millis(250));
            loop {
                let tick_delay = tick_interval.tick();
                let crossterm_event = reader.next().fuse();

                tokio::select! {
                    maybe_event = crossterm_event => {
                        if let Some(Ok(CrosstermEvent::Key(key))) = maybe_event
                            && key.kind == crossterm::event::KeyEventKind::Press {
                                let _ = tx.send(AppEvent::Input(key));
                        }
                    }
                    _ = tick_delay => {
                        let _ = tx.send(AppEvent::Tick);
                    }
                }
            }
        });

        // 2. Bridge events (poll crossbeam from tokio)
        tokio::spawn(async move {
            loop {
                // We use try_recv to avoid blocking the loop, but in a tokio task
                // it's better to yield if empty.
                while let Ok(msg) = bridge_rx.try_recv() {
                    let _ = tx_clone.send(AppEvent::Bridge(msg));
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });

        Self { rx }
    }
}
