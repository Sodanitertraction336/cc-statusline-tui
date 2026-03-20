use super::terminal::{self, Key};
use std::io::{Write, stdout};

// ── Result type ────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub enum ConfirmResult {
    Yes,
    No,
    Back,      // Left arrow
    Cancelled, // Esc or Ctrl+C
}

// ── ANSI helpers ───────────────────────────────────────────────────────────

const RESET: &str = "\x1b[0m";
const CYAN: &str = "\x1b[36m";
const DIM: &str = "\x1b[2m";
const CORNER_TL: &str = "┌";
const BAR: &str = "│";
const CORNER_BL: &str = "└";

// ── Drawing ────────────────────────────────────────────────────────────────

/// Render the confirm prompt into a byte buffer.
///
/// Layout:
/// ```text
///   ┌ Save configuration?
///   │  (Y/n)
///   └
/// ```
pub fn draw_confirm(message: &str, default: bool) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();

    let hint = if default { "Y/n" } else { "y/N" };

    let _ = write!(buf, "  {CYAN}{CORNER_TL}{RESET} {message}\r\n");
    let _ = write!(
        buf,
        "  {CYAN}{BAR}{RESET}  {DIM}({hint}){RESET}\r\n"
    );
    let _ = write!(buf, "  {CYAN}{CORNER_BL}{RESET}");

    buf
}

// ── Public API ─────────────────────────────────────────────────────────────

/// Simple y/n confirmation prompt.
///
/// Returns [`ConfirmResult`].
pub fn confirm(message: &str, default: bool) -> ConfirmResult {
    // 1. Draw the confirm UI
    let mut out = stdout();
    let rendered = draw_confirm(message, default);
    let _ = out.write_all(&rendered);
    let _ = out.flush();

    // 2. Enter raw mode + hide cursor
    terminal::enable_raw_mode();
    terminal::hide_cursor();

    // 3. Key loop
    let result = loop {
        match terminal::read_key() {
            Key::Char('y') | Key::Char('Y') => {
                break ConfirmResult::Yes;
            }
            Key::Char('n') | Key::Char('N') => {
                break ConfirmResult::No;
            }
            Key::Enter => {
                break if default {
                    ConfirmResult::Yes
                } else {
                    ConfirmResult::No
                };
            }
            Key::Left => {
                break ConfirmResult::Back;
            }
            Key::Escape | Key::CtrlC => {
                break ConfirmResult::Cancelled;
            }
            _ => {}
        }
    };

    // 4. Restore terminal state
    terminal::show_cursor();
    terminal::disable_raw_mode();

    // Final newline so the caller starts on a clean line
    let _ = writeln!(stdout());

    result
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirm_result_variants() {
        assert_eq!(ConfirmResult::Yes, ConfirmResult::Yes);
        assert_eq!(ConfirmResult::No, ConfirmResult::No);
        assert_eq!(ConfirmResult::Back, ConfirmResult::Back);
        assert_eq!(ConfirmResult::Cancelled, ConfirmResult::Cancelled);

        // All variants are distinct
        assert_ne!(ConfirmResult::Yes, ConfirmResult::No);
        assert_ne!(ConfirmResult::Yes, ConfirmResult::Back);
        assert_ne!(ConfirmResult::Yes, ConfirmResult::Cancelled);
        assert_ne!(ConfirmResult::No, ConfirmResult::Back);
        assert_ne!(ConfirmResult::No, ConfirmResult::Cancelled);
        assert_ne!(ConfirmResult::Back, ConfirmResult::Cancelled);
    }

    #[test]
    fn test_confirm_result_debug() {
        assert_eq!(format!("{:?}", ConfirmResult::Yes), "Yes");
        assert_eq!(format!("{:?}", ConfirmResult::No), "No");
        assert_eq!(format!("{:?}", ConfirmResult::Back), "Back");
        assert_eq!(format!("{:?}", ConfirmResult::Cancelled), "Cancelled");
    }

    #[test]
    fn test_draw_confirm_default_yes() {
        let buf = draw_confirm("Save config?", true);
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("Save config?"));
        assert!(output.contains("Y/n"));
        assert!(output.contains("┌"));
        assert!(output.contains("│"));
        assert!(output.contains("└"));
    }

    #[test]
    fn test_draw_confirm_default_no() {
        let buf = draw_confirm("Delete file?", false);
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("Delete file?"));
        assert!(output.contains("y/N"));
    }
}
