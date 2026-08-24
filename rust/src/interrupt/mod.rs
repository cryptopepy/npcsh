#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

pub fn spawn_listener(
    interrupt_tx: tokio::sync::mpsc::UnboundedSender<()>,
    queue_tx: tokio::sync::mpsc::UnboundedSender<String>,
    running: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> tokio::task::JoinHandle<()> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "linux")] {
            linux::spawn_listener(interrupt_tx, queue_tx, running)
        } else if #[cfg(target_os = "macos")] {
            macos::spawn_listener(interrupt_tx, queue_tx, running)
        } else if #[cfg(target_os = "windows")] {
            windows::spawn_listener(interrupt_tx, queue_tx, running)
        } else {
            // Fallback for other Unix-likes: macOS-style crossterm listener.
            macos::spawn_listener(interrupt_tx, queue_tx, running)
        }
    }
}
