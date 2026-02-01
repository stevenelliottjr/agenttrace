//! Event handling for the TUI

use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;
use tokio::sync::mpsc;

/// TUI events
#[derive(Debug, Clone)]
pub enum Event {
    /// Terminal tick (for animations/updates)
    Tick,
    /// Keyboard event
    Key(KeyEvent),
    /// Mouse event
    Mouse(crossterm::event::MouseEvent),
    /// Terminal resize
    Resize(u16, u16),
    /// New span received
    SpanReceived(String),
    /// Metrics updated
    MetricsUpdated,
    /// Error occurred
    Error(String),
}

/// Handles events from terminal and other sources
pub struct EventHandler {
    /// Sender for events
    tx: mpsc::UnboundedSender<Event>,
    /// Receiver for events
    rx: mpsc::UnboundedReceiver<Event>,
    /// Tick rate
    tick_rate: Duration,
}

impl EventHandler {
    /// Create a new event handler
    pub fn new(tick_rate_ms: u64) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            tx,
            rx,
            tick_rate: Duration::from_millis(tick_rate_ms),
        }
    }

    /// Get a sender to inject events
    pub fn sender(&self) -> mpsc::UnboundedSender<Event> {
        self.tx.clone()
    }

    /// Start the event loop
    pub fn start(&self) {
        let tick_rate = self.tick_rate;
        let tx = self.tx.clone();

        tokio::spawn(async move {
            let mut last_tick = std::time::Instant::now();

            loop {
                // Poll for crossterm events with timeout
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or(Duration::ZERO);

                if event::poll(timeout).unwrap_or(false) {
                    match event::read() {
                        Ok(CrosstermEvent::Key(key)) => {
                            if tx.send(Event::Key(key)).is_err() {
                                break;
                            }
                        }
                        Ok(CrosstermEvent::Mouse(mouse)) => {
                            if tx.send(Event::Mouse(mouse)).is_err() {
                                break;
                            }
                        }
                        Ok(CrosstermEvent::Resize(w, h)) => {
                            if tx.send(Event::Resize(w, h)).is_err() {
                                break;
                            }
                        }
                        _ => {}
                    }
                }

                // Send tick if enough time has passed
                if last_tick.elapsed() >= tick_rate {
                    if tx.send(Event::Tick).is_err() {
                        break;
                    }
                    last_tick = std::time::Instant::now();
                }
            }
        });
    }

    /// Get the next event
    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }
}

/// Check if a key event matches a key binding
pub fn key_match(key: KeyEvent, code: KeyCode, modifiers: KeyModifiers) -> bool {
    key.code == code && key.modifiers == modifiers
}

/// Check if key is quit command (q or Ctrl+C)
pub fn is_quit(key: KeyEvent) -> bool {
    key_match(key, KeyCode::Char('q'), KeyModifiers::NONE)
        || key_match(key, KeyCode::Char('c'), KeyModifiers::CONTROL)
}
