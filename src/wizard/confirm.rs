//! Y/N confirmation prompt component.
//!
//! Displays a question with a default answer hint (Y/n or y/N). Accepts
//! y/Y, n/N, Enter (uses default), Left (back), and Esc/Ctrl+C (cancel).
//!
//! Used at the end of the wizard for "Save and apply?" and for the
//! "Use defaults?" shortcut flow.
//!
//! Key function: `confirm(message, default, footer) -> ConfirmResult`

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
const BRIGHT_CYAN: &str = "\x1b[1;36m";
const DIM: &str = "\x1b[2m";
const CORNER_TL: &str = "┌";
const BAR: &str = "│";
const CORNER_BL: &str = "└";

// ── Drawing ────────────────────────────────────────────────────────────────

/// Render the confirm prompt into a byte buffer.
///
/// - `footer=None` → classic style (┌/└)
/// - `footer=Some(f)` → step style (◆ header, footer replaces └)
pub fn draw_confirm(message: &str, default: bool, footer: Option<&str>) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();

    let hint = if default { "Y/n" } else { "y/N" };

    // Header
    if footer.is_some() {
        let _ = write!(buf, "  {BRIGHT_CYAN}\u{25C6}{RESET} {message}\r\n");
    } else {
        let _ = write!(buf, "  {CYAN}{CORNER_TL}{RESET} {message}\r\n");
    }
    let _ = write!(
        buf,
        "  {CYAN}{BAR}{RESET}  {DIM}({hint}){RESET}\r\n"
    );

    // Footer
    match footer {
        None => {
            let _ = write!(buf, "  {CYAN}{CORNER_BL}{RESET}");
        }
        Some(f) if !f.is_empty() => {
            let _ = write!(buf, "{f}");
        }
        Some(_) => {
            // Step mode, last step (empty footer): no └
        }
    }

    buf
}

// ── Public API ─────────────────────────────────────────────────────────────

/// Simple y/n confirmation prompt.
///
/// `footer` controls step-integrated mode (see `draw_confirm`).
/// Returns [`ConfirmResult`].
pub fn confirm(message: &str, default: bool, footer: Option<&str>) -> ConfirmResult {
    // 1. Draw the confirm UI
    let mut out = stdout();
    let rendered = draw_confirm(message, default, footer);
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
        let buf = draw_confirm("Save config?", true, None);
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("Save config?"));
        assert!(output.contains("Y/n"));
        assert!(output.contains("┌"));
        assert!(output.contains("│"));
        assert!(output.contains("└"));
    }

    #[test]
    fn test_draw_confirm_default_no() {
        let buf = draw_confirm("Delete file?", false, None);
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("Delete file?"));
        assert!(output.contains("y/N"));
    }

    #[test]
    fn test_draw_confirm_step_mode() {
        // Step mode with footer
        let footer = "\r\n  \x1b[2m│\x1b[0m\r\n  \x1b[2m○ 4/4 Confirm\x1b[0m";
        let buf = draw_confirm("Save?", true, Some(footer));
        let output = String::from_utf8(buf).unwrap();

        // Uses ◆ instead of ┌
        assert!(output.contains("\u{25C6}"));
        assert!(!output.contains("┌"));
        assert!(!output.contains("└"));

        // Footer appended
        assert!(output.contains("4/4 Confirm"));
    }

    #[test]
    fn test_draw_confirm_step_mode_empty_footer() {
        // Step mode, last step (empty footer)
        let buf = draw_confirm("Save?", true, Some(""));
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("\u{25C6}"));
        assert!(!output.contains("┌"));
        assert!(!output.contains("└"));
    }
}
