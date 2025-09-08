use crossterm::event::{self, Event, KeyEvent};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// Event types for the TUI application
#[derive(Debug, Clone)]
pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    Refresh,
    Quit,
}

/// Event handler for managing user input and periodic tasks
pub struct EventHandler {
    sender: mpsc::UnboundedSender<AppEvent>,
    receiver: mpsc::UnboundedReceiver<AppEvent>,
    handler: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::unbounded_channel();
        let _sender = sender.clone();

        let handler = tokio::spawn(async move {
            let mut last_tick = Instant::now();
            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if let Ok(has_event) = event::poll(timeout) {
                    if has_event {
                        if let Ok(Event::Key(key)) = event::read() {
                            if key.kind == event::KeyEventKind::Press {
                                let _ = _sender.send(AppEvent::Key(key));
                            }
                        }
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    let _ = _sender.send(AppEvent::Tick);
                    last_tick = Instant::now();
                }
            }
        });

        Self {
            sender,
            receiver,
            handler,
        }
    }

    pub async fn next(&mut self) -> Option<AppEvent> {
        self.receiver.recv().await
    }

    pub fn sender(&self) -> &mpsc::UnboundedSender<AppEvent> {
        &self.sender
    }
}

impl Drop for EventHandler {
    fn drop(&mut self) {
        self.handler.abort();
    }
}
