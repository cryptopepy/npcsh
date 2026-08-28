use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// macOS interrupt listener.
///
/// On macOS, crossterm reliably observes Ctrl+C before the default SIGINT
/// handler kills the process, so the signal fallback is not needed. Esc and
/// Ctrl+C both send an interrupt signal; Enter queues a line of input.
pub fn spawn_listener(
    interrupt_tx: tokio::sync::mpsc::UnboundedSender<()>,
    queue_tx: tokio::sync::mpsc::UnboundedSender<String>,
    running: Arc<AtomicBool>,
) -> tokio::task::JoinHandle<()> {
    tokio::task::spawn_blocking(move || {
        let mut buf = String::new();
        while running.load(Ordering::Relaxed) {
            if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                if let Ok(Event::Key(key)) = event::read() {
                    if key.kind == crossterm::event::KeyEventKind::Release {
                        continue;
                    }
                    match key.code {
                        KeyCode::Esc => {
                            let _ = interrupt_tx.send(());
                        }
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            let _ = interrupt_tx.send(());
                        }
                        KeyCode::Enter => {
                            let line = std::mem::take(&mut buf);
                            if !line.is_empty() {
                                let _ = queue_tx.send(line);
                            }
                        }
                        KeyCode::Char(c) => {
                            buf.push(c);
                        }
                        KeyCode::Backspace => {
                            buf.pop();
                        }
                        _ => {}
                    }
                }
            }
        }
    })
}
