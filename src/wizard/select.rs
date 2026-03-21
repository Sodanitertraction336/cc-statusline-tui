//! Single-select prompt component with live preview callback.
//!
//! Renders a vertical list of options with Up/Down navigation, Enter/Right
//! to confirm, Left to go back, Esc/Ctrl+C to cancel. Supports two visual
//! styles: classic box (with corner chars) and step-integrated (with pending
//! footer from `step_progress`).
//!
//! The `on_change` callback fires on every arrow key, enabling live preview
//! updates in the wizard.
//!
//! Key types: `SelectOption`, `SelectResult`
//! Key function: `select(message, options, initial, on_change, footer)`

use super::terminal::{self, Key};
use std::io::{Write, stdout};

// ── Result & Option types ──────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub enum SelectResult {
    Selected(String), // Enter or Right arrow
    Back,             // Left arrow
    Cancelled,        // Esc or Ctrl+C
}

pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub hint: Option<String>,
}

// ── ANSI helpers ───────────────────────────────────────────────────────────

const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const CYAN: &str = "\x1b[36m";
const BRIGHT_CYAN: &str = "\x1b[1;36m";
const BAR: &str = "│";
const CORNER_TL: &str = "┌";
const CORNER_BL: &str = "└";

// ── Drawing ────────────────────────────────────────────────────────────────

/// Render the select UI into a byte buffer.
///
/// - `footer=None` → classic style (┌/└)
/// - `footer=Some(f)` → step style (◆ header, footer replaces └)
///
/// Returns `(rendered_bytes, lines_up)` where `lines_up` is the number of
/// `\r\n` sequences in the output (used by redraw to move cursor back up).
pub fn draw_select(
    message: &str,
    options: &[SelectOption],
    selected: usize,
    footer: Option<&str>,
) -> (Vec<u8>, u16) {
    let mut buf: Vec<u8> = Vec::new();
    let mut line_count: u16 = 0;

    // Header
    if footer.is_some() {
        // Step mode: ◆ header
        let _ = write!(buf, "  {BRIGHT_CYAN}\u{25C6}{RESET} {message}\r\n");
    } else {
        // Classic mode: ┌ header
        let _ = write!(buf, "  {CYAN}{CORNER_TL}{RESET} {message}\r\n");
    }
    line_count += 1;

    for (i, opt) in options.iter().enumerate() {
        if i == selected {
            // Selected: bright
            let hint_str = opt.hint.as_deref().map_or(String::new(), |h| format!(" ({h})"));
            let _ = write!(
                buf,
                "  {CYAN}{BAR}{RESET}  \x1b[1m● {}{}{RESET}\r\n",
                opt.label, hint_str
            );
        } else {
            // Not selected: dim
            let hint_str = opt.hint.as_deref().map_or(String::new(), |h| format!(" ({h})"));
            let _ = write!(
                buf,
                "  {CYAN}{BAR}{RESET}  {DIM}○ {}{}{RESET}\r\n",
                opt.label, hint_str
            );
        }
        line_count += 1;
    }

    // Footer
    match footer {
        None => {
            // Classic mode: └
            let _ = write!(buf, "  {CYAN}{CORNER_BL}{RESET}");
        }
        Some(f) if !f.is_empty() => {
            // Step mode with pending steps: append footer content
            let _ = write!(buf, "{f}");
            // Count \r\n in footer for lines_up
            line_count += f.matches("\r\n").count() as u16;
        }
        Some(_) => {
            // Step mode, last step (empty footer): no └, end after last option
        }
    }

    (buf, line_count)
}

/// Erase previous render and redraw.
fn redraw(
    message: &str,
    options: &[SelectOption],
    selected: usize,
    footer: Option<&str>,
    lines_up: u16,
) {
    let mut out = stdout();
    let _ = write!(out, "\x1b[{lines_up}A\r");
    // Clear from cursor to end of screen
    let _ = write!(out, "\x1b[J");
    let (rendered, _) = draw_select(message, options, selected, footer);
    let _ = out.write_all(&rendered);
    let _ = out.flush();
}

// ── Public API ─────────────────────────────────────────────────────────────

