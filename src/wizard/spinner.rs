use std::io::{Write, stdout};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// ── Constants ──────────────────────────────────────────────────────────────

/// Braille spinner frames.
pub const FRAMES: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

/// Frame interval in milliseconds.
const FRAME_INTERVAL_MS: u64 = 80;

// ── ANSI helpers ───────────────────────────────────────────────────────────

const RESET: &str = "\x1b[0m";
const CYAN: &str = "\x1b[36m";

// ── Spinner ────────────────────────────────────────────────────────────────

pub struct Spinner {
    handle: Option<thread::JoinHandle<()>>,
    running: Arc<AtomicBool>,
}

impl Spinner {
    /// Start a spinner with a message.
    ///
    /// Spawns a background thread that cycles through braille frames.
    /// Display format: `  ⠋ {message}`
    pub fn start(message: &str) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        let message = message.to_string();

        let handle = thread::spawn(move || {
            let mut frame_idx = 0;
            let mut out = stdout();

            while running_clone.load(Ordering::Relaxed) {
                let frame = FRAMES[frame_idx % FRAMES.len()];
                let _ = write!(out, "\r  {CYAN}{frame}{RESET} {message}");
                let _ = out.flush();
                frame_idx += 1;
                thread::sleep(Duration::from_millis(FRAME_INTERVAL_MS));
            }
        });

        Spinner {
            handle: Some(handle),
            running,
        }
    }

    /// Stop the spinner and display a final message.
    ///
    /// Clears the spinner line and prints the final message.
    pub fn stop(mut self, message: &str) {
        // Signal the thread to stop
        self.running.store(false, Ordering::Relaxed);

        // Wait for the thread to finish
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }

        // Clear the spinner line and print final message
        let mut out = stdout();
        let _ = write!(out, "\r\x1b[K  {message}\r\n");
        let _ = out.flush();
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_frames() {
        assert_eq!(FRAMES.len(), 10);
        assert_eq!(FRAMES[0], '⠋');
        assert_eq!(FRAMES[1], '⠙');
        assert_eq!(FRAMES[2], '⠹');
        assert_eq!(FRAMES[3], '⠸');
        assert_eq!(FRAMES[4], '⠼');
        assert_eq!(FRAMES[5], '⠴');
        assert_eq!(FRAMES[6], '⠦');
        assert_eq!(FRAMES[7], '⠧');
        assert_eq!(FRAMES[8], '⠇');
        assert_eq!(FRAMES[9], '⠏');
    }

    #[test]
    fn test_spinner_create_stop() {
        // Create a spinner and immediately stop it — should not panic
        let spinner = Spinner::start("Loading...");
        // Small delay to let the spinner thread start
        thread::sleep(Duration::from_millis(100));
        spinner.stop("Done!");
    }
}