/// Single-select with live preview.
///
/// `on_change` is called on every arrow key for live preview updates.
/// `footer` controls step-integrated mode (see `draw_select`).
/// Returns [`SelectResult`].
pub fn select(
    message: &str,
    options: &[SelectOption],
    initial: Option<&str>,
    on_change: &mut dyn FnMut(&str),
    footer: Option<&str>,
) -> SelectResult {
    if options.is_empty() {
        return SelectResult::Cancelled;
    }

    // 1. Find initial index from `initial` value, default to 0
    let mut idx: usize = initial
        .and_then(|v| options.iter().position(|o| o.value == v))
        .unwrap_or(0);

    // 2. Draw the select UI (initial render)
    let mut out = stdout();
    let (rendered, lines_up) = draw_select(message, options, idx, footer);
    let _ = out.write_all(&rendered);
    let _ = out.flush();

    // 3. Enter raw mode + hide cursor
    terminal::enable_raw_mode();
    terminal::hide_cursor();

    // 4. Key loop
    let result = loop {
        match terminal::read_key() {
            Key::Up => {
                if idx > 0 {
                    idx -= 1;
                    on_change(&options[idx].value);
                    redraw(message, options, idx, footer, lines_up);
                }
            }
            Key::Down => {
                if idx < options.len() - 1 {
                    idx += 1;
                    on_change(&options[idx].value);
                    redraw(message, options, idx, footer, lines_up);
                }
            }
            Key::Enter | Key::Right => {
                break SelectResult::Selected(options[idx].value.clone());
            }
            Key::Left => {
                break SelectResult::Back;
            }
            Key::Escape | Key::CtrlC => {
                break SelectResult::Cancelled;
            }
            _ => {}
        }
    };

    // 5. Restore terminal state
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
    fn test_select_result_variants() {
        // Selected variant holds a String
        let sel = SelectResult::Selected("foo".to_string());
        assert_eq!(sel, SelectResult::Selected("foo".to_string()));
        assert_ne!(sel, SelectResult::Back);
        assert_ne!(sel, SelectResult::Cancelled);

        // Back and Cancelled are distinct
        assert_eq!(SelectResult::Back, SelectResult::Back);
        assert_eq!(SelectResult::Cancelled, SelectResult::Cancelled);
        assert_ne!(SelectResult::Back, SelectResult::Cancelled);
    }

    #[test]
    fn test_select_result_debug() {
        assert_eq!(
            format!("{:?}", SelectResult::Selected("x".into())),
            "Selected(\"x\")"
        );
        assert_eq!(format!("{:?}", SelectResult::Back), "Back");
        assert_eq!(format!("{:?}", SelectResult::Cancelled), "Cancelled");
    }

    #[test]
    fn test_select_option_creation() {
        let opt = SelectOption {
            value: "val".to_string(),
            label: "Label".to_string(),
            hint: None,
        };
        assert_eq!(opt.value, "val");
        assert_eq!(opt.label, "Label");
        assert!(opt.hint.is_none());

        let opt_hint = SelectOption {
            value: "v2".to_string(),
            label: "Label 2".to_string(),
            hint: Some("a hint".to_string()),
        };
        assert_eq!(opt_hint.hint.as_deref(), Some("a hint"));
    }

    #[test]
    fn test_draw_select_output_structure() {
        let options = vec![
            SelectOption {
                value: "a".into(),
                label: "Alpha".into(),
                hint: None,
            },
            SelectOption {
                value: "b".into(),
                label: "Beta".into(),
                hint: Some("recommended".into()),
            },
            SelectOption {
                value: "c".into(),
                label: "Gamma".into(),
                hint: None,
            },
        ];

        // Classic mode (no footer)
        let (buf, lines_up) = draw_select("Pick one", &options, 0, None);
        let output = String::from_utf8(buf).unwrap();

        // Header line contains the message
        assert!(output.contains("Pick one"));

        // Selected marker for first option
        assert!(output.contains("● Alpha"));

        // Unselected markers
        assert!(output.contains("○ Beta"));
        assert!(output.contains("○ Gamma"));

        // Hint appears in parentheses
        assert!(output.contains("(recommended)"));

        // Box-drawing characters (classic mode)
        assert!(output.contains("┌"));
        assert!(output.contains("│"));
        assert!(output.contains("└"));

        // lines_up = header(1) + options(3) = 4
        assert_eq!(lines_up, 4);
    }

    #[test]
    fn test_draw_select_step_mode() {
        let options = vec![
            SelectOption {
                value: "a".into(),
                label: "Alpha".into(),
                hint: None,
            },
            SelectOption {
                value: "b".into(),
                label: "Beta".into(),
                hint: None,
            },
        ];

        // Step mode with footer
        let footer = "\r\n  \x1b[2m│\x1b[0m\r\n  \x1b[2m○ 3/4 Reorder\x1b[0m";
        let (buf, lines_up) = draw_select("Pick", &options, 0, Some(footer));
        let output = String::from_utf8(buf).unwrap();

        // Step mode uses ◆ instead of ┌
        assert!(output.contains("\u{25C6}"));
        assert!(!output.contains("┌"));
        assert!(!output.contains("└"));

        // Footer content is appended
        assert!(output.contains("3/4 Reorder"));

        // lines_up = header(1) + options(2) + footer \r\n count(2) = 5
        assert_eq!(lines_up, 5);

        // Step mode with empty footer (last step)
        let (buf2, lines_up2) = draw_select("Pick", &options, 0, Some(""));
        let output2 = String::from_utf8(buf2).unwrap();
        assert!(output2.contains("\u{25C6}"));
        assert!(!output2.contains("└"));
        assert_eq!(lines_up2, 3); // header(1) + options(2)
    }

    #[test]
    fn test_draw_select_different_selection() {
        let options = vec![
            SelectOption {
                value: "a".into(),
                label: "Alpha".into(),
                hint: None,
            },
            SelectOption {
                value: "b".into(),
                label: "Beta".into(),
                hint: None,
            },
        ];

        // Select index 1
        let (buf, _) = draw_select("Test", &options, 1, None);
        let output = String::from_utf8(buf).unwrap();

        // Alpha should be unselected (○), Beta should be selected (●)
        assert!(output.contains("○ Alpha"));
        assert!(output.contains("● Beta"));
    }

    #[test]
    fn test_draw_select_single_option() {
        let options = vec![SelectOption {
            value: "only".into(),
            label: "Only One".into(),
            hint: Some("solo".into()),
        }];

        let (buf, _) = draw_select("Single", &options, 0, None);
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("● Only One (solo)"));
    }

    #[test]
    fn test_select_empty_options_returns_cancelled() {
        let options: Vec<SelectOption> = vec![];
        let result = select("Empty", &options, None, &mut |_| {}, None);
        assert_eq!(result, SelectResult::Cancelled);
    }
}
